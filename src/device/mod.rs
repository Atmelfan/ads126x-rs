use core::marker::PhantomData;

use crate::commands::Command;
use crate::crc8::{checksum, crc_8_atm};
use crate::registers::{Interface, Register, OFCAL0, FSCAL0};

use bitfield::bitfield;
use embedded_hal::{
    blocking::{
        delay::{DelayMs, DelayUs},
        spi::{Transfer, Write},
    },
    digital::v2::{InputPin, OutputPin},
};

pub(crate) mod ads1262;
pub(crate) mod ads1263;

bitfield! {
    /// Status byte returned during a read (if enabled in [Interface] register)
    pub struct Status(u8);
    impl Debug;
    // The fields default to u16
    pub reset, _: 0;
    pub pgad_alm, _: 1;
    pub pgah_alm, _: 2;
    pub pgal_alm, _: 3;
    pub ref_alm, _: 4;
    pub extclk, _: 5;
    pub adc1, _: 6;
    pub adc2, _: 7;
}

impl Status {
    pub fn alarm(&self) -> bool {
        self.pgad_alm() || self.pgah_alm() || self.pgal_alm() || self.ref_alm()
    }
}

mod private {
    pub trait Sealed {}
    impl Sealed for super::ads1262::_Ads1262 {}
    impl Sealed for super::ads1263::_Ads1263 {}
}

/// Data returned from ADC during a read from either ADC1 or ADC2
#[derive(Debug)]
pub struct Data {
    /// Current ADC status
    pub status: Status,
    /// ADC result code
    pub code: i32,
}

impl Data {
    /// Convert adc result into a voltage given a refrence voltage and gain
    pub fn to_voltage(&self, vref: f64, gain: f64) -> f64 {
        f64::from(self.code) * vref / gain / 2147483648.0 /* 2^31 */
    }
}

impl Data {
    fn new(status: Status, code: i32) -> Self {
        Self { status, code }
    }
}

/// Generic ADS1263x device. Use [crate::Ads1262] or [crate::Ads1263] instead.
pub struct Ads126x<SPI, CS, DRDY, RST, X>
where
    X: private::Sealed,
{
    _x: PhantomData<X>,
    spi: SPI,
    cs: CS,
    drdy: DRDY,
    rst: RST,
    interface: Interface,
}

/// Adc error
#[derive(Debug)]
pub enum Ads126xError<SpiErr, IoErr> {
    /// Error during SPI communication
    SpiErr(SpiErr),
    /// Error setting/getting io state
    IoErr(IoErr),
    /// Crc error during read
    Crc,
    /// Some other error
    Other,
}

impl<SPI, CS, DRDY, RST, SpiErr, IoErr, X> Ads126x<SPI, CS, DRDY, RST, X>
where
    SPI: Transfer<u8, Error = SpiErr> + Write<u8, Error = SpiErr>,
    CS: OutputPin<Error = IoErr>,
    RST: OutputPin<Error = IoErr>,
    X: private::Sealed,
{
    /// Reset device by polling RST low for 100ms
    pub fn reset<DELAY>(&mut self, mut delay: DELAY) -> Result<(), Ads126xError<SpiErr, IoErr>>
    where
        DELAY: DelayMs<u32>,
    {
        self.rst.set_high().map_err(|e| Ads126xError::IoErr(e))?;
        delay.delay_ms(100);
        self.rst.set_low().map_err(|e| Ads126xError::IoErr(e))?;
        delay.delay_ms(100);
        self.rst.set_high().map_err(|e| Ads126xError::IoErr(e))?;
        delay.delay_ms(100);
        Ok(())
    }

    /// Read register from device
    /// 
    pub fn read_reg<REG>(&mut self) -> Result<REG, Ads126xError<SpiErr, IoErr>>
    where
        REG: Register,
    {
        let mut data = [Command::RReg.reg(REG::REG), 0x00, 0x00];

        self.cs.set_low().map_err(|e| Ads126xError::IoErr(e))?;
        self.spi
            .transfer(&mut data)
            .map_err(|e| Ads126xError::SpiErr(e))?;
        self.cs.set_high().map_err(|e| Ads126xError::IoErr(e))?;
        Ok(REG::from_byte(data[2]))
    }

    /// Write register to device
    /// 
    pub fn write_reg<REG>(&mut self, reg: REG) -> Result<(), Ads126xError<SpiErr, IoErr>>
    where
        REG: Register,
    {
        let data = [Command::WReg.reg(REG::REG), 0x00, reg.into_byte()];

        self.cs.set_low().map_err(|e| Ads126xError::IoErr(e))?;
        self.spi.write(&data).map_err(|e| Ads126xError::SpiErr(e))?;
        self.cs.set_high().map_err(|e| Ads126xError::IoErr(e))?;
        Ok(())
    }

    fn send_command(&mut self, cmd: Command) -> Result<(), Ads126xError<SpiErr, IoErr>> {
        let data = [cmd as u8];

        self.cs.set_low().map_err(|e| Ads126xError::IoErr(e))?;
        self.spi.write(&data).map_err(|e| Ads126xError::SpiErr(e))?;
        self.cs.set_high().map_err(|e| Ads126xError::IoErr(e))?;
        Ok(())
    }

    fn read_data(&mut self, cmd: Option<Command>) -> Result<Data, Ads126xError<SpiErr, IoErr>> {
        self.cs.set_low().map_err(|e| Ads126xError::IoErr(e))?;

        // Determine length of SPI transfer
        let mut buf = [0; 8];
        let mut len = 4;
        if let Some(cmd) = cmd {
            len += 1;
            buf[0] = cmd as u8;
        }
        if self.interface.status() {
            len += 1;
        }
        if self.interface.crc() > 0 {
            len += 1;
        }

        // SPI transfer
        let mut resp = self
            .spi
            .transfer(&mut buf[..len])
            .map_err(|e| Ads126xError::SpiErr(e))?;

        // First byte is command if used
        if cmd.is_some() {
            resp = &resp[1..];
        }

        // Read status byte if enabled
        let (status, value) = if self.interface.status() {
            let stat = Status(resp[0]);
            let value = &resp[1..5];
            resp = &resp[5..];
            (stat, value)
        } else {
            let value = &resp[0..4];
            resp = &resp[4..];
            (Status(0), value)
        };
        let code = i32::from_be_bytes(value.try_into().map_err(|_| Ads126xError::Other)?);

        // Read CRC if enabled
        if self.interface.crc() > 0 {
            let adc_crc = resp[0];

            // Verify checksum
            let crc = if self.interface.crc() == 1 {
                checksum(value)
            } else if self.interface.crc() == 2 {
                crc_8_atm(value)
            } else {
                0
            };

            if adc_crc != crc {
                return Err(Ads126xError::Crc);
            }
        }

        self.cs.set_high().map_err(|e| Ads126xError::IoErr(e))?;
        Ok(Data::new(status, code))
    }
}

impl<SPI, CS, DRDY, RST, SpiErr, IoErr, X> Ads126x<SPI, CS, DRDY, RST, X>
where
    SPI: Transfer<u8, Error = SpiErr> + Write<u8, Error = SpiErr>,
    CS: OutputPin<Error = IoErr>,
    DRDY: InputPin<Error = IoErr>,
    RST: OutputPin<Error = IoErr>,
    X: private::Sealed,
{
    /// Returns true if DRDY' is active i.e. low.
    ///
    /// **NOTE: ** Only available when DRDY implements embedded_hal [InputPin].
    pub fn drdy(&mut self) -> Result<bool, Ads126xError<SpiErr, IoErr>> {
        self.drdy.is_low().map_err(|e| Ads126xError::IoErr(e))
    }

    /// Busy-sleep loop waiting for DRDY to go low. Probably want to use something else to avoid a lockup if DRDY never goes low.
    ///
    /// **NOTE: ** Only available when DRDY implements embedded_hal [InputPin].
    pub fn wait_drdy<DELAY>(
        &mut self,
        mut delay: DELAY,
        us: u32,
    ) -> Result<(), Ads126xError<SpiErr, IoErr>>
    where
        DELAY: DelayUs<u32>,
    {
        while !self.drdy()? {
            delay.delay_us(us);
        }
        Ok(())
    }
}

impl<SPI, CS, DRDY, RST, SpiErr, IoErr, X> Ads126x<SPI, CS, DRDY, RST, X>
where
    SPI: Transfer<u8, Error = SpiErr> + Write<u8, Error = SpiErr>,
    CS: OutputPin<Error = IoErr>,
    RST: OutputPin<Error = IoErr>,
    X: private::Sealed,
{
    /// Send START1 command to start ADC1 conversion
    pub fn start1(&mut self) -> Result<(), Ads126xError<SpiErr, IoErr>> {
        self.send_command(Command::Start1)
    }

    /// Send STOP1 command to stop ADC1 conversion
    pub fn stop1(&mut self) -> Result<(), Ads126xError<SpiErr, IoErr>> {
        self.send_command(Command::Stop1)
    }

    /// Read data from ADC1
    pub fn read_adc1(&mut self) -> Result<Data, Ads126xError<SpiErr, IoErr>> {
        self.read_data(Some(Command::RData1))
    }

    /// Read data directly from ADC1
    /// 
    /// **NOTE:** This is only valid if DRDY has been asserted low after a conversion being started.
    pub fn read_direct(&mut self) -> Result<Data, Ads126xError<SpiErr, IoErr>> {
        self.read_data(None)
    }

    pub fn read_offs_cal1(&mut self) -> Result<i32, Ads126xError<SpiErr, IoErr>>
    {
        let mut data = [Command::RReg.reg(OFCAL0), 0x02, 0x00, 0x00, 0x00];

        self.cs.set_low().map_err(|e| Ads126xError::IoErr(e))?;
        self.spi
            .transfer(&mut data)
            .map_err(|e| Ads126xError::SpiErr(e))?;
        self.cs.set_high().map_err(|e| Ads126xError::IoErr(e))?;

        let value = (data[2] as i32) | (data[3] as i32) << 8 | (data[4] as i32) << 16;
        Ok(value)
    }

    pub fn read_fs_cal1(&mut self) -> Result<u32, Ads126xError<SpiErr, IoErr>>
    {
        let mut data = [Command::RReg.reg(FSCAL0), 0x02, 0x00, 0x00, 0x00];

        self.cs.set_low().map_err(|e| Ads126xError::IoErr(e))?;
        self.spi
            .transfer(&mut data)
            .map_err(|e| Ads126xError::SpiErr(e))?;
        self.cs.set_high().map_err(|e| Ads126xError::IoErr(e))?;

        let value = (data[2] as u32) | (data[3] as u32) << 8 | (data[4] as u32) << 16;
        Ok(value)
    }

}

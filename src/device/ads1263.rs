use core::marker::PhantomData;

use embedded_hal::{
    blocking::spi::{Transfer, Write},
    digital::v2::OutputPin,
};

use crate::{commands::Command, registers::{ADC2OFC0, ADC2FSC0}};

use super::{Ads126x, Ads126xError, Data};

pub struct _Ads1263;

impl<SPI, CS, DRDY, RST, SpiErr, IoErr> Ads126x<SPI, CS, DRDY, RST, _Ads1263>
where
    SPI: Transfer<u8, Error = SpiErr> + Write<u8, Error = SpiErr>,
    CS: OutputPin<Error = IoErr>,
    RST: OutputPin<Error = IoErr>,
{
    pub fn new(spi: SPI, cs: CS, drdy: DRDY, rst: RST) -> Self {
        Self {
            _x: PhantomData,
            spi,
            cs,
            drdy,
            rst,
            interface: Default::default(),
        }
    }

    /// Send START2 command to start ADC2 conversion
    pub fn start2(&mut self) -> Result<(), Ads126xError<SpiErr, IoErr>> {
        self.send_command(Command::Start2)
    }

    /// Send STOP2 command to stop ADC2 conversion
    pub fn stop2(&mut self) -> Result<(), Ads126xError<SpiErr, IoErr>> {
        self.send_command(Command::Stop2)
    }

    /// Read data from ADC2
    pub fn read_adc2(&mut self) -> Result<Data, Ads126xError<SpiErr, IoErr>> {
        self.read_data(Some(Command::RData1))
    }

    pub fn read_offs_cal2(&mut self) -> Result<i16, Ads126xError<SpiErr, IoErr>>
    {
        let mut data = [Command::RReg.reg(ADC2OFC0), 0x01, 0x00, 0x00];

        self.cs.set_low().map_err(|e| Ads126xError::IoErr(e))?;
        self.spi
            .transfer(&mut data)
            .map_err(|e| Ads126xError::SpiErr(e))?;
        self.cs.set_high().map_err(|e| Ads126xError::IoErr(e))?;

        let value = (data[2] as i16) | (data[3] as i16) << 8;
        Ok(value)
    }

    pub fn read_fs_cal2(&mut self) -> Result<u16, Ads126xError<SpiErr, IoErr>>
    {
        let mut data = [Command::RReg.reg(ADC2FSC0), 0x01, 0x00, 0x00];

        self.cs.set_low().map_err(|e| Ads126xError::IoErr(e))?;
        self.spi
            .transfer(&mut data)
            .map_err(|e| Ads126xError::SpiErr(e))?;
        self.cs.set_high().map_err(|e| Ads126xError::IoErr(e))?;

        let value = (data[2] as u16) | (data[3] as u16) << 8;
        Ok(value)
    }
}

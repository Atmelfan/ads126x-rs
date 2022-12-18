use core::marker::PhantomData;

use embedded_hal::{
    blocking::spi::{Transfer, Write},
    digital::v2::OutputPin,
};

use super::Ads126x;

pub struct _Ads1262;

impl<SPI, CS, DRDY, RST, SpiErr, IoErr> Ads126x<SPI, CS, DRDY, RST, _Ads1262>
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

    //No additional functions
}

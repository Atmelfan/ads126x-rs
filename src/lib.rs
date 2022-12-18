#![no_std]
use device::Ads126x;

/// Commands to send
mod commands;
pub mod crc8;
/// Generic ADS126x device drivers
pub mod device;
pub mod registers;

/// A ADS1262 device
pub type Ads1262<SPI, CS, DRDY, RST> = Ads126x<SPI, CS, DRDY, RST, device::ads1262::_Ads1262>;

/// A ADS1263 device
pub type Ads1263<SPI, CS, DRDY, RST> = Ads126x<SPI, CS, DRDY, RST, device::ads1263::_Ads1263>;

/// Dummy pin used for [Ads126x] DRDY pin when DRDY is not connected.
///
pub struct DrdyNoConnection;

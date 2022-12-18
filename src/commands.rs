#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum Command {
    /// ## NOP Command
    /// The NOP command sends a no operation command to the device. The NOP command opcode is 00h. Hold the
    /// DIN pin low for the NOP command.
    Nop = 0x00,

    /// ## RESET Command
    /// The RESET command resets the ADC operation and resets the device registers to default.
    Reset = 0x06,

    Start1 = 0x08,
    Stop1 = 0x0A,
    Start2 = 0x0C,
    /// ## START1, STOP1, START2, STOP2 Commands
    /// These commands start and stop the conversions of ADC1 and ADC2
    Stop2 = 0x0E,

    RData1 = 0x12,
    /// ## RDATA1, RDATA2 Commands
    /// These commands are used to read ADC1 or ADC2 conversion data from the respective data holding buffers.
    RData2 = 0x14,

    Syocal1 = 0x16,
    Sygcal1 = 0x17,
    Sfocal1 = 0x19,
    Syocal2 = 0x1B,
    Sygcal2 = 0x1C,
    /// ## SYOCAL1, SYGCAL1, SFOCAL1, SYOCAL2, SYGCAL2, SFOCAL2 Commands
    /// These commands are used to calibrate ADC1 or ADC2
    Sfocal2 = 0x1E,

    /// ## RREG Command
    /// Use the RREG command to read the device register data. Read the register data one register at a time, or read
    /// as a block of register data.
    RReg = 0x20,

    /// ## WREG Command
    /// Use the WREG command to write the device register data. The register data are written one register at a time or
    /// as a block of register data.
    WReg = 0x40,
}

impl Command {
    pub fn reg(&self, reg: u8) -> u8 {
        *self as u8 + reg
    }
}

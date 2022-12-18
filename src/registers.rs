//#![allow(dead_code)]// Allow unused register definitions not supported yet

use bitfield::{bitfield, BitMut, Bit};

pub(crate) const ID: u8 = 0x00;
pub(crate) const POWER: u8 = 0x01;
pub(crate) const INTERFACE: u8 = 0x02;
pub(crate) const MODE0: u8 = 0x03;
pub(crate) const MODE1: u8 = 0x04;
pub(crate) const MODE2: u8 = 0x05;
pub(crate) const INPMUX: u8 = 0x06;
pub(crate) const OFCAL0: u8 = 0x07;
//pub(crate) const OFCAL1: u8 = 0x08;
//pub(crate) const OFCAL2: u8 = 0x09;
pub(crate) const FSCAL0: u8 = 0x0A;
//pub(crate) const FSCAL1: u8 = 0x0B;
//pub(crate) const FSCAL2: u8 = 0x0C;
pub(crate) const IDACMUX: u8 = 0x0D;
pub(crate) const IDACMAG: u8 = 0x0E;
pub(crate) const REFMUX: u8 = 0x0F;
pub(crate) const TDACP: u8 = 0x10;
pub(crate) const TDACN: u8 = 0x11;
pub(crate) const GPIOCON: u8 = 0x12;
pub(crate) const GPIODIR: u8 = 0x13;
pub(crate) const GPIODAT: u8 = 0x14;
pub(crate) const ADC2CFG: u8 = 0x15;
pub(crate) const ADC2MUX: u8 = 0x16;
pub(crate) const ADC2OFC0: u8 = 0x17;
//pub(crate) const ADC2OFC1: u8 = 0x18;
pub(crate) const ADC2FSC0: u8 = 0x19;
//pub(crate) const ADC2FSC1: u8 = 0x1A;

pub trait Register {
    const REG: u8;

    fn from_byte(b: u8) -> Self;
    fn into_byte(&self) -> u8;
}

macro_rules! impl_register {
    ($typ:ident, $reg:ident = $def:literal) => {
        impl Default for $typ {
            fn default() -> Self {
                Self($def)
            }
        }

        impl_register!($typ, $reg);
    };
    ($typ:ident, $reg:expr) => {
        impl Register for $typ {
            const REG: u8 = $reg;

            fn from_byte(b: u8) -> Self {
                Self(b)
            }

            fn into_byte(&self) -> u8 {
                self.0
            }
        }
    };
}

bitfield! {
    pub struct Id(u8);
    impl Debug;
    // The fields default to u16
    pub rev_id, _: 4, 0;
    pub dev_id, _: 7, 5;
}
impl_register!(Id, ID = 0);

bitfield! {
    pub struct Power(u8);
    impl Debug;
    // The fields default to u16
    pub intref, set_intref: 0;
    pub vbias, set_vbias: 1;
    pub reset, set_reset: 4;
}
impl_register!(Power, POWER = 0x11);
impl Power {
    pub fn with(intref: bool, vbias: bool, reset: bool) -> Self {
        let mut this = Self(0);
        this.set_intref(intref);
        this.set_vbias(vbias);
        this.set_reset(reset);
        this
    }
}

pub enum InterfaceCrc {
    Off = 0,
    Checksum = 1,
    Crc = 2,
}

bitfield! {
    pub struct Interface(u8);
    impl Debug;
    // The fields default to u16
    pub crc, set_crc: 1, 0;
    pub status, set_status : 2;
    pub time_out, set_time_out : 3;
}
impl_register!(Interface, INTERFACE = 0x05);

pub enum Mode0Delay {
    None = 0,
    Us8_7 = 1,
    Us17 = 2,
    Us35 = 3,
    Us69 = 4,
    Us139 = 5,
    Us278 = 6,
    Us555 = 7,
    Ms1_1 = 8,
    Ms2_2 = 9,
    Ms4_4 = 10,
    Ms8_8 = 11,
}

pub enum Mode0Chop {
    Disabled = 0,
    Input = 1,
    Idac = 2,
    InputAndIdac = 3,
}

bitfield! {
    pub struct Mode0(u8);
    impl Debug;
    // The fields default to u16
    pub delay, set_delay: 3, 0;
    pub chop, set_chop: 5, 4;
    pub run_mode, set_run_mode: 6;
    pub refrev, set_refrev: 7;
}
impl_register!(Mode0, MODE0 = 0x00);
impl Mode0 {
    pub fn with(delay: Mode0Delay, chop: Mode0Chop, oneshot: bool, refrev: bool) -> Self {
        let mut this = Self(0);
        this.set_delay(delay as u8);
        this.set_chop(chop as u8);
        this.set_run_mode(oneshot);
        this.set_refrev(refrev);
        this
    }
}

pub enum Mode1Filter {
    Sinc1 = 0,
    Sinc2 = 1,
    Sinc3 = 2,
    Sinc4 = 3,
    Fir = 4,
}

pub enum Mode1SBMag {
    None = 0,
    Ua0_5 = 1,
    Ua2 = 2,
    Ua10 = 3,
    Ua50 = 4,
    Ua200 = 5,
    Res = 6,
}

bitfield! {
    pub struct Mode1(u8);
    impl Debug;
    // The fields default to u16
    pub sbmag, set_sbmag: 2, 0;
    pub sbpol, set_sbpol: 3;
    pub sbadc, set_sbadc: 4;
    pub filter, set_filter: 7, 5;
}
impl_register!(Mode1, MODE1 = 0x80);
impl Mode1 {
    pub fn with(sbmag: Mode1SBMag, sbpol: bool, sbadc: bool, filter: Mode1Filter) -> Self {
        let mut this = Self(0);
        this.set_sbmag(sbmag as u8);
        this.set_sbpol(sbpol);
        this.set_sbadc(sbadc);
        this.set_filter(filter as u8);
        this
    }
}

pub enum Mode2Gain {
    None = 0,
    Gain2 = 1,
    Gain4 = 2,
    Gain8 = 3,
    Gain16 = 4,
    Gain32 = 5,
}

impl Mode2Gain {
    pub fn gain(&self) -> f32 {
        match self {
            Mode2Gain::None => 1.0,
            Mode2Gain::Gain2 => 2.0,
            Mode2Gain::Gain4 => 4.0,
            Mode2Gain::Gain8 => 8.0,
            Mode2Gain::Gain16 => 16.0,
            Mode2Gain::Gain32 => 32.0,
        }
    }
}

pub enum Mode2Dr {
    Sps2_5 = 0,
    Sps5 = 1,
    Sps10 = 2,
    Sps16_6 = 3,
    Sps20 = 4,
    Sps50 = 5,
    Sps60 = 6,
    Sps100 = 7,
    Sps400 = 8,
    Sps1200 = 9,
    Sps2400 = 10,
    Sps4800 = 11,
    Sps7200 = 12,
    Sps14400 = 13,
    Sps19200 = 14,
    Sps38400 = 15,
}

bitfield! {
    pub struct Mode2(u8);
    impl Debug;
    // The fields default to u16
    pub dr, set_dr: 3, 0;
    pub gain, set_gain: 6, 4;
    pub bypass, set_bypass: 7;
}
impl_register!(Mode2, MODE2 = 0x04);
impl Mode2 {
    pub fn with(dr: Mode2Dr, gain: Mode2Gain, bypass: bool) -> Self {
        let mut this = Self(0);
        this.set_dr(dr as u8);
        this.set_gain(gain as u8);
        this.set_bypass(bypass);
        this
    }
}

pub enum InpMuxMuxx {
    Ain0 = 0,
    Ain1 = 1,
    Ain2 = 2,
    Ain3 = 3,
    Ain4 = 4,
    Ain5 = 5,
    Ain6 = 6,
    Ain7 = 7,
    Ain8 = 8,
    Ain9 = 9,
    AinCom = 10,
    Temperature = 11,
    AnalogPower = 12,
    DigitalPower = 13,
    Tdac = 14,
    Float = 15,
}

bitfield! {
    pub struct InpMux(u8);
    impl Debug;
    // The fields default to u16
    pub muxn, set_muxn: 3, 0;
    pub muxp, set_muxp: 7, 4;
}
impl_register!(InpMux, INPMUX = 0x01);
impl InpMux {
    pub fn differential(inp: InpMuxMuxx, inn: InpMuxMuxx) -> Self {
        let mut this = Self(0);
        this.set_muxn(inn as u8);
        this.set_muxp(inp as u8);
        this
    }

    pub fn singleended(inp: InpMuxMuxx) -> Self {
        let mut this = Self(0);
        this.set_muxn(InpMuxMuxx::AinCom as u8);
        this.set_muxp(inp as u8);
        this
    }

    pub fn temperature() -> Self {
        Self::differential(InpMuxMuxx::Temperature, InpMuxMuxx::Temperature)
    }

    pub fn analog_power() -> Self {
        Self::differential(InpMuxMuxx::AnalogPower, InpMuxMuxx::AnalogPower)
    }

    pub fn digital_power() -> Self {
        Self::differential(InpMuxMuxx::DigitalPower, InpMuxMuxx::DigitalPower)
    }

    pub fn tdac() -> Self {
        Self::differential(InpMuxMuxx::Tdac, InpMuxMuxx::Tdac)
    }
}

pub enum IdacMuxMuxx {
    Ain0 = 0,
    Ain1 = 1,
    Ain2 = 2,
    Ain3 = 3,
    Ain4 = 4,
    Ain5 = 5,
    Ain6 = 6,
    Ain7 = 7,
    Ain8 = 8,
    Ain9 = 9,
    AinCom = 10,
    NoConnection = 11,
}

bitfield! {
    pub struct IdacMux(u8);
    impl Debug;
    // The fields default to u16
    pub mux1, _: 3, 0;
    pub mux2, _ : 7, 4;
}
impl_register!(IdacMux, IDACMUX = 0xBB);

pub enum IdacMagMagx {
    Off = 0,
    Ua50 = 1,
    Ua100 = 2,
    Ua250 = 3,
    Ua500 = 4,
    Ua750 = 5,
    Ua1000 = 6,
    Ua1500 = 7,
    Ua2000 = 8,
    Ua2500 = 9,
    Ua3000 = 10,
}

bitfield! {
    pub struct IdacMag(u8);
    impl Debug;
    // The fields default to u16
    pub mag1, _: 3, 0;
    pub mag2, _ : 7, 4;
}
impl_register!(IdacMag, IDACMAG = 0x00);

pub enum RefMuxMuxP {
    InternalRefP = 0,
    Ain0 = 1,
    Ain2 = 2,
    Ain4 = 3,
    Avdd = 4,
}

pub enum RefMuxMuxN {
    InternalRefN = 0,
    Ain0 = 1,
    Ain2 = 2,
    Ain4 = 3,
    Avss = 4,
}

bitfield! {
    pub struct RefMux(u8);
    impl Debug;
    // The fields default to u16
    pub rmuxn, _: 2, 0;
    pub rmuxp, _ : 5, 3;
}
impl_register!(RefMux, REFMUX = 0x00);

bitfield! {
    pub struct TdacP(u8);
    impl Debug;
    // The fields default to u16
    pub magp, _: 4, 0;
    pub outp, _ : 7;
}
impl_register!(TdacP, TDACP = 0x00);

bitfield! {
    pub struct TdacN(u8);
    impl Debug;
    // The fields default to u16
    pub magn, _: 4, 0;
    pub outn, _ : 7;
}
impl_register!(TdacN, TDACN = 0x00);

bitfield! {
    pub struct GpioCon(u8);
    impl Debug;
    // The fields default to u16
    pub con, _: 7, 0;
}
impl_register!(GpioCon, GPIOCON = 0x00);
impl GpioCon {
    pub fn set_connected(&mut self, idx: usize) {
        self.set_bit(idx, true)
    }

    pub fn set_unconnected(&mut self, idx: usize) {
        self.set_bit(idx, false)
    }
}

bitfield! {
    pub struct GpioDir(u8);
    impl Debug;
    // The fields default to u16
    pub dir, _: 7, 0;
}
impl_register!(GpioDir, GPIODIR = 0x00);
impl GpioDir {
    pub fn set_input(&mut self, idx: usize) {
        self.set_bit(idx, true)
    }

    pub fn set_output(&mut self, idx: usize) {
        self.set_bit(idx, false)
    }
}

bitfield! {
    pub struct GpioDat(u8);
    impl Debug;
    // The fields default to u16
    pub dat, _: 7, 0;
}
impl_register!(GpioDat, GPIODAT = 0x00);
impl GpioDat {
    pub fn is_high(&mut self, idx: usize) -> bool {
        self.bit(idx)
    }

    pub fn is_low(&mut self, idx: usize) -> bool {
        !self.bit(idx)
    }
}

pub enum Adc2CfgDr {
    Sps10 = 0,
    Sps100 = 1,
    Sps400 = 2,
    Sps800 = 3,
}

pub enum Adc2CfgRef {
    InternalRef = 0,
    Ain01 = 1,
    Ain23 = 2,
    Ain45 = 3,
    AvddAvss = 4,
}

pub enum Adc2CfgGain {
    None = 0,
    Gain2 = 1,
    Gain4 = 2,
    Gain8 = 3,
    Gain16 = 4,
    Gain32 = 5,
    Gain64 = 6,
    Gain128 = 7,
}

impl Adc2CfgGain {
    pub fn gain(&self) -> f32 {
        match self {
            Adc2CfgGain::None => 1.0,
            Adc2CfgGain::Gain2 => 2.0,
            Adc2CfgGain::Gain4 => 4.0,
            Adc2CfgGain::Gain8 => 8.0,
            Adc2CfgGain::Gain16 => 16.0,
            Adc2CfgGain::Gain32 => 32.0,
            Adc2CfgGain::Gain64 => 64.0,
            Adc2CfgGain::Gain128 => 128.0,
        }
    }
}

bitfield! {
    pub struct Adc2Cfg(u8);
    impl Debug;
    // The fields default to u16
    pub gain2, _: 2, 0;
    pub ref2, _ : 5, 3;
    pub dr2, _ : 7, 6;
}
impl_register!(Adc2Cfg, ADC2CFG = 0x00);

bitfield! {
    pub struct Adc2Mux(u8);
    impl Debug;
    // The fields default to u16
    pub muxn2, _: 3, 0;
    pub muxp2, _ : 7, 4;
}
impl_register!(Adc2Mux, ADC2MUX = 0x01);

use ads126x::{
    registers::{
        Id, InpMux, InpMuxMuxx, Mode0, Mode0Chop, Mode0Delay, Mode2, Mode2Dr, Mode2Gain, Power,
    },
    Ads1263,
};
use clap::Parser;
use linux_embedded_hal::{
    gpio_cdev::{Chip, LineRequestFlags},
    spidev::{SpiModeFlags, SpidevOptions},
    CdevPin, Spidev,
};

/// Defaults are set for a Raspberry pi with Waveshare High-Precision AD HAT (see https://www.waveshare.com/wiki/High-Precision_AD_HAT).
#[derive(Parser, Debug)]
struct Args {
    /// SPI interface path
    #[clap(long, default_value = "/dev/spidev0.0")]
    spi: String,

    /// GPIO cdev interface path
    #[clap(long, default_value = "/dev/gpiochip0")]
    gpio: String,

    /// CS pin
    #[clap(long, default_value_t = 22)]
    cs: u32,

    /// DRDY pin
    #[clap(long, default_value_t = 17)]
    drdy: u32,

    /// RST pin
    #[clap(long, default_value_t = 18)]
    rst: u32,

    /// Enable VBIAS on AINCOM
    #[clap(long)]
    vbias: bool,

    /// Channels to measure
    channels: Vec<String>,
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();

    // Initialize GPIOs
    let mut gpio = Chip::new(args.gpio).expect("Failed to init GPIO controller");
    let cs_handle = gpio
        .get_line(args.cs)
        .unwrap()
        .request(LineRequestFlags::OUTPUT, 1, "ads1263-rs")
        .unwrap();
    let cs = CdevPin::new(cs_handle).unwrap();
    let drdy_handle = gpio
        .get_line(args.drdy)
        .unwrap()
        .request(LineRequestFlags::INPUT, 1, "ads1263-rs")
        .unwrap();
    let drdy = CdevPin::new(drdy_handle).unwrap();
    let rst_handle = gpio
        .get_line(args.rst)
        .unwrap()
        .request(LineRequestFlags::OUTPUT, 1, "ads1263-rs")
        .unwrap();
    let rst = CdevPin::new(rst_handle).unwrap();

    // Initialize SPI
    let mut spi = Spidev::open(args.spi)?;
    let mut spi_options = SpidevOptions::default();
    spi_options.mode(SpiModeFlags::SPI_MODE_1);
    spi_options.max_speed_hz(1_000_000);
    spi.configure(&spi_options).unwrap();

    // Initialize device
    let mut ads1263 = Ads1263::new(spi, cs, drdy, rst);
    ads1263.reset(linux_embedded_hal::Delay).unwrap();

    // Read device id
    let id = ads1263.read_reg::<Id>().unwrap();
    println!("ID = {id:?}");
    assert!(
        id.dev_id() == 1,
        "Expected ADS1263 with DEV_ID = 1, got {}",
        id.dev_id()
    );

    // Enable VBIAS
    if args.vbias {
        let power = Power::with(true, true, true);
        ads1263.write_reg(power).unwrap();
    }

    let mode0 = Mode0::with(Mode0Delay::None, Mode0Chop::Disabled, true, false);
    ads1263.write_reg(mode0).unwrap();

    let mode2 = Mode2::with(Mode2Dr::Sps2_5, Mode2Gain::None, false); //2.5SPS, gain = 1x,
    ads1263.write_reg(mode2).unwrap();

    let ofcal = ads1263.read_offs_cal1().unwrap();
    let fscal = ads1263.read_fs_cal1().unwrap();
    dbg!(ofcal, fscal);


    for ch in &args.channels {
        // Configure mux
        let mux = if let Some((inp, inn)) = ch.split_once(':') {
            let inp: u8 = inp.parse().unwrap();
            let inn: u8 = inn.parse().unwrap();
            let mut mux = InpMux(0);
            mux.set_muxp(inp);
            mux.set_muxn(inn);
            mux
        } else {
            let inp: u8 = ch.parse().unwrap();
            let mut mux = InpMux(0);
            mux.set_muxp(inp);
            mux.set_muxn(InpMuxMuxx::AinCom as u8);
            mux
        };
        ads1263.write_reg(mux).unwrap();

        // Start ADC1
        ads1263.start1().unwrap();

        // Wait for DRDY
        ads1263.wait_drdy(linux_embedded_hal::Delay, 100).unwrap();

        let data = ads1263.read_direct().unwrap();
        println!(
            "* {ch} = {}({:.5} V), status = {:?}",
            data.code,
            data.to_voltage(2.5, 1.0),
            data.status
        );
    }

    Ok(())
}

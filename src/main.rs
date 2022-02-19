//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

/// Prefix for the controller data
const CONTROLLER_NAME: &str = "CLASSIC";

/// Whether we're dumping hi-res data or not
const HIRES_MODE: bool = true;

/// List of fields we're collecting data for
#[rustfmt::skip]
const ANALOG_INPUT: [&str; 19] = [
    "IDLE",
    "LJOY_U", "LJOY_D", "LJOY_L", "LJOY_R",
    "RJOY_U", "RJOY_D", "RJOY_L", "RJOY_R",
    "LTRIG", "RTRIG",
    "LJOY_UR", "LJOY_DR", "LJOY_DL", "LJOY_UL",
    "RJOY_UR", "RJOY_DR", "RJOY_DL", "RJOY_UL",
];

const MILLISECONDS_BETWEEN_PROMPTS: u32 = 3000;

use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::blocking::i2c::{Read, Write};
use embedded_time::{fixed_point::FixedPoint, rate::Extensions};
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    gpio::FunctionI2C,
    pac,
    sio::Sio,
    watchdog::Watchdog,
    I2C,
};

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let sda_pin = pins.gpio4.into_mode::<FunctionI2C>();
    let scl_pin = pins.gpio5.into_mode::<FunctionI2C>();

    let mut i2c = I2C::i2c0(
        pac.I2C0,
        sda_pin,
        scl_pin,
        100u32.kHz(),
        &mut pac.RESETS,
        clocks.peripheral_clock,
    );

    delay.delay_us(100);
    let nunchuck_addr = 0x52;
    let _ = i2c.write(nunchuck_addr, &[0xF0, 0x55]).unwrap();

    delay.delay_us(20);
    let _ = i2c.write(nunchuck_addr, &[0xFB, 0x00]).unwrap();

    delay.delay_us(100);

    let hires_mode = HIRES_MODE;

    let mut i2c_id: [u8; 6] = [0; 6];

    i2c.write(nunchuck_addr, &[0xfa]).unwrap();
    delay.delay_us(100);
    i2c.read(nunchuck_addr, &mut i2c_id).unwrap();
    // Dump the ID for this controller
    println!(
        "pub const {}_ID: ExtReport = [{}, {}, {}, {}, {}, {}];",
        CONTROLLER_NAME, i2c_id[0], i2c_id[1], i2c_id[2], i2c_id[3], i2c_id[4], i2c_id[5]
    );
    delay.delay_us(100);
    i2c.write(nunchuck_addr, &[0xFE]).unwrap();
    delay.delay_us(100);
    i2c.read(nunchuck_addr, &mut i2c_id).unwrap();
    delay.delay_us(100);

    // Dump the ID for this controller
    println!(
        "pub const {}_HIRES_DEFAULT: u8 = {};",
        CONTROLLER_NAME, i2c_id[0]
    );
    // High_res disable = 0x01
    // To enable High Resolution Mode, you simply write 0x03 to address 0xFE in the extension controller memory.
    // Then you poll the controller by reading 8 bytes at address 0x00 instead of only 6.
    if hires_mode {
        i2c.write(nunchuck_addr, &[0xFE, 0x03]).unwrap();
    }

    loop {
        for input in ANALOG_INPUT {
            // Give the user time to press the controller
            println!("// Input {}", input);
            delay.delay_ms(MILLISECONDS_BETWEEN_PROMPTS);
            if !hires_mode {
                delay.delay_ms(6);
                i2c.write(nunchuck_addr, &[0]).unwrap();
                delay.delay_ms(10);
                let mut i2cdata: [u8; 6] = [0u8; 6];
                //let s = i2c.write_read(nunchuck_addr, &[0], &mut i2cdata).unwrap();
                i2c.read(nunchuck_addr, &mut i2cdata).unwrap();
                // println!(
                //     "data is {:03} {:03} {:03} {:03} {:03} {:03}",
                //     i2cdata[0], i2cdata[1], i2cdata[2], i2cdata[3], i2cdata[4], i2cdata[5]
                // );

                println!(
                    "pub const {}_{}: ExtReport = [{}, {}, {}, {}, {}, {}];",
                    CONTROLLER_NAME,
                    input,
                    i2cdata[0],
                    i2cdata[1],
                    i2cdata[2],
                    i2cdata[3],
                    i2cdata[4],
                    i2cdata[5]
                );
            } else {
                delay.delay_ms(6);
                i2c.write(nunchuck_addr, &[0]).unwrap();
                delay.delay_ms(10);

                //let s = i2c.write_read(nunchuck_addr, &[0], &mut i2cdata).unwrap();

                if hires_mode {
                    // 8byte report
                    let mut i2cdata: [u8; 8] = [0u8; 8];
                    i2c.read(nunchuck_addr, &mut i2cdata).unwrap();
                    println!(
                        // "data is {:03} {:03} {:03} {:03} {:03} {:03} {:03} {:03}",
                        "pub const {}_HD_{}: ExtHdReport = [{}, {}, {}, {}, {}, {}, {}, {}];",
                        CONTROLLER_NAME,
                        input,
                        i2cdata[0],
                        i2cdata[1],
                        i2cdata[2],
                        i2cdata[3],
                        i2cdata[4],
                        i2cdata[5],
                        i2cdata[6],
                        i2cdata[7]
                    );
                } else {
                    let mut i2cdata: [u8; 6] = [0u8; 6];
                    i2c.read(nunchuck_addr, &mut i2cdata).unwrap();
                    println!(
                        "pub const READING: ExtReport = [{}, {}, {}, {}, {}, {}];",
                        i2cdata[0], i2cdata[1], i2cdata[2], i2cdata[3], i2cdata[4], i2cdata[5]
                    );
                }
            }
        }
        println!("// Sample complete!");
        loop {
            delay.delay_ms(5000);
        }
    }
}

// End of file

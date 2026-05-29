#![allow(unknown_lints)]
#![allow(unexpected_cfgs)]

use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use std::thread;
use std::time::Duration;
use esp_idf_hal::adc::config::Config;
use esp_idf_hal::adc::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::sys::adc_atten_t; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_idf_hal::gpio::PinDriver;


#[cfg(any(feature = "adc-oneshot-legacy", esp_idf_version_major = "4"))]
fn main() -> anyhow::Result<()> {
    const ATTENUATION: adc_atten_t = attenuation::DB_12;

    let peripherals = Peripherals::take()?;

    let mut led = PinDriver::output(peripherals.pins.gpio21).unwrap();

    led.set_low().unwrap();

    led.set_high().unwrap();
    thread::sleep(Duration::from_millis(300));
    led.set_low().unwrap();
    thread::sleep(Duration::from_millis(300));
    led.set_high().unwrap();
    thread::sleep(Duration::from_millis(300));
    led.set_low().unwrap();
    thread::sleep(Duration::from_millis(300));
    led.set_high().unwrap();
    thread::sleep(Duration::from_millis(300));
    led.set_low().unwrap();
    thread::sleep(Duration::from_millis(300));

    led.set_high().unwrap();


    let mut adc = AdcDriver::new(peripherals.adc1, &Config::new().calibration(true))?;

    let mut adc_pin: esp_idf_hal::adc::AdcChannelDriver<{ ATTENUATION }, _> =
        AdcChannelDriver::new(peripherals.pins.gpio32)?;

    loop {
        thread::sleep(Duration::from_millis(100));
        println!("Sound. ADC value: {}", adc.read(&mut adc_pin)?);
    }
}

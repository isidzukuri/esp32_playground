//! ADC example, reading a value form a pin and printing it on the terminal
//!

#![allow(unknown_lints)]
#![allow(unexpected_cfgs)]

use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use std::thread;
use std::time::Duration;


#[cfg(any(feature = "adc-oneshot-legacy", esp_idf_version_major = "4"))]
fn main() -> anyhow::Result<()> {
    use esp_idf_hal::adc::config::Config;
    use esp_idf_hal::adc::*;
    use esp_idf_hal::peripherals::Peripherals;
    use esp_idf_hal::sys::adc_atten_t; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

    let peripherals = Peripherals::take()?;

    // #[cfg(not(esp32))]
    let mut adc = AdcDriver::new(peripherals.adc1, &Config::new().calibration(true))?;

    // #[cfg(esp32)]
    // let mut adc = AdcDriver::new(peripherals.adc2, &Config::new().calibration(true))?;

    const ATTENUATION: adc_atten_t = attenuation::DB_12;

    // configuring pin to analog read, you can regulate the adc input voltage range depending on your need
    // for this example we use the attenuation of 11db which sets the input voltage range to around 0-3.6V
    // #[cfg(not(esp32))]
    // let mut adc_pin: esp_idf_hal::adc::AdcChannelDriver<{ ATTENUATION }, _> =
    //     AdcChannelDriver::new(peripherals.pins.gpio4)?;

    #[cfg(esp32)]
    let mut adc_pin: esp_idf_hal::adc::AdcChannelDriver<{ ATTENUATION }, _> =
        AdcChannelDriver::new(peripherals.pins.gpio32)?;

    loop {
        // you can change the sleep duration depending on how often you want to sample
        thread::sleep(Duration::from_millis(100));
        println!("ADC value: {}", adc.read(&mut adc_pin)?);
    }
}

// #[cfg(esp_idf_version_major = "4")]
// fn main() {
//     println!("Building with ESP-IDF v4.x");
// }

// #[cfg(esp_idf_version_major = "5")]
// fn main() {
//     println!("Building with ESP-IDF v5.x");
// }

// #[cfg(not(any(feature = "adc-oneshot-legacy", esp_idf_version_major = "4")))]
// fn main() -> anyhow::Result<()> {
//     println!("This example requires feature `adc-oneshot-legacy` enabled or using ESP-IDF v4.4.X");

//     loop {
//         thread::sleep(Duration::from_millis(1000));
//     }
// }










// use std::thread;
// use std::time::Duration;

// use esp_idf_hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
// use esp_idf_hal::adc::{config::Config, Atten11dB, ADC1};
// use esp_idf_hal::gpio::PinDriver;
// use esp_idf_hal::peripherals::Peripherals;
// use esp_idf_svc::sys::link_patches;
// use esp_idf_svc::log::EspLogger;

// fn main() {
//     // 1. Basic System Initialization
//     link_patches();
//     EspLogger::initialize_default();

//     let peripherals = Peripherals::take().unwrap();

//     // 2. Configure ADC1 on GPIO 32
//     // calibration(true) is essential for linear, accurate readings
//     let adc_config = Config::new().calibration(true);
//     let mut adc = AdcDriver::new(peripherals.adc1, &adc_config).unwrap();
//     let mut adc_pin = AdcChannelDriver::<Atten11dB<ADC1>, _>::new(peripherals.pins.gpio32).unwrap();

//     // 3. Define Threshold
//     const THRESHOLD: u16 = 2000; 
//     log::info!("Sound monitoring started on GPIO 32...");

//     // 4. Monitoring Loop
//     loop {
//         // Perform a oneshot read
//         match adc.read(&mut adc_pin) {
//             Ok(value) => {
//                 // Log the value for debugging (optional: keep this minimal to save CPU)
//                 log::debug!("Raw Sound Value: {}", value);

//                 if value > THRESHOLD {
//                     log::warn!("ALERT: High Sound Intensity Detected! Value: {}", value);
//                 }
//             }
//             Err(e) => {
//                 log::error!("Failed to read ADC: {:?}", e);
//             }
//         }

//         // Sampling interval
//         thread::sleep(Duration::from_millis(30));
//     }
// }
















// // use std::thread;
// // use std::time::Duration;

// // use esp_idf_hal::adc::{config::Config, attenuation, AdcContDriver, AdcDriver};
// // use esp_idf_hal::gpio::PinDriver;
// // use esp_idf_hal::peripherals::Peripherals;

// // fn main() {
// //     // It is necessary to call this function once. Otherwise, some patches to the runtime
// //     // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
// //     esp_idf_svc::sys::link_patches();

// //     // Bind the log crate to the ESP Logging facilities
// //     esp_idf_svc::log::EspLogger::initialize_default();

// //     log::info!("Starting LED blink on GPIO21...");

// //     // Get peripherals and configure GPIO21 as output
// //     let peripherals = Peripherals::take().unwrap();
// //     let mut led = PinDriver::output(peripherals.pins.gpio21).unwrap();


// //     led.set_low().unwrap();

// //     led.set_high().unwrap();
// //     thread::sleep(Duration::from_millis(300));
// //     led.set_low().unwrap();
// //     thread::sleep(Duration::from_millis(300));
// //     led.set_high().unwrap();
// //     thread::sleep(Duration::from_millis(300));
// //     led.set_low().unwrap();
// //     thread::sleep(Duration::from_millis(300));
// //     led.set_high().unwrap();
// //     thread::sleep(Duration::from_millis(300));
// //     led.set_low().unwrap();
// //     thread::sleep(Duration::from_millis(300));

// //     led.set_high().unwrap();

    
// //     // for i in 0..500 {
// //     //     led.set_high().unwrap();
// //     //     thread::sleep(Duration::from_millis(500));
// //     //     led.set_low().unwrap();
// //     //     thread::sleep(Duration::from_millis(500));
// //     //     log::info!("Blink {}", i + 1);
// //     // }

// //     log::info!("LED blinking complete!");

// //     // Sound sensor code starts here
// //     // let mut sound_digital = PinDriver::input(peripherals.pins.gpio19).unwrap();
// //     let mut adc = AdcDriver::new(peripherals.adc1, &Config::new().calibration(true)).unwrap();
// //     let mut sound_analog: AdcChannelDriver<{ attenuation::DB_12 }, _> =
// //         AdcChannelDriver::new(peripherals.pins.gpio36).unwrap();

// //     loop {
// //         // let digital_value = sound_digital.is_high().unwrap();
// //         let analog_mv = adc.read(&mut sound_analog).unwrap();

// //         // if digital_value {
// //         //     led.set_high().unwrap();
// //         // } else {
// //         //     led.set_low().unwrap();
// //         // }

// //         log::info!("sound sensor: digital=-- analog={}mV", analog_mv);
// //         // log::info!("sound sensor: digital={} analog={}mV", digital_value as u8, analog_mv);
// //         thread::sleep(Duration::from_millis(200));
// //     }
// // }

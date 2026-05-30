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

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::{EspNvsPartition, NvsDefault};
use esp_idf_svc::mdns::EspMdns;

mod web_server;
mod wifi_access_point;


#[cfg(any(feature = "adc-oneshot-legacy", esp_idf_version_major = "4"))]
fn main() {

    let peripherals = Peripherals::take().unwrap();

    let mut led = PinDriver::output(peripherals.pins.gpio21).unwrap();
    led_hello(&mut led);


    println!("Initializing Wi-Fi Access Point...");
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspNvsPartition::<NvsDefault>::take().unwrap();
    let wifi = wifi_access_point::init_ap(peripherals.modem, sys_loop, nvs);
    println!("Access Point is running! {:?}", wifi.get_configuration().unwrap());



    println!("Initializing DNS...");
    let mut mdns = EspMdns::take().unwrap();
    mdns.set_hostname("sensors").unwrap();
    mdns.set_instance_name("ESP32 Sensors").unwrap();
    mdns.add_service(None, "_http", "_tcp", 80, &[("path", "/")]).unwrap();
    println!("mDNS responder started: http://sensors.local");

    // println!("Web server accessible at http://192.168.4.1");

    println!("Initializing Web Server...");

    // start web server (keep Arc to keep server alive)
    let _server = web_server::start_web_server("/sd/log.csv").unwrap();
    println!("Web Server started.");


    
    let mut adc = AdcDriver::new(peripherals.adc1, &Config::new().calibration(true)).unwrap();
    let mut adc_pin: esp_idf_hal::adc::AdcChannelDriver<{ attenuation::DB_12 }, _> =
        AdcChannelDriver::new(peripherals.pins.gpio32).unwrap();

    loop {
        thread::sleep(Duration::from_millis(100));
        // println!("Sound. ADC value: {}", adc.read(&mut adc_pin).unwrap());
    }
}

fn led_hello<MODE: esp_idf_hal::gpio::OutputMode>(led: &mut PinDriver<MODE>){
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
}

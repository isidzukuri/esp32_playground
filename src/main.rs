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
mod sd_card;
mod threads_controller;

use std::fs::{File, OpenOptions};
use std::io::{Write, BufRead, BufReader};


fn spawn_thread_demo() {
    const STACK_SIZE: u32 = 4096;

    // Thread 1 on Core 0 (PRO_CPU)
    threads_controller::spawn_pinned_task("Thread 1", STACK_SIZE, 0, || loop {
        println!("Thread 1 - running on Core 0");
        std::thread::sleep(Duration::from_millis(100));
    });

    // Thread 2 on Core 0 (PRO_CPU)
    threads_controller::spawn_pinned_task("Thread 2", STACK_SIZE, 0, || loop {
        println!("Thread 2 - running on Core 0");
        std::thread::sleep(Duration::from_millis(100));
    });

    // Thread 3 on Core 1 (APP_CPU)
    threads_controller::spawn_pinned_task("Thread 3", STACK_SIZE, 1, || loop {
        println!("Thread 3 - running on Core 1");
        std::thread::sleep(Duration::from_millis(100));
    });
}



#[cfg(any(feature = "adc-oneshot-legacy", esp_idf_version_major = "4"))]
fn main() {

    let peripherals = Peripherals::take().unwrap();
    spawn_thread_demo();

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

    println!("Initializing Web Server...");
    // start web server (keep Arc to keep server alive)
    let _server = web_server::start_web_server("/sd/log.csv").unwrap();
    println!("Web Server started.");

    let mut adc = AdcDriver::new(peripherals.adc1, &Config::new().calibration(true)).unwrap();
    let mut adc_pin: esp_idf_hal::adc::AdcChannelDriver<{ attenuation::DB_12 }, _> =
        AdcChannelDriver::new(peripherals.pins.gpio32).unwrap();

    // 1. Mount the physical SD card 
    let _card_handle = sd_card::mount_sd_card();//.unwrap();

    // 2. Write a file using standard std::io error mapping
    println!("Writing data sample to file...");
    
    {
        let mut file = OpenOptions::new()
            .create(true)   // create if not exists
            .append(true)   // append to the end
            .open("/sdcard/log.csv").unwrap();

        // Write new lines at the end
        writeln!(file, "Timestamp,Sensor,Value").unwrap();
        writeln!(file, "171569420,Sound,42").unwrap();
        file.flush().unwrap(); // ensure data is written
        println!("File write successful!");
    }
    // --- Reading ---
    let file = File::open("/sdcard/log.csv").unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();
        println!("{}", line);
    }
    println!("File reading ended.");

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



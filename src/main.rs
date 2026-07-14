#![allow(unknown_lints)]
#![allow(unexpected_cfgs)]

use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::peripherals::Peripherals;
use std::thread;
use std::time::Duration;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

mod clock;
mod dns;
mod sd_card;
mod top;
mod web_server;
mod wifi_access_point;
// mod threads_controller;

// use fake_sensors::*;

const SD_CARD_MOUNT_PATH: &str = "/sdcard";
const SENSOR_DATA_LOG_PATH: &str = "/sdcard/log.csv";
const WIFI_AP_DEFAULT_SSID: &str = "Sensor-Server";
const WIFI_AP_DEFAULT_PASSWORD: &str = "password123";
const DNS_DEFAULT_HOSTNAME: &str = "sensors";
const DNS_DEFAULT_INSTANCE_NAME: &str = "ESP32 Sensors";
const DEFAULT_TIMESTAMP: u64 = 1767268800; // Jan 1, 2026 12:00:00 UTC is 1767268800 seconds since 1970

#[cfg(any(feature = "adc-oneshot-legacy", esp_idf_version_major = "4"))]
fn main() {
    let peripherals = Peripherals::take().unwrap();

    // TODO: get timestamp from last data log entry
    clock::set_time(DEFAULT_TIMESTAMP);

    let mut led = PinDriver::output(peripherals.pins.gpio21).unwrap();
    led_hello(&mut led);

    println!("Initializing Wi-Fi Access Point...");
    let wifi = wifi_access_point::init_ap(
        peripherals.modem,
        WIFI_AP_DEFAULT_SSID,
        WIFI_AP_DEFAULT_PASSWORD,
    );
    println!(
        "Access Point is running! {:?}",
        wifi.get_configuration().unwrap()
    );

    let _dns = dns::initialize_dns(DNS_DEFAULT_HOSTNAME, DNS_DEFAULT_INSTANCE_NAME);

    println!("Initializing Web Server...");
    // start web server (keep Arc to keep server alive)
    let _server = web_server::start_web_server(SENSOR_DATA_LOG_PATH).unwrap();
    println!("Web Server started.");

    // threads_controller::spawn_pinned_task("sd-reader", 4096, 1, || {
    read_sd();
    // });

    // TODO:
    // - emulate sensor data flow
    //        - sound sensor measurment every 100 ms
    //        - temperature every 2000 ms
    //        - light sensor 200 ms
    //        - write highest value for a minute if changed significantly compared to the last entry
    // - develop map-reduce for data before storage
    // - only one thread should read/write SD card, make a queue
    // - display js plot
    // - setup clock
    // - remove magic variables and hardcoded values
    // - add tests
    
    start_sensor_data_aquisition_engine();

    loop {
        println!("Heartbeat. TS: {}", clock::get_current_timestamp() );
        top::print_system_stats();
        thread::sleep(Duration::from_millis(5000));
        //     // println!("Sound. ADC value: {}", adc.read(&mut adc_pin).unwrap());
    }
}



// pub struct SensorsDataEntry {
//     ts: u64,
//     sound: f32,
//     temperature: f32,
//     humidity: f32,
//     light: f32,
// }

// use rand::RngExt;
use fake_sensors::*;

use std::collections::HashMap;
use std::sync::mpsc;
use daq_engine;
use daq_engine::DataEntryTrait;


#[derive(Default, Debug)]
pub struct SensorDataEntry {
    ts: u64,
    attrs: HashMap<String, f32>,
}
daq_engine::impl_daq_data_entry_trait!(SensorDataEntry);

fn build_sensor_reader(name: String, wait_ms: u64, toleration_percentage: f32) -> daq_engine::SensorReader{
    daq_engine::SensorReader {
        name: name,
        wait_ms: wait_ms,
        function: random_float,
        toleration_percentage: toleration_percentage
    }
}

fn random_float() -> f32 {
    fake_sensors::read_sensor(fake_sensors::SensorType::Sound)
}

fn start_sensor_data_aquisition_engine() {
    let last_data_entry = SensorDataEntry {
        ts: 1767268800,
        attrs: HashMap::new()
    };
    let sound_sensor_reader = build_sensor_reader("sound".to_string(), 100, 2.0);
    let temperature_sensor_reader = build_sensor_reader("temperature".to_string(), 200, 0.01);
    let sensor_readers = vec![sound_sensor_reader, temperature_sensor_reader];
    let (storage_tx, storage_rx) = mpsc::channel();

    daq_engine::run(last_data_entry, sensor_readers, storage_tx);
}


fn read_sd() {
    // 1. Mount the physical SD card
    let _card_handle = sd_card::mount_sd_card(SD_CARD_MOUNT_PATH); //.unwrap();

    // 2. Write a file using standard std::io error mapping
    println!("Writing data sample to file...");

    {
        let mut file = OpenOptions::new()
            .create(true) // create if not exists
            .append(true) // append to the end
            .open(SENSOR_DATA_LOG_PATH)
            .unwrap();

        // Write new lines at the end
        writeln!(file, "Timestamp,Sensor,Value").unwrap();
        writeln!(file, "171569420,Sound,42").unwrap();
        file.flush().unwrap(); // ensure data is written
        println!("File write successful!");
    }
    // --- Reading ---
    let file = File::open(SENSOR_DATA_LOG_PATH).unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();
        println!("{}", line);
    }
    println!("File reading ended.");
}

fn led_hello<MODE: esp_idf_hal::gpio::OutputMode>(led: &mut PinDriver<MODE>) {
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

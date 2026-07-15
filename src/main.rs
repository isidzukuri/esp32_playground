#![allow(unknown_lints)]
#![allow(unexpected_cfgs)]

use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::peripherals::Peripherals;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

mod clock;
mod daq_engine_initializer;
mod dns;
mod led;
mod sd_card;
mod storage_initializer;
mod top;
mod web_server;
mod wifi_access_point;
// mod threads_controller;

use storage::StorageController;
use storage::StorageControllerTrait;
use storage::VectorStorageClass;

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
    led::hello(&mut led);

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

    // TODO:
    // - implement read whole fro VectorStorage
    // - implement SdStorage

    let _card_handle = sd_card::mount_sd_card(SD_CARD_MOUNT_PATH); //.unwrap();

    println!("Initializing Storage...");
    let (storage_tx, storage_rx) = mpsc::channel();
    let storage_controller = storage_initializer::run::<VectorStorageClass>(storage_rx);
    println!("Storage initialized.");

    println!("Initializing Data Acquisition Engine...");
    let last_entry_data = storage_controller
        .last_entry()
        .expect("Failed to read last data entry");
    daq_engine_initializer::run(storage_tx, last_entry_data);
    println!("Data Acquisition Engine initialized.");

    println!("Initializing Web Server...");
    // start web server (keep Arc to keep server alive)
    // let _server = web_server::start_web_server(SENSOR_DATA_LOG_PATH).unwrap();
    // let _server = web_server::start_web_server::<VectorStorageClass, StorageController>(Arc::new(Mutex::new(storage_controller))).unwrap();
    let _server = web_server::start_web_server(Arc::new(Mutex::new(storage_controller))).unwrap();
    println!("Web Server started.");

    // threads_controller::spawn_pinned_task("sd-reader", 4096, 1, || {
    // read_sd();
    // });

    // TODO:
    // - develop map-reduce for data before storage
    // - display js plot
    // - setup clock
    // - remove magic variables and hardcoded values
    // - add tests

    loop {
        println!("Heartbeat. TS: {}", clock::get_current_timestamp());
        // top::print_system_stats();
        thread::sleep(Duration::from_millis(15000));
        //     // println!("Sound. ADC value: {}", adc.read(&mut adc_pin).unwrap());
    }
}

// use rand::RngExt;

// fn read_sd() {
//     // 1. Mount the physical SD card
//     let _card_handle = sd_card::mount_sd_card(SD_CARD_MOUNT_PATH); //.unwrap();

//     // 2. Write a file using standard std::io error mapping
//     println!("Writing data sample to file...");

//     {
//         let mut file = OpenOptions::new()
//             .create(true) // create if not exists
//             .append(true) // append to the end
//             .open(SENSOR_DATA_LOG_PATH)
//             .unwrap();

//         // Write new lines at the end
//         writeln!(file, "Timestamp,Sensor,Value").unwrap();
//         writeln!(file, "171569420,Sound,42").unwrap();
//         file.flush().unwrap(); // ensure data is written
//         println!("File write successful!");
//     }
//     // --- Reading ---
//     let file = File::open(SENSOR_DATA_LOG_PATH).unwrap();
//     let reader = BufReader::new(file);

//     for line in reader.lines() {
//         let line = line.unwrap();
//         println!("{}", line);
//     }
//     println!("File reading ended.");
// }

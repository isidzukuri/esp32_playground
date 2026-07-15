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



    // TODO:
    // - build with Storage module
    // - implement read whole fro VectorStorage
    // - implement SdStorage


    let (storage_tx, storage_rx) = mpsc::channel();

    let data_schema = vec!["timestamp".to_string(),
                            "temperature".to_string(),
                            "humidity".to_string(),
                            "light".to_string(),
                            "sound".to_string()];

    let storage_class = VectorStorageClass::default();
    let storage_controller = StorageController::run(data_schema, storage_class, storage_rx).expect("Failed to start StorageController");

    let last_entry_data = storage_controller.last_entry().expect("Failed to read last data entry");
    
    let last_data_entry = SensorDataEntry {
        ts: last_entry_data.0,
        attrs: last_entry_data.1
    };

    start_sensor_data_aquisition_engine(storage_tx, last_data_entry);

    println!("Initializing Web Server...");
    // start web server (keep Arc to keep server alive)
    let _server = web_server::start_web_server(SENSOR_DATA_LOG_PATH).unwrap();
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
        println!("Heartbeat. TS: {}", clock::get_current_timestamp() );
        top::print_system_stats();
        thread::sleep(Duration::from_millis(15000));
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
use storage::*;
// use storage::StorageController;
// use storage::VectorStorageClass;



#[derive(Default, Debug)]
pub struct SensorDataEntry {
    ts: u64,
    attrs: HashMap<String, f32>,
}
daq_engine::impl_daq_data_entry_trait!(SensorDataEntry);

fn read_sound_sensor() -> f32 {
    fake_sensors::read_sensor(fake_sensors::SensorType::Sound)
}

fn read_temperature_sensor() -> f32 {
    fake_sensors::read_sensor(fake_sensors::SensorType::Temperature)
}

fn read_humidity_sensor() -> f32 {
    fake_sensors::read_sensor(fake_sensors::SensorType::Humidity)
}

fn read_light_sensor() -> f32 {
    fake_sensors::read_sensor(fake_sensors::SensorType::Light)
}

fn start_sensor_data_aquisition_engine(storage_tx: mpsc::Sender<(String, (u64, HashMap<String, f32>))>, last_data_entry: SensorDataEntry) {
    let sound_sensor_reader = daq_engine::SensorReader {
        name: "sound".to_string(),
        wait_ms: 50,
        function: read_sound_sensor,
        toleration: 0.05
    };
    let temperature_sensor_reader = daq_engine::SensorReader {
        name: "temperature".to_string(),
        wait_ms: 60000,
        function: read_temperature_sensor,
        toleration: 0.001
    };
    let humidity_sensor_reader = daq_engine::SensorReader {
        name: "humidity".to_string(),
        wait_ms: 60000,
        function: read_humidity_sensor,
        toleration: 0.001
    };
    let light_sensor_reader = daq_engine::SensorReader {
        name: "light".to_string(),
        wait_ms: 500,
        function: read_light_sensor,
        toleration: 0.01
    };
    let sensor_readers = vec![sound_sensor_reader, 
                             temperature_sensor_reader, 
                             humidity_sensor_reader, 
                             light_sensor_reader];

    daq_engine::run(last_data_entry, sensor_readers, storage_tx);
}


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

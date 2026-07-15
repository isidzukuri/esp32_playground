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

pub fn run(storage_tx: mpsc::Sender<(String, (u64, HashMap<String, f32>))>, last_data: (u64, HashMap<String, f32>)) {
    let last_data_entry = SensorDataEntry {
        ts: last_data.0,
        attrs: last_data.1
    };

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

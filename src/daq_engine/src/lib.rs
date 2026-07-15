// DaqEngine (DataAcquisitionEngine)
// TODO: split into files

use std::collections::HashMap;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

mod sensor_reader;
mod data_entry_trait;

pub use crate::sensor_reader::SensorReader;
pub use crate::data_entry_trait::*;

const DEFAULT_LOGGER_WAIT_MS: u64 = 1000;

pub fn run(mut last_data_entry: impl DataEntryTrait + Send + 'static, 
           sensor_readers: Vec<SensorReader>,
           storage_chnl: Sender<(String, (u64, HashMap<String, f32>))>){

    let (tx, rx) = mpsc::channel();

    spawn_sensor_readers(tx, &mut last_data_entry, sensor_readers);
    spawn_data_logger(rx, last_data_entry, storage_chnl);
}

// Saves data to storage only if differ from last entry
fn spawn_data_logger(rx: Receiver<(String, f32)>, mut data_entry: impl DataEntryTrait + Send + 'static, storage_chnl: Sender<(String, (u64, HashMap<String, f32>))>) {
    thread::spawn(move || {
        let mut updated = false;
        loop {
            thread::sleep(Duration::from_millis(DEFAULT_LOGGER_WAIT_MS));
            let mut highest_deviations = HashMap::new();
            
            loop {
                match rx.try_recv() {
                    Ok((sensor_name, new_value)) => {
                        // println!("[DaqEngine] Incoming data: {} = {}", sensor_name, new_value);

                        let baseline_value = data_entry.attrs().get(&sensor_name).copied().unwrap_or(0.0).abs();
                        let current_deviation = (new_value.abs() - baseline_value).abs();
                        let highest_deviation = highest_deviations.entry(sensor_name.clone()).or_insert(0.0);

                        if current_deviation > *highest_deviation {
                            *highest_deviation = current_deviation;
                            // println!("[DaqEngine] New highest deviation for {}: {}", sensor_name, current_deviation);
                            
                            highest_deviations
                                .entry(sensor_name.clone())
                                .and_modify(|deviation| *deviation = current_deviation);

                            data_entry
                                .attrs()
                                .entry(sensor_name.clone())
                                .and_modify(|val| *val = new_value)
                                .or_insert(new_value);

                            data_entry.increment_ts(); 

                            updated = true;
                        }
                    },
                    Err(TryRecvError::Empty) => { 
                        if updated {
                            let serialized_entry = data_entry.for_storage_channel();
                            println!("[DaqEngine] Saving data to the storage: {:?}", &serialized_entry);
                            let _ = storage_chnl.send(("save".to_string(), serialized_entry));
                            updated = false;
                        }
                        println!("[DaqEngine] Waiting for new data");
                        break;
                    },
                    Err(TryRecvError::Disconnected) => { panic!("[DaqEngine] Error: all Senders have been dropped!"); }
                }
            }
        }
    });
}

fn spawn_sensor_readers(tx: Sender<(String, f32)>, last_data_entry: &mut impl DataEntryTrait, sensor_readers: Vec<SensorReader>) {
    for item in sensor_readers.iter() {
        let thread_tx = tx.clone();
        let reader = item.clone();
        let mut current_value = last_data_entry.attrs().get(&reader.name).cloned().unwrap_or_default();

        thread::spawn(move || {
            loop {
                let new_value = (reader.function)(); 
                // println!("[DaqEngine][SENSOR] {}: {}", reader.name, new_value);
                if is_deviation_significant(current_value, new_value, reader.toleration) {
                    current_value = new_value;
                    thread_tx.send((reader.name.clone(), new_value)).unwrap();
                }
                // println!("[DaqEngine][SENSOR] waiting {}", reader.wait_ms);
                thread::sleep(Duration::from_millis(reader.wait_ms));
            }
        });
    }
}

// if diff is > tolerated %
fn is_deviation_significant(current: f32, new: f32, toleration: f32) -> bool {
    if current == 0.0 {
        return new != 0.0;
    }
    
    ((new - current).abs() / current.abs()) > toleration
}


#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngExt;

    const TIMESTAMP: u64 = 1767268800;
    
    #[derive(Default, Debug)]
    pub struct TestDataEntry {
        ts: u64,
        attrs: HashMap<String, f32>,
    }
    impl_daq_data_entry_trait!(TestDataEntry);

    fn build_data_entry(ts: Option<u64>, attrs: Option<HashMap<String, f32>>) -> TestDataEntry {
        TestDataEntry {
            ts: ts.unwrap_or(TIMESTAMP),
            attrs: attrs.unwrap_or(HashMap::new())
        }
    }

    fn build_sensor_reader(name: String, wait_ms: u64, toleration: f32) -> SensorReader{
        SensorReader {
            name: name,
            wait_ms: wait_ms,
            function: random_float,
            toleration: toleration
        }
    }

    fn random_float() -> f32 {
        rand::random_range(0.0..90.0)
    }

    #[test]
    fn test_spawn_sensor_readers(){
        let (tx, rx) = mpsc::channel();
        let mut last_data_entry = build_data_entry(None, None);
        let test_sender = build_sensor_reader("test_name".to_string(), 5, 0.0);
        let sensor_readers = vec![test_sender];

        spawn_sensor_readers(tx, &mut last_data_entry, sensor_readers);

        thread::sleep(Duration::from_millis(1));
        let package = rx.try_recv();
        assert!(package.is_ok());
        let message = package.unwrap();
        assert_eq!(message.0, "test_name".to_string());
        assert!(message.1 > 0.0);
        assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
    }
    
    // TODO: fix potentially flaky test
    #[test]
    fn test_spawn_sensor_readers_multiple_sends(){
        let (tx, rx) = mpsc::channel();
        let mut last_data_entry = build_data_entry(None, None);
        let test_sender = build_sensor_reader("test_name".to_string(), 8, -1.0);
        let sensor_readers = vec![test_sender];

        spawn_sensor_readers(tx, &mut last_data_entry, sensor_readers);

        thread::sleep(Duration::from_millis(11));
        let package = rx.try_recv();
        assert!(package.is_ok());
        let message = package.unwrap();
        assert_eq!(message.0, "test_name".to_string());
        assert!(message.1 > 0.0);
        assert!(rx.try_recv().is_ok());
        // assert!(rx.try_recv().is_ok());
        assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
    }

    #[test]
    fn test_spawn_sensor_readers_when_data_has_attr(){
        let (tx, rx) = mpsc::channel();
        let mut attrs = HashMap::new();
        attrs.insert("test_name".to_string(), 0.0);
        let mut last_data_entry = build_data_entry(None, Some(attrs));
        let test_sender = build_sensor_reader("test_name".to_string(), 5, -1.0);
        let sensor_readers = vec![test_sender];

        spawn_sensor_readers(tx, &mut last_data_entry, sensor_readers);

        thread::sleep(Duration::from_millis(1));
        let package = rx.try_recv();
        assert!(package.is_ok());
        let message = package.unwrap();
        assert_eq!(message.0, "test_name".to_string());
        assert!(message.1 > 0.0);
        assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
    }

    #[test]
    fn test_spawn_data_logger(){
        let (tx, rx) = mpsc::channel();
        let (storage_tx, storage_rx) = mpsc::channel();
        let last_data_entry = build_data_entry(None, None);

        tx.send(("temperature".to_string(), 23.0)).unwrap();

        spawn_data_logger(rx, last_data_entry, storage_tx);

        thread::sleep(Duration::from_millis(2010));

        let package = storage_rx.try_recv();
        let message = package.unwrap();

        assert_eq!(message.0, "save".to_string());
        assert_eq!(message.1.0, 1767268801);
        assert_eq!(*message.1.1.get("temperature").unwrap(), 23.0);
        assert!(message.1.1.get("not_existing").is_none());
        assert_eq!(storage_rx.try_recv(), Err(TryRecvError::Empty));
    }

    #[test]
    fn test_spawn_data_logger_when_data_exists(){
        let (tx, rx) = mpsc::channel();
        let (storage_tx, storage_rx) = mpsc::channel();
        let last_data_entry = build_data_entry(None, Some(HashMap::from([("temperature".to_string(), 1.2)])));

        tx.send(("temperature".to_string(), 23.0)).unwrap();

        spawn_data_logger(rx, last_data_entry, storage_tx);

        thread::sleep(Duration::from_millis(1010));

        let package = storage_rx.try_recv();
        let message = package.unwrap();

        assert_eq!(message.0, "save".to_string());
        assert_eq!(message.1.0, 1767268801);
        assert_eq!(*message.1.1.get("temperature").unwrap(), 23.0);
        assert_eq!(storage_rx.try_recv(), Err(TryRecvError::Empty));
    }

    #[test]
    fn test_spawn_data_logger_when_data_measurment_is_not_changed(){
        let (tx, rx) = mpsc::channel();
        let (storage_tx, storage_rx) = mpsc::channel();
        let last_data_entry = build_data_entry(None, Some(HashMap::from([("temperature".to_string(), 1.2)])));

        tx.send(("temperature".to_string(), 1.2)).unwrap();

        spawn_data_logger(rx, last_data_entry, storage_tx);

        thread::sleep(Duration::from_millis(1010));

        assert_eq!(storage_rx.try_recv(), Err(TryRecvError::Empty));
    }

    #[test]
    fn test_run() {
        let mut attrs = HashMap::new();
        attrs.insert("sound".to_string(), 0.0);
        attrs.insert("temperature".to_string(), 0.0);
        let last_data_entry = build_data_entry(None, Some(attrs));
        let sound_sensor_reader = build_sensor_reader("sound".to_string(), 100, 2.0);
        let temperature_sensor_reader = build_sensor_reader("temperature".to_string(), 200, 0.01);
        let sensor_readers = vec![sound_sensor_reader, temperature_sensor_reader];
        let (storage_tx, storage_rx) = mpsc::channel();

        run(last_data_entry, sensor_readers, storage_tx);

        thread::sleep(Duration::from_millis(1010));

        let package = storage_rx.try_recv();
        let message = package.unwrap();

        assert_eq!(message.0, "save".to_string());
        assert!(message.1.0 > 1767268800);
        assert!(*message.1.1.get("temperature").unwrap() != 0.0);
        assert!(*message.1.1.get("sound").unwrap() != 0.0);
        assert!(message.1.1.get("not_existing").is_none());
        assert_eq!(storage_rx.try_recv(), Err(TryRecvError::Empty));

        thread::sleep(Duration::from_millis(1010));

        assert!(storage_rx.try_recv().is_ok());
        assert_eq!(storage_rx.try_recv(), Err(TryRecvError::Empty));
    }

    #[test]
    fn test_is_deviation_significant_no_deviation() {
        // No change should mean no significant deviation
        assert!(!is_deviation_significant(100.0, 100.0, 0.1));
    }

    #[test]
    fn test_is_deviation_significant_within_tolerance() {
        // A 5% increase with a 10% tolerance should be false
        assert!(!is_deviation_significant(100.0, 105.0, 0.10));
        // A 5% decrease with a 10% tolerance should be false
        assert!(!is_deviation_significant(100.0, 95.0, 0.10));
    }

    #[test]
    fn test_is_deviation_significant_exceeds_tolerance() {
        // A 15% increase with a 10% tolerance should be true
        assert!(is_deviation_significant(100.0, 115.0, 0.10));
        // A 15% decrease with a 10% tolerance should be true
        assert!(is_deviation_significant(100.0, 85.0, 0.10));
    }

    #[test]
    fn test_is_deviation_significant_exactly_at_tolerance() {
        // Depending on strict inequality (>), exactly 10% deviation 
        // with 10% tolerance should evaluate to false.
        assert!(!is_deviation_significant(100.0, 110.0, 0.10));
    }

    #[test]
    fn test_is_deviation_significant_current_is_zero() {
        // If current is 0 and new is 0, deviation is not significant
        assert!(!is_deviation_significant(0.0, 0.0, 0.1));
        // If current is 0 and new changes to anything else, it is significant
        assert!(is_deviation_significant(0.0, 1.0, 0.1));
        assert!(is_deviation_significant(0.0, -1.0, 0.1));
    }

    #[test]
    fn test_is_deviation_significant_negative_values() {
        // Going from -100 to -115 is a 15% deviation, which exceeds 10% tolerance
        assert!(is_deviation_significant(-100.0, -115.0, 0.10));
        // Going from -100 to -105 is a 5% deviation, within 10% tolerance
        assert!(!is_deviation_significant(-100.0, -105.0, 0.10));
    }
}

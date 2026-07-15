use crate::storage_class_trait::StorageClassTrait;
use crate::storage_controller_trait::StorageControllerTrait;
use crate::storage_error::StorageError;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct StorageController<SC: StorageClassTrait> {
    pub storage_mutex: Arc<Mutex<SC>>,
    pub data_schema: Vec<String>,
}

impl<SC: StorageClassTrait + Send + 'static> StorageControllerTrait<SC> for StorageController<SC> {
    fn run(
        data_schema: Vec<String>,
        storage_class: SC,
        receiver: Receiver<(String, (u64, HashMap<String, f32>))>,
    ) -> Result<Self, StorageError> {
        let instance = Self {
            storage_mutex: Arc::new(Mutex::new(storage_class)),
            data_schema,
        };
        instance.update_headers()?;
        instance.start_listening(receiver);
        Ok(instance)
    }

    fn update_headers(&self) -> Result<(), StorageError> {
        let mut storage = self.storage_mutex.lock()?;
        let headers_line = self.data_schema.join(",");

        if storage.lines_len()? > 0 {
            let current_headers_line = storage.read_line(0)?;
            if current_headers_line == headers_line {
                return Ok(());
            }
            storage.replace_line(headers_line, 0)
        } else {
            storage.append_line(headers_line)
        }
    }

    fn start_listening(&self, receiver: Receiver<(String, (u64, HashMap<String, f32>))>) {
        let storage_mutex = Arc::clone(&self.storage_mutex);
        let data_schema = self.data_schema.clone();
        thread::spawn(move || {
            for message in receiver {
                // println!("[StorageController] Worker woke up! Processing: {:?}", message);
                if let Err(e) = Self::process_message(message, &storage_mutex, &data_schema) {
                    panic!("Error processing message in background thread: {}", e);
                }
            }
        });
    }

    fn process_message(
        message: (String, (u64, HashMap<String, f32>)),
        storage_mutex: &Arc<Mutex<SC>>,
        data_schema: &Vec<String>,
    ) -> Result<(), StorageError> {
        match message {
            (ref msg_type, (timestamp, payload)) if msg_type == "save" => {
                let mut storage = storage_mutex.lock()?;
                let formatted_payload =
                    Self::serialize_payload_for_storage(timestamp, payload, data_schema)?;
                storage.append_line(formatted_payload)?;
            }
            (msg_type, _) => {
                return Err(StorageError::NotImplemented {
                    method_name: msg_type,
                    entity: "StorageController".to_string(),
                });
            }
        }
        Ok(())
    }

    fn serialize_payload_for_storage(
        timestamp: u64,
        payload: HashMap<String, f32>,
        data_schema: &Vec<String>,
    ) -> Result<String, StorageError> {
        let mut ordered_items = vec![timestamp.to_string()];
        for field in data_schema.iter() {
            if field == "timestamp" {
                continue;
            };
            match payload.get(field) {
                Some(val) => ordered_items.push(val.to_string()),
                None => ordered_items.push("0.0".to_string()),
            }
        }
        Ok(ordered_items.join(","))
    }

    fn last_entry(&self) -> Result<(u64, HashMap<String, f32>), StorageError> {
        let mut timestamp = Self::current_timestamp();
        let storage = self.storage_mutex.lock()?;
        let mut attrs = HashMap::new();
        let lines = storage.lines_len()?;

        let mut parts: Vec<String> = vec![];
        if lines > 1 {
            let line = storage.read_line(lines - 1)?;
            parts = line.split(',').map(String::from).collect();
        }

        for (idx, field_name) in self.data_schema.iter().enumerate() {
            if field_name == "timestamp" {
                if let Some(ts_str) = parts.get(idx) {
                    timestamp = ts_str.parse::<u64>()?
                }
            } else {
                let default_val = "0.0".to_string();
                let val_str = parts.get(idx).unwrap_or(&default_val);
                let val = val_str.parse::<f32>()?;
                attrs.insert(field_name.clone(), val);
            }
        }

        Ok((timestamp, attrs))
    }

    fn read_whole_storage<F>(&self, mut closure: F) -> Result<(), StorageError>
    where
        F: FnMut(&mut dyn std::io::Read)
    {
        let storage = self.storage_mutex.lock()?;
        closure(&mut storage.reader());
        Ok(())
    }

    fn current_timestamp() -> u64 {
        let now = SystemTime::now();
        let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
        since_the_epoch.as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage_classes::vector_storage_class::VectorStorageClass;
    use std::sync::mpsc;
    use std::time::Duration;

    const TIMESTAMP: u64 = 1767268800;

    #[test]
    fn test_saving_incoming_channel_messages() {
        let storage_class = VectorStorageClass::default();
        let (storage_tx, storage_rx) = mpsc::channel();
        let data_schema = vec![
            "timestamp".to_string(),
            "test".to_string(),
            "second".to_string(),
        ];
        let storage_controller = StorageController::run(data_schema, storage_class, storage_rx).unwrap();
        let test_message = (
            "save".to_string(),
            (TIMESTAMP, HashMap::from([("test".to_string(), 1.2)])),
        );
        let _ = storage_tx.send(test_message.clone());
        let test_message = (
            "save".to_string(),
            (TIMESTAMP, HashMap::from([("second".to_string(), 2.8)])),
        );
        let _ = storage_tx.send(test_message.clone());

        let test_message = (
            "save".to_string(),
            (
                TIMESTAMP,
                HashMap::from([("second".to_string(), 2.8), ("test".to_string(), 1.2)]),
            ),
        );
        let _ = storage_tx.send(test_message.clone());

        thread::sleep(Duration::from_millis(200));

        let storage_class = storage_controller.storage_mutex.lock().unwrap();
        assert_eq!(storage_class.storage.len(), 4);
        assert_eq!(
            storage_class.storage[0],
            "timestamp,test,second".to_string()
        );
        assert_eq!(storage_class.storage[1], "1767268800,1.2,0.0".to_string());
        assert_eq!(storage_class.storage[2], "1767268800,0.0,2.8".to_string());
        assert_eq!(storage_class.storage[3], "1767268800,1.2,2.8".to_string());
    }

    #[test]
    fn test_last_entry() {
        let storage_class = VectorStorageClass::default();
        let (storage_tx, storage_rx) = mpsc::channel();
        let data_schema = vec![
            "timestamp".to_string(),
            "test".to_string(),
            "second".to_string(),
        ];
        let storage_controller = StorageController::run(data_schema, storage_class, storage_rx).unwrap();
        let test_message = (
            "save".to_string(),
            (TIMESTAMP, HashMap::from([("test".to_string(), 1.2)])),
        );
        let _ = storage_tx.send(test_message.clone());

        let test_message = (
            "save".to_string(),
            (
                TIMESTAMP,
                HashMap::from([("second".to_string(), 2.8), ("test".to_string(), 1.2)]),
            ),
        );
        let _ = storage_tx.send(test_message.clone());

        thread::sleep(Duration::from_millis(20));

        let last_entry = storage_controller.last_entry().unwrap();

        assert!(last_entry.0 > 1767268000);
        assert_eq!(last_entry.1["test"], 1.2);
        assert_eq!(last_entry.1["second"], 2.8);
    }

    #[test]
    fn test_format_payload_success_all_fields_present() {
        let timestamp = 1711111111;
        let mut payload = HashMap::new();
        payload.insert("temp".to_string(), 23.5);
        payload.insert("humidity".to_string(), 60.0);
        let data_schema = vec!["temp".to_string(), "humidity".to_string()];
        let result = StorageController::<VectorStorageClass>::serialize_payload_for_storage(
            timestamp,
            payload,
            &data_schema,
        );
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert_eq!(formatted, "1711111111,23.5,60");
    }

    #[test]
    fn test_format_payload_missing_fields_uses_fallback() {
        let timestamp = 1711111111;
        let mut payload = HashMap::new();
        payload.insert("temp".to_string(), 18.2);
        let data_schema = vec!["temp".to_string(), "humidity".to_string()];
        let result = StorageController::<VectorStorageClass>::serialize_payload_for_storage(
            timestamp,
            payload,
            &data_schema,
        );
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert_eq!(formatted, "1711111111,18.2,0.0");
    }

    #[test]
    fn test_format_payload_empty_schema() {
        let timestamp = 1711111111;
        let payload = HashMap::new();
        let data_schema = vec![]; // Schema is empty
        let result = StorageController::<VectorStorageClass>::serialize_payload_for_storage(
            timestamp,
            payload,
            &data_schema,
        );
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert_eq!(formatted, "1711111111");
    }

    #[test]
    fn test_format_payload_ignores_extra_payload_fields() {
        let timestamp = 1711111111;
        let mut payload = HashMap::new();
        payload.insert("temp".to_string(), 23.5);
        payload.insert("untracked_field".to_string(), 99.9);

        let data_schema = vec!["temp".to_string()];
        let result = StorageController::<VectorStorageClass>::serialize_payload_for_storage(
            timestamp,
            payload,
            &data_schema,
        );
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert_eq!(formatted, "1711111111,23.5");
    }
}

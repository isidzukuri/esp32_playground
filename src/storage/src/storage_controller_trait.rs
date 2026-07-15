use crate::storage_class_trait::StorageClassTrait;
use crate::storage_error::StorageError;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
// use std::io::Read;
// use esp_idf_svc::http::server;

pub trait StorageControllerTrait<SC: StorageClassTrait> {
    fn run(
        data_schema: Vec<String>,
        storage_class: SC,
        receiver: Receiver<(String, (u64, HashMap<String, f32>))>,
    ) -> Result<Self, StorageError> where Self: Sized;
    fn update_headers(&self) -> Result<(), StorageError>;
    fn start_listening(&self, receiver: Receiver<(String, (u64, HashMap<String, f32>))>);
    fn process_message(
        message: (String, (u64, HashMap<String, f32>)),
        storage_mutex: &Arc<Mutex<SC>>,
        data_schema: &Vec<String>,
    ) -> Result<(), StorageError>;
    fn serialize_payload_for_storage(
        timestamp: u64,
        payload: HashMap<String, f32>,
        data_schema: &Vec<String>,
    ) -> Result<String, StorageError>;
    fn last_entry(&self) -> Result<(u64, HashMap<String, f32>), StorageError>;
    fn current_timestamp() -> u64;
    fn read_whole_storage<F>(&self, closure: F)
    where
        F: FnMut(&mut dyn std::io::Read);
    // fn purge(&self) -> Result<(), StorageError>;
}

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

    // fn read_whole_storage(&self, reader: fn(path: &'static str) -> ()) -> Result<(), StorageError>;
    fn read_whole_storage<F>(&self, closure: F)
    where
        F: FnMut(&mut dyn std::io::Read);


    // fn read_whole_storage(&self, reader: &mut dyn Read) -> Result<(), StorageError>;
    // fn read_whole_storage<F>(&self, f: F) -> Result<(), StorageError>
    //     where
    //         F: FnOnce(&mut dyn Read) -> ();

    // fn read_whole_storage<F>(&self, reader: F) -> Result<(), StorageError>
    //     where
    //         F: FnOnce(&mut R, &mut esp_idf_svc::http::server::Response<C>) -> (),
    //         R: std::io::Read,
    //         C: esp_idf_svc::http::server::Connection;

    // fn read_whole_storage<F, C>(&self, reader: F) -> Result<(), StorageError>
    //     where
    //         // We use "impl std::io::Read" or a concrete type if possible
    //         F: for<'a, 'b> FnOnce(&'a mut dyn std::io::Read, &'b mut super::esp_idf_svc::http::server::Response<C>),
    //         C: super::esp_idf_svc::http::server::Connection;
    
    // fn read_whole_storage<F, R, C, RES>(&self, reader: F) -> Result<(), StorageError>
    // where
    //     F: for<'a, 'b> FnOnce(&'a mut R, &'b mut RES),
    //     R: std::io::Read,
    //     RES: YourGenericResponseTrait<Connection = C>
    
    
    // fn stream_to_response<R, C>(
    //     mut reader: R, 
    //     resp: &mut esp_idf_svc::http::server::Response<C>
    // )
    // where 
    //     R: std::io::Read,
    //     C: esp_idf_svc::http::server::Connection;
    
    
    // fn purge(&self) -> Result<(), StorageError>;
}

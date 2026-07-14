// mod storage_class_trait;
// mod storage_classes;
// mod storage_error;

// use crate::storage_classes::vector_storage_class::VectorStorageClass;

// use crate::storage_error::StorageError;


// pub trait StorageControllerTrait {
//     fn init(storage_class: impl StorageClassTrait,receiver: Receiver<(String, (u64, HashMap<String, f32>))>) -> Self;
// }



// use std::sync::mpsc::Receiver;
// use std::sync::mpsc::TryRecvError;
// use std::sync::mpsc;
// use std::collections::HashMap;

// use crate::storage_class_trait::StorageClassTrait;

// pub struct StorageController<SC: StorageClassTrait> {
//     storage_class: SC,
//     receiver: Receiver<(String, (u64, HashMap<String, f32>))>
// }



// impl<SC: StorageClassTrait> StorageControllerTrait for StorageController<SC> {
//     fn init(storage_class: impl StorageClassTrait, receiver: Receiver<(String, (u64, HashMap<String, f32>))>) -> Self {
//         Self{
//             storage_class: storage_class,
//             receiver: receiver   
//         }
//     }
// }



// it maintains data schema
// it accepts incoming data entries in given on init channel
// it reads whole storage on request
// it wraps StorageClass into Mutex




mod storage_class_trait;
mod storage_classes;
mod storage_error;

use std::sync::{Arc, Mutex};
use std::thread;

use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use crate::storage_error::StorageError;
use crate::storage_class_trait::StorageClassTrait;

pub trait StorageControllerTrait<SC: StorageClassTrait> {
    fn new(storage_class: SC, receiver: Receiver<(String, (u64, HashMap<String, f32>))>) -> Self;
    fn start_listening(&self, receiver: Receiver<(String, (u64, HashMap<String, f32>))>);
    // fn last_entry(&self) -> Result<DataEntry, StorageError>;
    fn read_whole_storage(&self, reader: fn(path: &'static str) -> ()) -> Result<(), StorageError>;
// //     fn purge(&self) -> Result<(), StorageError>;
}

pub struct StorageController<SC: StorageClassTrait> {
    pub storage_mutex: Arc<Mutex<SC>>,
}

impl<SC: StorageClassTrait + Send + 'static> StorageControllerTrait<SC> for StorageController<SC> {
    fn new(storage_class: SC, receiver: Receiver<(String, (u64, HashMap<String, f32>))>) -> Self {
        let instance = Self {
            storage_mutex: Arc::new(Mutex::new(storage_class)),
        };
        instance.start_listening(receiver);
        instance
    }

    fn start_listening(&self, receiver: Receiver<(String, (u64, HashMap<String, f32>))>){
        let storage_mutex = Arc::clone(&self.storage_mutex);
        thread::spawn(move || {
            for message in receiver { // After this block, the thread goes back to sleep waiting for the next message
                println!("Worker woke up! Processing: {:?}", message);

                // if `save` message
                    // check_schema
                    // update_schema if new attr in message
                    // format message`s payload
                    // persist formated payload
                {
                    match storage_mutex.lock() {
                        Ok(mut storage) => {
                            storage.append_line("test, incoming emulation".to_string());
                        }
                        Err(poisoned) => { 
                            panic!("Failed to lock storage because the mutex is poisoned: {:?}", poisoned);
                        }
                    }
                }
            }
        });
    }

    fn read_whole_storage(&self, reader: fn(path: &'static str) -> ()) -> Result<(), StorageError> {
        let mut storage = self.storage_mutex.lock()?;
        storage.exec_file_reader(reader)
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
    fn test_init() {
        let mut storage_class = VectorStorageClass::default();
        let (storage_tx, storage_rx) = mpsc::channel();

        let storage_controller = StorageController::new(storage_class, storage_rx);
    

        let test_message = ("save".to_string(),
                            (   TIMESTAMP, 
                                HashMap::from([("test".to_string(), 1.2)])
                            )
                            );
        storage_tx.send(test_message.clone());
        storage_tx.send(test_message.clone());
        storage_tx.send(test_message.clone());

        thread::sleep(Duration::from_millis(200));

        let storage = storage_controller.storage_mutex.lock().unwrap();
        dbg!(storage);
    }
}



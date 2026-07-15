use std::collections::HashMap;
use std::sync::mpsc;
use storage::*;

pub fn run(
    storage_rx: mpsc::Receiver<(String, (u64, HashMap<String, f32>))>,
) -> impl StorageControllerTrait<VectorStorageClass> {
    let data_schema = vec![
        "timestamp".to_string(),
        "temperature".to_string(),
        "humidity".to_string(),
        "light".to_string(),
        "sound".to_string(),
    ];

    let storage_class = VectorStorageClass::default();
    StorageController::run(data_schema, storage_class, storage_rx)
        .expect("Failed to start StorageController")
}
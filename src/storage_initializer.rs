use std::collections::HashMap;
use std::sync::mpsc;
use storage::*;

pub fn run<StorageClass>(
    storage_rx: mpsc::Receiver<(String, (u64, HashMap<String, f32>))>,
) -> impl StorageControllerTrait<StorageClass>
where
    StorageClass: StorageClassTrait + Default + Send + 'static,
{
    let data_schema = vec![
        "timestamp".to_string(),
        "temperature".to_string(),
        "humidity".to_string(),
        "light".to_string(),
        "sound".to_string(),
    ];

    let storage_class = StorageClass::default();

    StorageController::run(data_schema, storage_class, storage_rx)
        .expect("Failed to start StorageController")
}

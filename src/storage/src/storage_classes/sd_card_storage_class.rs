use crate::storage_class_trait::StorageClassTrait;
use crate::storage_error::StorageError;
use crate::vector_string_reader::VecStringReader;
use std::collections::HashMap;
use std::path::Path;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Write};

// TODO: handle errors. Do not use unwrap and expect

#[derive(Default, Debug)]
pub struct SdCardStorageClass {
    pub storage_file_path: String,
    pub options: HashMap<String, String>,
}

impl SdCardStorageClass {
    fn maintain_file(file_path: &String) {
        println!("[SdCardStorageClass] checking storage file");

        let file_path = Path::new(file_path);
        if let Some(parent) = file_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("Failed to create directory: {}", e);
                return;
            }
        }

        let _file = OpenOptions::new()
            .create(true) 
            .write(true) 
            .open(file_path)
            .expect("[SdCardStorageClass] Failed to open or create file");

            println!("[SdCardStorageClass] checking storage file: successful");
    }
}

impl StorageClassTrait for SdCardStorageClass {

    fn new(options: HashMap<String, String>) -> Self {
        let storage_file_path = options.get("path_to_storage_file").cloned().expect("[SdCardStorageClass] Error: option path_to_storage_file is not set.");
        Self::maintain_file(&storage_file_path);
        Self { options, storage_file_path: storage_file_path.to_string(), ..Default::default()}
    }

    fn append_line(&mut self, line: String) -> Result<(), StorageError> {
        let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&self.storage_file_path).unwrap();

        writeln!(file, "{}", line).unwrap();

        Ok(())
    }

    fn replace_line(&mut self, line: String, line_number: usize) -> Result<(), StorageError> {
        let path = Path::new(&self.storage_file_path);
        let temp_path = path.with_extension("tmp");

        let input = File::open(path).unwrap();
        let reader = BufReader::new(input);

        let output = File::create(&temp_path).unwrap();
        let mut writer = BufWriter::new(output);

        let mut current_line = 0;

        for ln in reader.lines() {
            let ln = ln.unwrap();
            if current_line == line_number {
                writeln!(writer, "{}", line).unwrap();
            } else {
                writeln!(writer, "{}", ln).unwrap();
            }
            current_line += 1;
        }

        writer.flush().unwrap();
        std::fs::rename(temp_path, path).unwrap();

        Ok(())
    }

    fn read_line(&self, line_number: usize) -> Result<String, StorageError> {
        let len = self.lines_len().unwrap();
        if len <= line_number { 
            return Err(StorageError::OutOfBounds {
                    requested: line_number,
                    len: len,
            });
        }

        let file = File::open(&self.storage_file_path).unwrap();
        let reader = BufReader::new(file);
        
        for (index, line_result) in reader.lines().enumerate() {
            if index == line_number {
                let line = line_result.unwrap();
                return Ok(line);
            }
        }
        Ok("".to_string())
    }

    fn lines_len(&self) -> Result<usize, StorageError> {
        let file = File::open(&self.storage_file_path).unwrap();
        let mut reader = BufReader::new(file);
        let mut count = 0;
        let mut line_buffer = String::new();
        while reader.read_line(&mut line_buffer).unwrap() > 0 {
            count += 1;
            line_buffer.clear();
        }
        Ok(count)
    }

    fn purge(&mut self) -> Result<(), StorageError> {
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true) 
            .open(&self.storage_file_path).unwrap();
        Ok(())
    }

    fn reader(&self) -> impl std::io::Read {
        File::open(&self.storage_file_path).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::env;
    use std::fs;
    use std::panic;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // Atomic counter to ensure unique folder names if tests run in parallel
    static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

    // A helper function to set up and clean up the temp directory
    fn run_in_temp_dir<F>(test_body: F)
    where
        F: FnOnce(&PathBuf) + panic::UnwindSafe,
    {
        let sys_temp = env::temp_dir();
        let thread_id = std::thread::current().id();
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_subdir = sys_temp.join(format!("rust_test_{:?}_{}", thread_id, counter));

        fs::create_dir_all(&temp_subdir).expect("Failed to create temp dir");

        let result = panic::catch_unwind(|| {
            test_body(&temp_subdir);
        });

        let _ = fs::remove_dir_all(&temp_subdir);

        if let Err(err) = result {
            panic::resume_unwind(err);
        }
    }

    #[test]
    fn test_new_creates_file_and_directories() {
        run_in_temp_dir(|temp_path| {
            // Test that it can create nested parent directories automatically
            let nested_file_path = temp_path.join("nested_dir").join("storage.log");
            let file_name = nested_file_path.to_str().unwrap().to_string();
            
            let mut options = HashMap::new();
            options.insert("path_to_storage_file".to_string(), file_name.clone());

            let instance = SdCardStorageClass::new(options);

            assert_eq!(instance.storage_file_path, file_name);
            assert!(nested_file_path.exists());
            assert!(nested_file_path.is_file());
        });
    }

    #[test]
    fn test_append_line_and_lines_len() {
        run_in_temp_dir(|temp_path| {
            let file_name = temp_path.join("append_test.log").to_str().unwrap().to_string();
            let mut options = HashMap::new();
            options.insert("path_to_storage_file".to_string(), file_name);

            let mut storage = SdCardStorageClass::new(options);

            // Verify initial state is empty
            assert_eq!(storage.lines_len().unwrap(), 0);

            // Append first line
            storage.append_line("First Line".to_string()).unwrap();
            assert_eq!(storage.lines_len().unwrap(), 1);

            // Append second line
            storage.append_line("Second Line".to_string()).unwrap();
            assert_eq!(storage.lines_len().unwrap(), 2);
        });
    }

    #[test]
    fn test_read_line_success() {
        run_in_temp_dir(|temp_path| {
            let file_name = temp_path.join("read_test.log").to_str().unwrap().to_string();
            let mut options = HashMap::new();
            options.insert("path_to_storage_file".to_string(), file_name);

            let mut storage = SdCardStorageClass::new(options);
            storage.append_line("Line 0".to_string()).unwrap();
            storage.append_line("Line 1".to_string()).unwrap();
            storage.append_line("Line 2".to_string()).unwrap();

            assert_eq!(storage.read_line(0).unwrap(), "Line 0");
            assert_eq!(storage.read_line(1).unwrap(), "Line 1");
            assert_eq!(storage.read_line(2).unwrap(), "Line 2");
        });
    }

    #[test]
    fn test_read_line_out_of_bounds() {
        run_in_temp_dir(|temp_path| {
            let file_name = temp_path.join("bounds_test.log").to_str().unwrap().to_string();
            let mut options = HashMap::new();
            options.insert("path_to_storage_file".to_string(), file_name);

            let mut storage = SdCardStorageClass::new(options);
            storage.append_line("Only Line".to_string()).unwrap();

            // Index 0 is valid, Index 1 should be out of bounds
            let result = storage.read_line(1);
            assert!(result.is_err());
            
            if let Err(StorageError::OutOfBounds { requested, len }) = result {
                assert_eq!(requested, 1);
                assert_eq!(len, 1);
            } else {
                panic!("Expected StorageError::OutOfBounds");
            }
        });
    }

    #[test]
    fn test_replace_line() {
        run_in_temp_dir(|temp_path| {
            let file_name = temp_path.join("replace_test.log").to_str().unwrap().to_string();
            let mut options = HashMap::new();
            options.insert("path_to_storage_file".to_string(), file_name);

            let mut storage = SdCardStorageClass::new(options);
            storage.append_line("Apple".to_string()).unwrap();
            storage.append_line("Banana".to_string()).unwrap();
            storage.append_line("Cherry".to_string()).unwrap();

            // Replace "Banana" (index 1) with "Blueberry"
            storage.replace_line("Blueberry".to_string(), 1).unwrap();

            assert_eq!(storage.read_line(0).unwrap(), "Apple");
            assert_eq!(storage.read_line(1).unwrap(), "Blueberry");
            assert_eq!(storage.read_line(2).unwrap(), "Cherry");
            assert_eq!(storage.lines_len().unwrap(), 3);
        });
    }

    #[test]
    fn test_purge() {
        run_in_temp_dir(|temp_path| {
            let file_name = temp_path.join("purge_test.log").to_str().unwrap().to_string();
            let mut options = HashMap::new();
            options.insert("path_to_storage_file".to_string(), file_name);

            let mut storage = SdCardStorageClass::new(options);
            storage.append_line("Data 1".to_string()).unwrap();
            storage.append_line("Data 2".to_string()).unwrap();

            assert_eq!(storage.lines_len().unwrap(), 2);

            // Truncate/purge storage
            storage.purge().unwrap();

            assert_eq!(storage.lines_len().unwrap(), 0);
        });
    }

    #[test]
    fn test_reader() {
        run_in_temp_dir(|temp_path| {
            let file_name = temp_path.join("reader_test.log").to_str().unwrap().to_string();
            let mut options = HashMap::new();
            options.insert("path_to_storage_file".to_string(), file_name);

            let mut storage = SdCardStorageClass::new(options);
            storage.append_line("Hello World!".to_string()).unwrap();

            let mut reader = storage.reader();
            let mut content = String::new();
            reader.read_to_string(&mut content).unwrap();

            // Because append_line uses writeln!, it adds a newline character
            assert_eq!(content, "Hello World!\n");
        });
    }
}

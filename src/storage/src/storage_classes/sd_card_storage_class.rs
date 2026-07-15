use crate::storage_class_trait::StorageClassTrait;
use crate::storage_error::StorageError;
use crate::vector_string_reader::VecStringReader;
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct SdCardStorageClass {
    pub storage: Vec<String>,
    pub options: HashMap<String, String>,
}

impl StorageClassTrait for SdCardStorageClass {

    fn new(options: HashMap<String, String>) -> Self {
        Self { options, ..Default::default()}
    }

    fn append_line(&mut self, line: String) -> Result<(), StorageError> {
        self.storage.push(line);
        Ok(())
    }

    fn replace_line(&mut self, line: String, line_number: usize) -> Result<(), StorageError> {
        if let Some(stored_line) = self.storage.get_mut(line_number) {
            *stored_line = line;
            Ok(())
        } else {
            Err(StorageError::OutOfBounds {
                requested: line_number,
                len: self.storage.len(),
            })
        }
    }

    fn read_line(&self, line_number: usize) -> Result<String, StorageError> {
        if let Some(stored_line) = self.storage.get(line_number) {
            Ok(stored_line.clone())
        } else {
            Err(StorageError::OutOfBounds {
                requested: line_number,
                len: self.storage.len(),
            })
        }
    }

    fn lines_len(&self) -> Result<usize, StorageError> {
        Ok(self.storage.len())
    }

    fn purge(&mut self) -> Result<(), StorageError> {
        self.storage.clear();
        Ok(())
    }

    fn reader(&self) -> impl std::io::Read {
        VecStringReader {
            data: &self.storage,
            index: 0,
            pos: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    // #[test]
    // fn test_append_line() {
    //     let mut instance = SdCardStorageClass::default();
    //     assert_eq!(instance.storage.len(), 0);
    //     assert!(instance.append_line("test 1".to_string()).is_ok());
    //     assert_eq!(instance.storage[0], "test 1".to_string());
    // }

    // #[test]
    // fn test_replace_line() {
    //     let mut instance = SdCardStorageClass::default();
    //     let _ = instance.append_line("test 1".to_string());
    //     let _ = instance.append_line("test 2".to_string());
    //     let _ = instance.append_line("test 3".to_string());

    //     assert!(instance.replace_line("replacement".to_string(), 1).is_ok());
    //     assert_eq!(instance.storage[1], "replacement".to_string());
    // }

    // #[test]
    // fn test_replace_line_when_out_of_bounds() {
    //     let mut instance = SdCardStorageClass::default();
    //     assert_eq!(
    //         instance.replace_line("replacement".to_string(), 99),
    //         Err(StorageError::OutOfBounds {
    //             requested: 99,
    //             len: 0
    //         })
    //     );
    // }

    // #[test]
    // fn test_read_line() {
    //     let mut instance = SdCardStorageClass::default();
    //     let _ = instance.append_line("test 1".to_string());

    //     assert_eq!(instance.read_line(0), Ok("test 1".to_string()));
    // }

    // #[test]
    // fn test_read_line_when_out_of_bounds() {
    //     let instance = SdCardStorageClass::default();
    //     assert_eq!(
    //         instance.read_line(99),
    //         Err(StorageError::OutOfBounds {
    //             requested: 99,
    //             len: 0
    //         })
    //     );
    // }

    // #[test]
    // fn test_lines_len() {
    //     let mut instance = SdCardStorageClass::default();
    //     let _ = instance.append_line("test 1".to_string());

    //     assert_eq!(instance.lines_len(), Ok(1));
    // }

    // #[test]
    // fn test_purge() {
    //     let mut instance = SdCardStorageClass::default();
    //     let _ = instance.append_line("test 1".to_string());
    //     assert_eq!(instance.storage.len(), 1);
    //     let _ = instance.purge();
    //     assert_eq!(instance.storage.len(), 0);
    // }

    // #[test]
    // fn test_reader_empty_storage() {
    //     let storage_class = SdCardStorageClass::default();
    //     let mut reader = storage_class.reader();
        
    //     let mut output = String::new();
    //     let bytes_read = reader.read_to_string(&mut output).unwrap();
        
    //     assert_eq!(bytes_read, 0);
    //     assert_eq!(output, "");
    // }

    // #[test]
    // fn test_reader_single_line() {
    //     let mut storage_class = SdCardStorageClass::default();
    //     storage_class.append_line("Hello World".to_string()).unwrap();
        
    //     let mut reader = storage_class.reader();
    //     let mut output = String::new();
    //     let bytes_read = reader.read_to_string(&mut output).unwrap();
        
    //     assert_eq!(bytes_read, 11);
    //     assert_eq!(output, "Hello World");
    // }

    // #[test]
    // fn test_reader_multiple_lines() {
    //     let mut storage_class = SdCardStorageClass::default();
    //     storage_class.append_line("Line 1".to_string()).unwrap();
    //     storage_class.append_line("Line 2".to_string()).unwrap();
    //     storage_class.append_line("Line 3".to_string()).unwrap();
        
    //     let mut reader = storage_class.reader();
    //     let mut output = String::new();
    //     let bytes_read = reader.read_to_string(&mut output).unwrap();
        
    //     // "Line 1\nLine 2\nLine 3" -> 6 + 1 + 6 + 1 + 6 = 20 bytes
    //     assert_eq!(bytes_read, 20);
    //     assert_eq!(output, "Line 1\nLine 2\nLine 3");
    // }
}

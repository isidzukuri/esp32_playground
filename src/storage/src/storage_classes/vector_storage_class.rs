use crate::storage_class_trait::StorageClassTrait;
use crate::storage_error::StorageError;

#[derive(Default, Debug)]
pub struct VectorStorageClass {
    pub storage: Vec<String>,
}

impl StorageClassTrait for VectorStorageClass {
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

    // fn reader(
    //     &mut self,
    //     // _reader: fn(_path: &'static str) -> (),
    fn reader(&self) -> impl std::io::Read {
        // let storage = vec!["string 1".to_string(), "string 2".to_string()];

        VecStringReader {
            data: &self.storage,
            index: 0,
            pos: 0,
        }
    }
        // let dynamic_csv_data: Vec<u8> = b"Name,Age\nAlice,30\nBob,25".to_vec();
        // Err(StorageError::NotImplemented {
        //     method_name: "exec_file_reader".to_string(),
        //     entity: "VectorStorageClass".to_string(),
        // })
}


use std::io::{self, Read};

struct VecStringReader<'a> {
    data: &'a Vec<String>,
    index: usize, // Which string we are currently reading
    pos: usize,   // Position inside the current string
}

impl Read for VecStringReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() || self.index >= self.data.len() {
            return Ok(0);
        }

        let mut bytes_written = 0;

        while bytes_written < buf.len() && self.index < self.data.len() {
            let current_str = &self.data[self.index];

            if self.pos < current_str.len() {
                let src = &current_str.as_bytes()[self.pos..];
                let dest = &mut buf[bytes_written..];
                
                let amt = std::cmp::min(src.len(), dest.len());
                dest[..amt].copy_from_slice(&src[..amt]);
                
                self.pos += amt;
                bytes_written += amt;
            } else {
                if self.index + 1 < self.data.len() {
                    buf[bytes_written] = b'\n';
                    bytes_written += 1;
                    
                    self.index += 1;
                    self.pos = 0;
                } else {
                    self.index += 1;
                }
            }
        }

        Ok(bytes_written)
    }
}





#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_line() {
        let mut instance = VectorStorageClass::default();
        assert_eq!(instance.storage.len(), 0);
        assert!(instance.append_line("test 1".to_string()).is_ok());
        assert_eq!(instance.storage[0], "test 1".to_string());
    }

    #[test]
    fn test_replace_line() {
        let mut instance = VectorStorageClass::default();
        let _ = instance.append_line("test 1".to_string());
        let _ = instance.append_line("test 2".to_string());
        let _ = instance.append_line("test 3".to_string());

        assert!(instance.replace_line("replacement".to_string(), 1).is_ok());
        assert_eq!(instance.storage[1], "replacement".to_string());
    }

    #[test]
    fn test_replace_line_when_out_of_bounds() {
        let mut instance = VectorStorageClass::default();
        assert_eq!(
            instance.replace_line("replacement".to_string(), 99),
            Err(StorageError::OutOfBounds {
                requested: 99,
                len: 0
            })
        );
    }

    #[test]
    fn test_read_line() {
        let mut instance = VectorStorageClass::default();
        let _ = instance.append_line("test 1".to_string());

        assert_eq!(instance.read_line(0), Ok("test 1".to_string()));
    }

    #[test]
    fn test_read_line_when_out_of_bounds() {
        let instance = VectorStorageClass::default();
        assert_eq!(
            instance.read_line(99),
            Err(StorageError::OutOfBounds {
                requested: 99,
                len: 0
            })
        );
    }

    #[test]
    fn test_lines_len() {
        let mut instance = VectorStorageClass::default();
        let _ = instance.append_line("test 1".to_string());

        assert_eq!(instance.lines_len(), Ok(1));
    }

    #[test]
    fn test_purge() {
        let mut instance = VectorStorageClass::default();
        let _ = instance.append_line("test 1".to_string());
        assert_eq!(instance.storage.len(), 1);
        let _ = instance.purge();
        assert_eq!(instance.storage.len(), 0);
    }

    #[test]
    fn test_exec_file_reader() {
        let mut instance = VectorStorageClass::default();
        fn test_fn(_path: &'static str) -> () {
            ()
        }

        assert_eq!(
            instance.exec_file_reader(test_fn),
            Err(StorageError::NotImplemented {
                method_name: "exec_file_reader".to_string(),
                entity: "VectorStorageClass".to_string(),
            })
        );
    }
}

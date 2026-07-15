use crate::storage_error::StorageError;
use std::collections::HashMap;

pub trait StorageClassTrait {
    fn new(options: HashMap<String, String>) -> Self;
    fn append_line(&mut self, line: String) -> Result<(), StorageError>;
    fn replace_line(&mut self, line: String, line_number: usize) -> Result<(), StorageError>;
    fn read_line(&self, line_number: usize) -> Result<String, StorageError>;
    fn lines_len(&self) -> Result<usize, StorageError>;
    fn purge(&mut self) -> Result<(), StorageError>;
    fn reader(&self) -> impl std::io::Read;
}




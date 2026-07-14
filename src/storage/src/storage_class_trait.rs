use crate::storage_error::StorageError;

pub trait StorageClassTrait {
    fn append_line(&mut self, line: String) -> Result<(), StorageError>;
    fn replace_line(&mut self, line: String, line_number: usize) -> Result<(), StorageError>;
    fn read_line(&self, line_number: usize) -> Result<String, StorageError>;
    fn lines_len(&self) -> Result<usize, StorageError>;
    fn purge(&mut self) -> Result<(), StorageError>;
    fn exec_file_reader(
        &mut self,
        reader: fn(path: &'static str) -> (),
    ) -> Result<(), StorageError>;
}

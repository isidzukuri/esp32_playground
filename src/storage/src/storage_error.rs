use std::fmt;
use std::sync::PoisonError;


#[derive(Debug, PartialEq)]
pub enum StorageError {
    OutOfBounds { requested: usize, len: usize },
    NotImplemented { method_name: String, entity: String },
    LockFailure(String),
    // Io(io::Error),
    // Parse(ParseIntError),
    // InvalidPort(u32),
}

// Implement std::fmt::Display
impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::OutOfBounds { requested, len } => {
                write!(
                    f,
                    "Index out of bounds: requested line {requested}, but storage only has {len} lines."
                )
            }
            StorageError::NotImplemented {
                method_name,
                entity,
            } => {
                write!(f, "Method is not implemented: '{entity}.{method_name}'")
            }
            StorageError::LockFailure(err_msg) => {
                // Fixed: replaced the typo `err` with `err_msg`
                write!(f, "Lock fails: {err_msg}")
            }
            
                // StorageError::Io(err) => write!(f, "I/O error occurred: {err}"),
              // StorageError::Parse(err) => write!(f, "Failed to parse configuration: {err}"),
              // StorageError::InvalidPort(port) => write!(f, "Port {port} is out of the valid range (1-65535)"),
        }
    }
}

// Implement std::error::Error (enables compatibility with `source()` and `dyn Error`)
impl std::error::Error for StorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StorageError::OutOfBounds {
                requested: _,
                len: _,
            } => None, // Custom errors typically have no underlying cause
            StorageError::NotImplemented {
                method_name: _,
                entity: _,
            } => None,
            StorageError::LockFailure(_) => None, 
                       // Custom errors typically have no underlying cause
                       // StorageError::Io(err) => Some(err),
                       // StorageError::Parse(err) => Some(err),
                       // StorageError::InvalidPort(_) => None, // Custom errors typically have no underlying cause
        }
    }
}

// Implement std::convert::From to allow the `?` operator to automatically convert errors
impl<T> From<PoisonError<T>> for StorageError {
    fn from(err: PoisonError<T>) -> Self {
        // We format the PoisonError into a String to drop the generic `T` guard safely.
        StorageError::LockFailure(err.to_string())
    }
}

// impl From<ParseIntError> for ConfigError {
//     fn from(err: ParseIntError) -> Self {
//         ConfigError::Parse(err)
//     }
// }

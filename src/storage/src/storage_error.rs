use std::fmt;

#[derive(Debug, PartialEq)]
pub enum StorageError {
    OutOfBounds { requested: usize, len: usize },
    NotImplemented { method_name: String, entity: String },
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
            } // StorageError::Io(err) => write!(f, "I/O error occurred: {err}"),
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
            } => None, // Custom errors typically have no underlying cause
                       // StorageError::Io(err) => Some(err),
                       // StorageError::Parse(err) => Some(err),
                       // StorageError::InvalidPort(_) => None, // Custom errors typically have no underlying cause
        }
    }
}

// Implement std::convert::From to allow the `?` operator to automatically convert errors
// impl From<io::Error> for ConfigError {
//     fn from(err: io::Error) -> Self {
//         ConfigError::Io(err)
//     }
// }

// impl From<ParseIntError> for ConfigError {
//     fn from(err: ParseIntError) -> Self {
//         ConfigError::Parse(err)
//     }
// }

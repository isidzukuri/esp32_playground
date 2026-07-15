use std::io::{self, Read};

/// A reader that sequentially streams the contents of a slice or vector of strings.
///
/// `VecStringReader` implements the [`Read`] trait, allowing you to treat a collection 
/// of `String`s as a continuous byte stream. 
///
/// # Behavior
/// - Individual strings are read character by character (as UTF-8 bytes).
/// - Adjacent strings in the collection are separated by a newline character (`\n`) in the output stream.
/// - The reader keeps track of its current position and will return `Ok(0)` (EOF) once all strings 
///   and their separators have been fully consumed.
///
/// ```

pub struct VecStringReader<'a> {
    /// Reference to the shared vector of strings to read from.
    pub data: &'a Vec<String>,
    /// Index of the string currently being read.
    pub index: usize,
    /// Byte position inside the current string.
    pub pos: usize,
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
    fn test_empty_vector() {
        let data = vec![];
        let mut reader = VecStringReader { data: &data, index: 0, pos: 0 };
        let mut buf = [0; 10];
        
        let bytes_read = reader.read(&mut buf).unwrap();
        assert_eq!(bytes_read, 0);
    }

    #[test]
    fn test_single_empty_string() {
        let data = vec!["".to_string()];
        let mut reader = VecStringReader { data: &data, index: 0, pos: 0 };
        let mut buf = [0; 10];
        
        let bytes_read = reader.read(&mut buf).unwrap();
        assert_eq!(bytes_read, 0);
    }

    #[test]
    fn test_multiple_strings_joined_with_newlines() {
        let data = vec![
            "Apple".to_string(),
            "Banana".to_string(),
            "Cherry".to_string(),
        ];
        let mut reader = VecStringReader { data: &data, index: 0, pos: 0 };
        
        let mut result = String::new();
        reader.read_to_string(&mut result).unwrap();
        
        assert_eq!(result, "Apple\nBanana\nCherry");
    }

    #[test]
    fn test_small_buffer_reads() {
        let data = vec!["AB".to_string(), "CD".to_string()];
        let mut reader = VecStringReader { data: &data, index: 0, pos: 0 };
        
        // Read 1 byte at a time to force transition logic 
        let mut buf = [0; 1];
        
        assert_eq!(reader.read(&mut buf).unwrap(), 1);
        assert_eq!(&buf[..1], b"A");
        
        assert_eq!(reader.read(&mut buf).unwrap(), 1);
        assert_eq!(&buf[..1], b"B");
        
        assert_eq!(reader.read(&mut buf).unwrap(), 1);
        assert_eq!(&buf[..1], b"\n"); // separator
        
        assert_eq!(reader.read(&mut buf).unwrap(), 1);
        assert_eq!(&buf[..1], b"C");
        
        assert_eq!(reader.read(&mut buf).unwrap(), 1);
        assert_eq!(&buf[..1], b"D");
        
        assert_eq!(reader.read(&mut buf).unwrap(), 0); // EOF
    }
}
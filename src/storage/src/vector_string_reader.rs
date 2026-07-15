use std::io::{self, Read};

pub struct VecStringReader<'a> {
    pub data: &'a Vec<String>,
    pub index: usize, // Which string we are currently reading
    pub pos: usize,   // Position inside the current string
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

// TODO: add tests
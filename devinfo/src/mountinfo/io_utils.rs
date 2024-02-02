use std::{
    fs::read,
    io::{BufRead, Read},
    path::Path,
};

const DEFAULT_RETRY_COUNT: u32 = 2;

type InconsistentReadError = String;
/// This is a container which is a BufRead while carrying the entire read
/// payload in a buffer.
pub(crate) struct ConsistentBufReader {
    buf: Vec<u8>,
}

impl Read for ConsistentBufReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.buf.as_slice().read(buf)
    }
}

impl BufRead for ConsistentBufReader {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        Ok(self.buf.as_slice())
    }

    fn consume(&mut self, amt: usize) {
        self.buf.as_slice().consume(amt)
    }

    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        self.buf.as_slice().read_until(byte, buf)
    }

    fn read_line(&mut self, buf: &mut String) -> std::io::Result<usize> {
        self.buf.as_slice().read_line(buf)
    }
}

impl ConsistentBufReader {
    // This tries to perform a consistent read, i.e. if two consecutive reads return the same byte
    // sequence, then the read is consistent, and we are not missing out on any entries due to a
    // seq index on a /proc file.
    pub(crate) fn new(
        path: &Path,
        retry_count: Option<u32>,
    ) -> Result<Self, InconsistentReadError> {
        let read_error = |error: std::io::Error| -> InconsistentReadError {
            format!(
                "failed to read file at {}: {}",
                path.to_string_lossy(),
                error
            )
        };

        let mut current_content = read(path).map_err(read_error)?;

        let retries = retry_count.unwrap_or(DEFAULT_RETRY_COUNT);
        for _ in 0 .. retries {
            let new_content = read(path).map_err(read_error)?;

            if new_content.eq(&current_content) {
                return Ok(Self { buf: new_content });
            }

            current_content = new_content;
        }

        Err(format!(
            "failed to get a consistent read output from file {} after {} attempts",
            path.to_string_lossy(),
            retries
        ))
    }
}

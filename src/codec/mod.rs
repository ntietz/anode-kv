use std::io::Read;

use thiserror::Error;

#[derive(Debug)]
pub enum Atom {
    SimpleString(String),
    Integer(i64),
    Error(String),
    BulkString(Option<Vec<u8>>),
    Array(Vec<Atom>),
}

#[derive(Error, Debug)]
pub enum ReadError {
    #[error("not implemented yet")]
    NotImplemented
}

/// decode takes in a Read and returns the first Atom it can decode, or an error
/// if the stream is empty or otherwise malformed.
pub fn decode<T: Read>(s: &mut T) -> Result<Atom, ReadError> {
    Err(ReadError::NotImplemented)
}

#[cfg(test)]
mod tests {
    #[test]
    fn not_implemented_yet() {

    }
}

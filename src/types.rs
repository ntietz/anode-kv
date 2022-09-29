#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Bytes(pub Vec<u8>);

impl From<Vec<u8>> for Bytes {
    fn from(t: Vec<u8>) -> Self {
        Bytes(t)
    }
}

pub type Key = Bytes;
pub type Value = Bytes;

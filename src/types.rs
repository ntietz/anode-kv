use std::fmt::{Debug, Error, Formatter};

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Bytes(pub Vec<u8>);

impl From<Vec<u8>> for Bytes {
    fn from(t: Vec<u8>) -> Self {
        Bytes(t)
    }
}

pub type Key = Bytes;
pub type Value = Bytes;

impl Debug for Bytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", String::from_utf8_lossy(&self.0))
    }
}

#[cfg(test)]
mod test {
    use super::Bytes;

    impl From<&str> for Bytes {
        fn from(t: &str) -> Self {
            Bytes(Vec::from(t))
        }
    }

    #[test]
    fn debug_format_is_readable() {
        assert_eq!(format!("{:?}", &Bytes::from("foo")), "foo");
    }
}

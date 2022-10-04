use std::fmt::{Debug, Error, Formatter};

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Blob(pub Vec<u8>);

impl From<Vec<u8>> for Blob {
    fn from(t: Vec<u8>) -> Self {
        Blob(t)
    }
}

pub type Key = Blob;
pub type Value = Blob;

impl Debug for Blob {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", String::from_utf8_lossy(&self.0))
    }
}

#[cfg(test)]
mod test {
    use super::Blob;

    impl From<&str> for Blob {
        fn from(t: &str) -> Self {
            Blob(Vec::from(t))
        }
    }

    #[test]
    fn debug_format_is_readable() {
        assert_eq!(format!("{:?}", &Blob::from("foo")), "foo");
    }
}

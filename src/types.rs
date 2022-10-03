use std::fmt::{Debug, Error, Formatter};

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Bytes(pub Vec<u8>);

impl<T> From<T> for Bytes
where
    T: AsRef<[u8]>,
{
    fn from(t: T) -> Self {
        Bytes(Vec::from(t.as_ref()))
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

    #[test]
    fn from() {
        assert_eq!(Bytes::from("foo"), Bytes(vec![102, 111, 111]));
        assert_eq!(Bytes::from("foo".to_string()), Bytes(vec![102, 111, 111]));
        assert_eq!(Bytes::from(vec![0x13, 0x37]), Bytes(vec![0x13, 0x37]));
        assert_eq!(Bytes::from([0x13, 0x37]), Bytes(vec![0x13, 0x37]));
        assert_eq!(Bytes::from(&[0x13, 0x37]), Bytes(vec![0x13, 0x37]));
    }

    #[test]
    fn debug() {
        assert_eq!(format!("{:?}", &Bytes::from("foo")), "foo");
        assert_eq!(format!("{:?}", &Bytes::from(&[0x13, 0x37])), "\u{13}7");
    }
}

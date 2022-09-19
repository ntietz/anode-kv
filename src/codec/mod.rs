use std::io::{Read, Write};

use thiserror::Error;

pub const CRLF: &str = "\r\n";

#[derive(Debug, PartialEq)]
pub enum Atom {
    SimpleString(String),
    Integer(i64),
    Error(String),
    BulkString(Option<Vec<u8>>),
    Array(Vec<Atom>),
}

#[derive(Error, Debug)]
pub enum ReadError {
    #[error("not implemented")]
    NotImplemented,

    #[error("incomplete read")]
    Incomplete,

    #[error("malformed input")]
    Malformed(&'static str),

    #[error("insufficient bytes")]
    InsufficientBytes(std::io::Error),

    #[error("unknown reason")]
    Failed(std::io::Error),
}

impl From<std::io::Error> for ReadError {
    fn from(err: std::io::Error) -> ReadError {
        ReadError::Failed(err)
    }
}

#[derive(Error, Debug)]
pub enum WriteError {
    #[error("not implemented")]
    NotImplemented,

    #[error("failed while writing")]
    Failed(std::io::Error),
}

/// encode takes in a Write and an Atom and encodes it.
pub fn encode<W: Write>(w: &mut W, atom: &Atom) -> Result<(), WriteError> {
    let mut buf: Vec<u8> = vec![];
    match atom {
        Atom::SimpleString(s) => {
            buf.push(b'+');
            buf.extend(s.bytes());
            buf.extend(CRLF.bytes());

            w.write_all(&buf[..]).map_err(WriteError::Failed)?;
            Ok(())
        }
        Atom::Error(s) => {
            buf.push(b'-');
            buf.extend(s.bytes());
            buf.extend(CRLF.bytes());

            w.write_all(&buf[..]).map_err(WriteError::Failed)?;
            Ok(())
        }
        Atom::Integer(v) => {
            buf.push(b':');
            buf.extend(format!("{}", v).bytes());
            buf.extend(CRLF.bytes());

            w.write_all(&buf[..]).map_err(WriteError::Failed)?;
            Ok(())
        }
        Atom::BulkString(s) => {
            buf.push(b'$');
            match s {
                Some(s) => {
                    buf.extend(format!("{}", s.len()).bytes());
                    buf.extend(CRLF.bytes());
                    buf.extend(s);
                    buf.extend(CRLF.bytes());
                }
                None => {
                    buf.extend(format!("{}", -1).bytes());
                    buf.extend(CRLF.bytes());
                }
            };

            w.write_all(&buf[..]).map_err(WriteError::Failed)?;
            Ok(())
        }
        Atom::Array(elems) => {
            buf.push(b'*');
            buf.extend(format!("{}", elems.len()).bytes());
            buf.extend(CRLF.bytes());
            w.write_all(&buf[..]).map_err(WriteError::Failed)?;

            for elem in elems {
                encode(w, elem)?;
            }
            Ok(())
        }
    }
}

/// decode takes in a Read and returns the first complete message which it
/// can decode, or an error if the stream is empty or otherwise malformed.
pub fn decode<T: Read>(s: &mut T) -> Result<Atom, ReadError> {
    let mut buf: [u8; 1] = [0];

    s.read_exact(&mut buf)
        .map_err(ReadError::InsufficientBytes)?;
    let tag = buf[0];

    match tag {
        b':' => Ok(Atom::Integer(read_integer(s)?)),
        b'+' => Ok(Atom::SimpleString(read_simple_string(s)?)),
        b'-' => Ok(Atom::Error(read_simple_string(s)?)),
        b'$' => {
            let length = read_integer(s)?;
            if length < 0 {
                Ok(Atom::BulkString(None))
            } else {
                Ok(Atom::BulkString(Some(read_bulk_string(
                    s,
                    length as usize,
                )?)))
            }
        }
        b'*' => {
            let length = read_integer(s)?;
            let mut elems: Vec<Atom> = vec![];
            for _ in 0..length {
                elems.push(decode(s)?);
            }
            Ok(Atom::Array(elems))
        }
        _ => Err(ReadError::NotImplemented),
    }
}

fn read_simple_string<T: Read>(s: &mut T) -> Result<String, ReadError> {
    let mut buf: [u8; 1] = [0];
    let mut bytes: Vec<u8> = Vec::with_capacity(1024);
    loop {
        s.read_exact(&mut buf)
            .map_err(ReadError::InsufficientBytes)?;

        if buf[0] == b'\r' {
            s.read_exact(&mut buf)
                .map_err(ReadError::InsufficientBytes)?;
            if buf[0] != b'\n' {
                return Err(ReadError::Malformed("expected \\n after \\r"));
            }
            break;
        }

        bytes.push(buf[0]);
    }

    let val = std::string::String::from_utf8_lossy(&bytes).into_owned();
    Ok(val)
}

fn read_integer<T: Read>(s: &mut T) -> Result<i64, ReadError> {
    let mut buf: [u8; 1] = [0];
    let mut val: i64 = 0;
    let mut positive = true;
    loop {
        s.read_exact(&mut buf)
            .map_err(ReadError::InsufficientBytes)?;

        if buf[0] == b'\r' {
            s.read_exact(&mut buf)
                .map_err(ReadError::InsufficientBytes)?;
            return match buf[0] {
                b'\n' if positive => Ok(val),
                b'\n' if !positive => Ok(-val),
                _ => Err(ReadError::Malformed("expected \\n after \\r")),
            };
        } else if buf[0] == b'-' {
            positive = false;
        } else {
            let digit = (buf[0] - b'0') as i64;
            val = val
                .checked_mul(10)
                .ok_or(ReadError::Malformed("overflowed i64"))?;
            val = val
                .checked_add(digit)
                .ok_or(ReadError::Malformed("overflowed i64"))?;
        }
    }
}

fn read_bulk_string<T: Read>(s: &mut T, length: usize) -> Result<Vec<u8>, ReadError> {
    let mut buf: Vec<u8> = vec![0; length];
    s.read_exact(&mut buf[..])
        .map_err(ReadError::InsufficientBytes)?;

    let mut crlf_buf: [u8; 2] = [0, 0];
    s.read_exact(&mut crlf_buf)
        .map_err(ReadError::InsufficientBytes)?;

    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[test]
    fn decodes_integers() {
        let encoded = ":123\r\n";
        let expected = Atom::Integer(123);

        let mut encoded_stream = encoded.as_bytes();
        let decoded = decode(&mut encoded_stream);
        assert!(decoded.is_ok());
        assert_eq!(expected, decoded.unwrap());
    }

    #[test]
    fn decoding_empty_string_fails() {
        let encoded = "";
        let mut encoded_stream = encoded.as_bytes();
        let decoded = decode(&mut encoded_stream);
        assert!(decoded.is_err());
        assert!(matches!(decoded, Err(ReadError::InsufficientBytes(_))))
    }

    #[test]
    fn decoding_int_extra_digits_fails() {
        let encoded = ":19223372036854775807\r\n";
        let mut encoded_stream = encoded.as_bytes();
        let decoded = decode(&mut encoded_stream);
        assert!(decoded.is_err());
        assert!(matches!(decoded, Err(ReadError::Malformed(m)) if m == "overflowed i64"));
    }

    #[test]
    fn decoding_int_too_big_fails() {
        let encoded = ":9223372036854775808\r\n";
        let mut encoded_stream = encoded.as_bytes();
        let decoded = decode(&mut encoded_stream);
        assert!(decoded.is_err());
        assert!(matches!(decoded, Err(ReadError::Malformed(m)) if m == "overflowed i64"));
    }

    #[test]
    fn decodes_basic_string() {
        let encoded = "+hello\r\n";
        let expected = Atom::SimpleString("hello".to_string());

        let mut encoded_stream = encoded.as_bytes();
        let decoded = decode(&mut encoded_stream);
        assert!(decoded.is_ok());
        assert_eq!(expected, decoded.unwrap());
    }

    #[test]
    fn decodes_error() {
        let encoded = "-ERR unknown command\r\n";
        let expected = Atom::Error("ERR unknown command".to_string());

        let mut encoded_stream = encoded.as_bytes();
        let decoded = decode(&mut encoded_stream);
        assert!(decoded.is_ok());
        assert_eq!(expected, decoded.unwrap());
    }

    #[test]
    fn decodes_bulk_string() {
        let encoded = "$5\r\nhello\r\n";
        let expected = Atom::BulkString(Some(vec![b'h', b'e', b'l', b'l', b'o']));

        let mut encoded_stream = encoded.as_bytes();
        let decoded = decode(&mut encoded_stream);
        assert!(decoded.is_ok());
        assert_eq!(expected, decoded.unwrap());
    }

    #[test]
    fn decodes_bulk_string_empty() {
        let encoded = "$0\r\n\r\n";
        let expected = Atom::BulkString(Some(vec![]));

        let mut encoded_stream = encoded.as_bytes();
        let decoded = decode(&mut encoded_stream);
        assert!(decoded.is_ok());
        assert_eq!(expected, decoded.unwrap());
    }

    #[test]
    fn decodes_bulk_string_null() {
        let encoded = "$-1\r\n";
        let expected = Atom::BulkString(None);

        let mut encoded_stream = encoded.as_bytes();
        let decoded = decode(&mut encoded_stream);
        assert!(decoded.is_ok());
        assert_eq!(expected, decoded.unwrap());
    }

    #[test]
    fn decodes_arrays() {
        let encoded = "*3\r\n+hello\r\n+world\r\n:1\r\n";
        let expected = Atom::Array(vec![
            Atom::SimpleString("hello".to_string()),
            Atom::SimpleString("world".to_string()),
            Atom::Integer(1),
        ]);

        let mut encoded_stream = encoded.as_bytes();
        let decoded = decode(&mut encoded_stream);
        assert!(decoded.is_ok());
        assert_eq!(expected, decoded.unwrap());
    }

    #[test]
    fn can_encode_decoded_messages() {
        let messages = vec![
            ":123\r\n",
            "+hello\r\n",
            "-ERR unknown command\r\n",
            "$5\r\nhello\r\n",
            "$0\r\n\r\n",
            "$-1\r\n",
            "*3\r\n+hello\r\n+world\r\n:1\r\n",
        ];

        for message in &messages {
            println!("handling: {}", message);
            let mut encoded_stream = message.as_bytes();
            let decoded = decode(&mut encoded_stream);
            assert!(decoded.is_ok());

            let mut buf: Vec<u8> = vec![];
            let encoded = encode(&mut buf, &decoded.unwrap());

            assert!(encoded.is_ok());
            assert_eq!(message.bytes().collect::<Vec<u8>>(), buf);
        }
    }

    #[bench]
    fn bench_parse_strings(b: &mut Bencher) {
        let encoded = "+Hello\r\n";

        b.iter(|| {
            let mut encoded_stream = encoded.as_bytes();
            let _decoded = decode(&mut encoded_stream);
        })
    }

    #[bench]
    fn bench_parse_integers(b: &mut Bencher) {
        let encoded = ":123456\r\n";

        b.iter(|| {
            let mut encoded_stream = encoded.as_bytes();
            let _decoded = decode(&mut encoded_stream);
        })
    }
}

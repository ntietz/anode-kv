use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Write};
use std::sync::{Arc, Mutex};

use thiserror::Error;

use crate::storage::StorageCommand;
use crate::types::{Blob, Value};

#[derive(Error, Debug)]
pub enum TransactionLogError {
    #[error("unknown reason: {0}")]
    Failed(#[from] std::io::Error),
}

pub struct TransactionLog {
    base: String,
    current_log: Arc<Mutex<File>>,
}

impl TransactionLog {
    pub fn new(base: &str) -> Result<Self, TransactionLogError> {
        let log_filename = current_log_filename(base);
        println!("log: {}", log_filename);
        let current_log = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .append(true)
            .open(&log_filename)?;
        let current_log = Arc::new(Mutex::new(current_log));

        Ok(Self {
            base: base.into(),
            current_log,
        })
    }

    pub fn record(&self, cmd: &StorageCommand) -> Result<(), TransactionLogError> {
        let mut log = self.current_log.lock().unwrap();
        match cmd {
            StorageCommand::Incr(key) => {
                log.write_all(b"I")?;
                log.write_all(&key.0.len().to_le_bytes()[..])?;
                log.write_all(&key.0[..])?;
            }
            StorageCommand::Decr(key) => {
                log.write_all(b"D")?;
                log.write_all(&key.0.len().to_le_bytes()[..])?;
                log.write_all(&key.0[..])?;
            }
            StorageCommand::Set(key, value) => {
                log.write_all(b"S")?;
                log.write_all(&key.0.len().to_le_bytes()[..])?;
                log.write_all(&key.0[..])?;
                match value {
                    Value::Int(i) => {
                        log.write_all(b"I")?;
                        log.write_all(&i.to_le_bytes()[..])?;
                    }
                    Value::Blob(b) => {
                        log.write_all(b"B")?;
                        log.write_all(&b.0.len().to_le_bytes()[..])?;
                        log.write_all(&b.0[..])?;
                    }
                }
            }
            _ => {}
        };

        Ok(())
    }

    // TODO: batch.
    //fn record_batch(&self, cmd: &[StorageCommand]) -> Result<(), TransactionLogError>;
    // TODO: compaction.
    //fn compact(&self) -> Result<(), TransactionLogError>;
    // TODO: force to disk
    //fn fsync(&self) -> Result<(), TransactionLogError>;

    pub fn read(&self) -> Result<LogIterator, TransactionLogError> {
        // acquire the lock for the write log to ensure that there are no writes
        // while we read the log.
        let write_lock = self.current_log.lock().unwrap();

        let log_filename = current_log_filename(&self.base);
        let read_log = OpenOptions::new().read(true).open(&log_filename)?;

        let reader = BufReader::new(read_log);

        // explicitly drop it so that it isn't released early
        // TODO: is there a possibility the rust compiler erases this entirely?
        drop(write_lock);

        Ok(LogIterator { reader })
    }
}

pub struct LogIterator {
    reader: BufReader<File>,
}

impl Iterator for LogIterator {
    type Item = StorageCommand;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_result() {
            Err(TransactionLogError::Failed(io_err)) => {
                if io_err.kind() == std::io::ErrorKind::UnexpectedEof {
                    // TODO: use logs/tracing
                    println!("reached end of log!");
                } else {
                    // TODO: log the error, this means the log had some error
                    // while reading. we can have a setting for whether to
                    // eat the error or panic.
                    panic!("encountered unexpected log error")
                }
                None
            }
            Ok(cmd) => cmd,
        }
    }
}

impl LogIterator {
    fn next_result(&mut self) -> Result<Option<StorageCommand>, TransactionLogError> {
        let mut header: [u8; 9] = [0; 9];
        self.reader.read_exact(&mut header[..])?;

        let tag = header[0];

        let (key_len_bytes, _) = header[1..].split_at(std::mem::size_of::<usize>());
        let key_len = usize::from_le_bytes(key_len_bytes.try_into().unwrap());

        let mut key_bytes: Vec<u8> = vec![0; key_len];
        self.reader.read_exact(&mut key_bytes[..])?;
        let key = Blob(key_bytes);

        match tag {
            b'I' => Ok(Some(StorageCommand::Incr(key))),
            b'D' => Ok(Some(StorageCommand::Decr(key))),
            b'S' => {
                // reuse the header bytes since we're done with the previous one
                self.reader.read_exact(&mut header[..])?;
                let value_tag = header[0];

                match value_tag {
                    b'I' => {
                        let (int_bytes, _) = header[1..].split_at(std::mem::size_of::<usize>());
                        let val = i64::from_le_bytes(int_bytes.try_into().unwrap());

                        Ok(Some(StorageCommand::Set(key, Value::Int(val))))
                    }
                    b'B' => {
                        let (len_bytes, _) = header[1..].split_at(std::mem::size_of::<usize>());
                        let value_len = usize::from_le_bytes(len_bytes.try_into().unwrap());
                        let mut value_bytes: Vec<u8> = vec![0; value_len];
                        self.reader.read_exact(&mut value_bytes[..])?;

                        let value = Blob(value_bytes);

                        Ok(Some(StorageCommand::Set(key, Value::Blob(value))))
                    }
                    _ => {
                        // TODO: log the error, this means the log is corrupted.
                        // once this is logged, we can have a setting for whether
                        // to panic on corruption or swallow the error.
                        panic!("encountered log corruption")
                    }
                }
            }
            _ => {
                // TODO: log the error, this means the log is corrupted.
                // once this is logged, we can have a setting for whether
                // to panic on corruption or swallow the error.
                panic!("encountered log corruption")
            }
        }
    }
}

fn current_log_filename(base: &str) -> String {
    format!("{}.current", base)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_out_commands() {
        let tmp = ".tmp/tlog-test-write/";
        setup_tmp_dir(tmp);

        let base_path = format!("{}/log", tmp);

        let commands = vec![
            StorageCommand::Set("a".into(), "1".bytes().collect::<Vec<u8>>().into()),
            StorageCommand::Incr("a".into()),
        ];

        let log = TransactionLog::new(&base_path).expect("should create log");
        for cmd in commands {
            log.record(&cmd).expect("should record command");
        }

        //let content = std::fs::read(current_log_filename(&base_path)).expect("should read the file");
        let content = std::fs::read_to_string(current_log_filename(&base_path))
            .expect("should read the file");

        let expected_log = "S\u{1}\0\0\0\0\0\0\0aB\u{1}\0\0\0\0\0\0\01I\u{1}\0\0\0\0\0\0\0a";
        assert_eq!(expected_log, content);

        cleanup_tmp_dir(tmp);
    }

    #[test]
    fn reads_commands_back() {
        let tmp = ".tmp/tlog-test-read/";
        setup_tmp_dir(tmp);
        let base_path = format!("{}/log", tmp);

        let commands = vec![
            StorageCommand::Set("a".into(), "1".bytes().collect::<Vec<u8>>().into()),
            StorageCommand::Incr("a".into()),
        ];

        let log = TransactionLog::new(&base_path).expect("should create log");
        for cmd in &commands {
            log.record(&cmd).expect("should record command");
        }

        let read_log = TransactionLog::new(&base_path).expect("should create log");
        let recorded_commands: Vec<StorageCommand> = read_log.read().unwrap().into_iter().collect();
        assert_eq!(commands, recorded_commands);

        cleanup_tmp_dir(tmp);
    }

    /// sets up the tmp dir including cleaning it beforehand, in case it exists.
    fn setup_tmp_dir(dir: &str) {
        cleanup_tmp_dir(dir);
        std::fs::create_dir_all(&dir).unwrap();
    }

    /// sets up the tmp dir including cleaning it beforehand, in case it exists.
    fn cleanup_tmp_dir(dir: &str) {
        match std::fs::remove_dir_all(&dir) {
            Err(_) => println!("yay, it was already cleaned up!"),
            Ok(_) => println!("cleaning up after someone else"),
        }
    }
}

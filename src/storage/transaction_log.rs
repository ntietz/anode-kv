use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufRead, Write};
use std::sync::{Arc, Mutex};

use thiserror::Error;

use super::StorageCommand;

#[derive(Error, Debug)]
pub enum TransactionLogError {
    #[error("failed to serialize: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("unknown reason: {0}")]
    Failed(#[from] std::io::Error),
}


pub trait TransactionLog {
    type Iterable: IntoIterator<Item=StorageCommand>;

    fn record(&self, cmd: StorageCommand) -> Result<(), TransactionLogError>;
    fn compact(&self) -> Result<(), TransactionLogError>;
    fn fsync(&self) -> Result<(), TransactionLogError>;

    fn read(&self) -> Result<Self::Iterable, TransactionLogError>;
}

pub struct NaiveFileBackedTransactionLog {
    // current new writes
    // snapshot
    // <- new new writes

    current_log: Arc<Mutex<File>>,
    //base: String,
}

impl NaiveFileBackedTransactionLog {
    pub fn new(base: &str) -> Result<Self, TransactionLogError> {
        let log_filename = current_log_filename(base);
        println!("log: {}", log_filename);
        let current_log = OpenOptions::new().create(true).read(true).write(true).append(true).open(&log_filename)?;
        let current_log = Arc::new(Mutex::new(current_log));

        Ok(NaiveFileBackedTransactionLog {
            //base: base.into(),
            current_log,
        })
    }

}

fn current_log_filename(base: &str) -> String {
    format!("{}.current", base)
}

impl TransactionLog for NaiveFileBackedTransactionLog {
    type Iterable = Vec<StorageCommand>;

    fn record(&self, cmd: StorageCommand) -> Result<(), TransactionLogError> {
        let serialized = serde_json::to_string(&cmd)?;

        {
            let mut log = self.current_log.lock().unwrap();
            writeln!(log, "{}", serialized)?;
        }

        Ok(())
    }

    fn compact(&self) -> Result<(), TransactionLogError> {
        // we can get writes during the compaction
        // the first thing we do with compaction:
        //  - redirect writes to a new file
        //  - start compacting the old file

        todo!("not implemented")
    }

    fn fsync(&self) -> Result<(), TransactionLogError> {
        todo!("not implemented")
    }

    fn read(&self) -> Result<Self::Iterable, TransactionLogError> {
        let log = self.current_log.lock().unwrap().try_clone()?;
        let reader = BufReader::new(log);

        let lines = reader.lines();

        let result_iter: Result<Vec<_>, _> = lines.map(|line| -> Result<StorageCommand, TransactionLogError> {
            let line = line?;
            let parsed_line = serde_json::from_str(&line)?;
            Ok(parsed_line)
        }).collect();
        let iter = result_iter?;

        Ok(iter)
    }
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

        let naive_log = NaiveFileBackedTransactionLog::new(&base_path).expect("should create log");
        for cmd in commands {
            naive_log.record(cmd).expect("should record command");
        }

        let content = std::fs::read_to_string(current_log_filename(&base_path)).expect("should read the file");

        let expected_log = "{\"Set\":[[97],{\"Blob\":[49]}]}\n{\"Incr\":[97]}\n".to_string();
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

        let naive_log = NaiveFileBackedTransactionLog::new(&base_path).expect("should create log");
        for cmd in &commands {
            naive_log.record(cmd.clone()).expect("should record command");
        }

        let read_log = NaiveFileBackedTransactionLog::new(&base_path).expect("should create log");
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

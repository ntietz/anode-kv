use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::types::{Blob, Key, Value};

mod transaction_log;
pub use transaction_log::{NaiveFileBackedTransactionLog, TransactionLog, TransactionLogError};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum StorageCommand {
    Set(Key, Value),
    Get(Key),
    Incr(Key),
    Decr(Key),
}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("integer overflow")]
    Overflow,

    #[error("not an integer")]
    NotAnInteger,

    #[error("transaction log error: {0}")]
    LogError(#[from] TransactionLogError),

    #[error("unknown reason: {0}")]
    Failed(#[from] std::io::Error),
}

pub struct InMemoryStorage<L: TransactionLog> {
    data: HashMap<Key, Value>,
    recv_queue: StorageRecvQueue,
    log: L,
    durable: bool,
}

pub type StorageRecvQueue = mpsc::Receiver<(
    StorageCommand,
    oneshot::Sender<Result<Option<Value>, StorageError>>,
)>;
pub type StorageSendQueue = mpsc::Sender<(
    StorageCommand,
    oneshot::Sender<Result<Option<Value>, StorageError>>,
)>;

impl InMemoryStorage<NaiveFileBackedTransactionLog> {
    pub fn new(recv_queue: StorageRecvQueue) -> Self {
        let data = HashMap::new();
        // TODO: pass this in instead
        let log = NaiveFileBackedTransactionLog::new("log").expect("creating transaction log shold not fail");

        Self { data, recv_queue, log, durable: true }
    }

    pub async fn from_log(recv_queue: StorageRecvQueue) -> Self {
        let mut store = Self::new(recv_queue);

        println!("starting log read");
        store.disable_durability();
        let mut count: usize = 0;
        for cmd in store.log.read().unwrap() {
            store.handle_cmd(cmd).await.expect("should work");
            count += 1;
        }
        store.enable_durability();
        println!("finished log read; {} records", count);

        store

    }

    pub async fn run(&mut self) {
        while let Some((cmd, tx)) = self.recv_queue.recv().await {
            let response = self.handle_cmd(cmd).await;

            if tx.send(response).is_err() {
                log::error!("could not return value to requester; early disconnection?");
            }
        }
    }

    pub async fn handle_cmd(&mut self, cmd: StorageCommand) -> Result<Option<Value>, StorageError> {
        self.record_cmd(&cmd)?;
        match cmd {
            StorageCommand::Set(key, value) => {
                self.data.insert(key, value);
                Ok(None)
            }
            StorageCommand::Get(key) => Ok(self.data.get(&key).cloned()),
            StorageCommand::Incr(key) => self.handle_add(key, 1).await,
            StorageCommand::Decr(key) => self.handle_add(key, -1).await,
        }
    }

    async fn handle_add(&mut self, key: Key, amount: i64) -> Result<Option<Value>, StorageError> {
        let entry = self.data.entry(key).or_insert(Value::Int(0));
        match entry {
            Value::Int(i) => {
                *i = safe_add(*i, amount)?;
                Ok(Some(Value::Int(*i)))
            }
            Value::Blob(Blob(b)) => match atoi::atoi::<i64>(b) {
                None => Err(StorageError::NotAnInteger),
                Some(i) => {
                    *entry = Value::Int(safe_add(i, amount)?);
                    Ok(Some(entry.clone()))
                }
            },
        }
    }

    fn enable_durability(&mut self) {
        self.durable = true;
    }

    fn disable_durability(&mut self) {
        self.durable = false;
    }

    fn record_cmd(&self, cmd: &StorageCommand) -> Result<(), StorageError> {
        if self.durable {
            match cmd {
                StorageCommand::Get(_) => {},
                _ => {
                    self.log.record(cmd)?;
                }
            };
        }
        Ok(())
    }
}

fn safe_add(a: i64, b: i64) -> Result<i64, StorageError> {
    match a.checked_add(b) {
        Some(c) => Ok(c),
        None => Err(StorageError::Overflow),
    }
}

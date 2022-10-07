use std::collections::HashMap;

use thiserror::Error;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::types::{Blob, Key, Value};

mod transaction_log;

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

    #[error("unknown reason: {0}")]
    Failed(#[from] std::io::Error),
}

pub struct InMemoryStorage {
    data: HashMap<Key, Value>,
    recv_queue: StorageRecvQueue,
}

pub type StorageRecvQueue = mpsc::Receiver<(
    StorageCommand,
    oneshot::Sender<Result<Option<Value>, StorageError>>,
)>;
pub type StorageSendQueue = mpsc::Sender<(
    StorageCommand,
    oneshot::Sender<Result<Option<Value>, StorageError>>,
)>;

impl InMemoryStorage {
    pub fn new(recv_queue: StorageRecvQueue) -> Self {
        let data = HashMap::new();

        Self { data, recv_queue }
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
}

fn safe_add(a: i64, b: i64) -> Result<i64, StorageError> {
    match a.checked_add(b) {
        Some(c) => Ok(c),
        None => Err(StorageError::Overflow),
    }
}

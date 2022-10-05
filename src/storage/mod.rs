use std::collections::HashMap;

use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::types::{Blob, Key, Value};

pub enum StorageCommand {
    Set(Key, Value),
    Get(Key),
    Incr(Key),
    Decr(Key),
}

pub struct InMemoryStorage {
    data: HashMap<Key, Value>,
    recv_queue: StorageRecvQueue,
}

pub type StorageRecvQueue = mpsc::Receiver<(
    StorageCommand,
    oneshot::Sender<Result<Option<Value>, std::io::Error>>,
)>;
pub type StorageSendQueue = mpsc::Sender<(
    StorageCommand,
    oneshot::Sender<Result<Option<Value>, std::io::Error>>,
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

    pub async fn handle_cmd(
        &mut self,
        cmd: StorageCommand,
    ) -> Result<Option<Value>, std::io::Error> {
        match cmd {
            StorageCommand::Set(key, value) => {
                self.data.insert(key, value);
                Ok(None)
            }
            StorageCommand::Get(key) => Ok(self.data.get(&key).cloned()),
            StorageCommand::Incr(key) => {
                let entry = self.data.entry(key).or_insert(Value::Int(0));
                match entry {
                    Value::Int(i) => {
                        *i += 1;
                        return Ok(Some(Value::Int(*i)));
                    }
                    Value::Blob(Blob(b)) => match atoi::atoi::<i64>(b) {
                        None => {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "not an integer",
                            ));
                        }
                        Some(i) => {
                            *entry = Value::Int(i + 1);
                            return Ok(Some(entry.clone()));
                        }
                    },
                }
            }
            StorageCommand::Decr(key) => {
                let entry = self.data.entry(key).or_insert(Value::Int(0));
                match entry {
                    Value::Int(i) => {
                        *i -= 1;
                        return Ok(Some(Value::Int(*i)));
                    }
                    Value::Blob(Blob(b)) => match atoi::atoi::<i64>(b) {
                        None => {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "not an integer",
                            ));
                        }
                        Some(i) => {
                            *entry = Value::Int(i - 1);
                            return Ok(Some(entry.clone()));
                        }
                    },
                }
            }

        }
    }
}

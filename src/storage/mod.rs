use std::collections::HashMap;

use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::types::{Key, Value};

pub enum StorageCommand {
    Set(Key, Value),
    Get(Key),
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
            let response = match cmd {
                StorageCommand::Set(key, value) => {
                    self.data.insert(key, value);
                    Ok(None)
                }
                StorageCommand::Get(key) => Ok(self.data.get(&key).cloned()),
            };

            if tx.send(response).is_err() {
                log::error!("could not return value to requester; early disconnection?");
            }
        }
    }
}

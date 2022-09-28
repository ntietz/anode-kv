use tokio::sync::mpsc;
use tokio::sync::oneshot;

use std::collections::HashMap;

pub enum StorageCommand {
    Set(Vec<u8>, Vec<u8>),
    Get(Vec<u8>),
}

pub struct InMemoryStorage {
    data: HashMap<Vec<u8>, Vec<u8>>,
    recv_queue: StorageRecvQueue,
}

pub type StorageRecvQueue = mpsc::Receiver<(
    StorageCommand,
    oneshot::Sender<Result<Option<Vec<u8>>, std::io::Error>>,
)>;
pub type StorageSendQueue = mpsc::Sender<(
    StorageCommand,
    oneshot::Sender<Result<Option<Vec<u8>>, std::io::Error>>,
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

            if let Err(_) = tx.send(response) {
                log::error!("could not return value to requester; early disconnection?");
            }
        }
    }
}

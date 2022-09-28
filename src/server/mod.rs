use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

use crate::connection::ConnectionManager;
use crate::storage::{InMemoryStorage, StorageSendQueue};

// TODO: custom type that is wraps Vec<u8> and debug prints utf-8 string if possible, else bytes

pub struct Server {
    listener: TcpListener,
    connection_manager: ConnectionManager,
    storage: Arc<Mutex<InMemoryStorage>>,
    context: Context,
}

#[derive(Clone)]
pub struct Context {
    pub storage_queue: StorageSendQueue,
}

impl Server {
    pub async fn create(addr: &str) -> std::io::Result<Server> {
        let listener = TcpListener::bind(addr).await?;
        let connection_manager = ConnectionManager::default();

        let (tx, rx) = mpsc::channel(100); // TODO: tunable option?

        let storage = Arc::new(Mutex::new(InMemoryStorage::new(rx)));
        let context = Context::new(tx);

        Ok(Server {
            listener,
            connection_manager,
            storage,
            context,
        })
    }

    pub async fn run(&mut self) {
        let storage = self.storage.clone();
        let _storage_handle = tokio::spawn(async move {
            let mut storage = storage.lock().await;
            storage.run().await;
        });

        loop {
            match self.listener.accept().await {
                Ok((socket, addr)) => {
                    self.connection_manager
                        .take_connection(self.context.clone(), socket, addr)
                        .await;
                }
                Err(e) => {
                    log::error!("error accepting connection: {}", e);
                }
            }
        }
    }

    pub fn addr(&self) -> String {
        let local_addr = self.listener.local_addr().unwrap();
        format!("127.0.0.1:{}", local_addr.port())
    }
}

impl Context {
    pub fn new(storage_queue: StorageSendQueue) -> Self {
        Self { storage_queue }
    }

    pub fn dummy() -> Self {
        let (tx, _rx) = mpsc::channel(1);
        Self { storage_queue: tx }
    }
}

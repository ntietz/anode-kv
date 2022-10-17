use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

use crate::config::Config;
use crate::connection::ConnectionManager;
use crate::storage::{InMemoryStorage, StorageSendQueue};

pub struct Server {
    listener: TcpListener,
    connection_manager: ConnectionManager,
    storage: Arc<Mutex<InMemoryStorage>>,
    context: Context,
}

/// Context is used to pass dependencies into connection handlers and the command
/// processor.
///
/// This is Clone and certain constraints will be upheld:
///  - Anything which is present will be efficient to clone (or requires clone, like queues)
///  - Any shared state will be wrapped in Arc<Mutex<>> or similar to ensure safety
#[derive(Clone)]
pub struct Context {
    pub storage_queue: StorageSendQueue,
    pub config: Config,
}

impl Server {
    pub async fn create(config: Config) -> std::io::Result<Server> {
        let (tx, rx) = mpsc::channel(config.storage_queue_size);
        let context = Context::new(tx, config.clone());

        let listener = TcpListener::bind(&config.address).await?;
        let connection_manager = ConnectionManager::default();

        let storage = Arc::new(Mutex::new(InMemoryStorage::new(rx, context.clone())));

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
    pub fn new(storage_queue: StorageSendQueue, config: Config) -> Self {
        Self {
            storage_queue,
            config,
        }
    }
}

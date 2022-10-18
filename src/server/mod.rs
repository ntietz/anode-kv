use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

use crate::config::Config;
use crate::connection::ConnectionManager;
use crate::storage::{InMemoryStorage, StorageSendQueue};
use crate::transaction::{TransactionSendQueue, TransactionWorker};

pub struct Server {
    listener: TcpListener,
    connection_manager: ConnectionManager,
    storage: Arc<Mutex<InMemoryStorage>>,
    transaction_worker: Arc<Mutex<TransactionWorker>>,
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
    pub transaction_queue: TransactionSendQueue,
    pub config: Config,
}

impl Server {
    pub async fn create(config: Config) -> std::io::Result<Server> {
        let (tx, rx) = mpsc::channel(config.storage_queue_size);
        let (ttx, trx) = mpsc::channel(config.transaction_queue_size);
        let context = Context::new(tx, ttx, config.clone());

        let listener = TcpListener::bind(&config.address).await?;
        let connection_manager = ConnectionManager::default();

        let mut storage_impl = InMemoryStorage::new(rx, context.clone());
        if config.read_log {
            storage_impl.load_from_log(config.clone()).await;
        }
        let storage = Arc::new(Mutex::new(storage_impl));

        let transaction_impl = TransactionWorker::new(trx, config.clone());
        let transaction_worker = Arc::new(Mutex::new(transaction_impl));

        Ok(Server {
            listener,
            connection_manager,
            storage,
            context,
            transaction_worker,
        })
    }

    pub async fn run(&mut self) {
        let storage = self.storage.clone();
        let _storage_handle = tokio::spawn(async move {
            let mut storage = storage.lock().await;
            storage.run().await;
        });

        let transaction_worker = self.transaction_worker.clone();
        let _transaction_worker_handle = tokio::spawn(async move {
            let mut transaction_worker = transaction_worker.lock().await;
            transaction_worker.run().await;
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
    pub fn new(
        storage_queue: StorageSendQueue,
        transaction_queue: TransactionSendQueue,
        config: Config,
    ) -> Self {
        Self {
            storage_queue,
            transaction_queue,
            config,
        }
    }
}

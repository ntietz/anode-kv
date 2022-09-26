use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

use crate::connection::ConnectionManager;

pub struct Server {
    listener: TcpListener,
    connection_manager: Arc<Mutex<ConnectionManager>>,
}

impl Server {
    pub async fn create(addr: &str) -> std::io::Result<Server> {
        let listener = TcpListener::bind(addr).await?;
        let connection_manager = Arc::new(Mutex::new(ConnectionManager::default()));

        Ok(Server {
            listener,
            connection_manager,
        })
    }

    pub async fn run(&mut self) {
        loop {
            match self.listener.accept().await {
                Ok((socket, addr)) => {
                    let mut connection_manager =
                        self.connection_manager.lock().expect("lock is poisoned");
                    let mut connection = connection_manager.add_connection(socket, addr);
                    drop(connection_manager); // unlock it as soon as possible
                    tokio::spawn(async move {
                        connection.handle().await;
                    });
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

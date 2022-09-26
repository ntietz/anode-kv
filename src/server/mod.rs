use tokio::net::TcpListener;

use crate::connection::ConnectionManager;

pub struct Server {
    listener: TcpListener,
    connection_manager: ConnectionManager,
}

impl Server {
    pub async fn create(addr: &str) -> std::io::Result<Server> {
        let listener = TcpListener::bind(addr).await?;
        let connection_manager = ConnectionManager::default();

        Ok(Server {
            listener,
            connection_manager,
        })
    }

    pub async fn run(&mut self) {
        loop {
            match self.listener.accept().await {
                Ok((socket, addr)) => {
                    self.connection_manager.take_connection(socket, addr).await;
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

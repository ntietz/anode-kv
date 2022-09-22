use tokio::net::TcpListener;

use crate::connection::accept_connection;

pub async fn launch(addr: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                accept_connection(socket, addr).await?;
            }
            Err(e) => {
                log::error!("Error accepting connection: {}", e);
            }
        }
    }
}

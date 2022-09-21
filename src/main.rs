use tokio::net::TcpListener;

use anode_kv::connection::accept_connection;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let listener = TcpListener::bind("127.0.0.1:11311").await?;

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

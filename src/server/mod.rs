use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use std::net::SocketAddr;

use crate::connection::accept_connection;

pub async fn launch(addr: &str) -> std::io::Result<(SocketAddr, JoinHandle<()>)> {
    let listener = TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr().expect("should bind to local address");

    let accept_loop = tokio::spawn(accept_connection_loop(listener));

    Ok((local_addr, accept_loop))
}

pub async fn accept_connection_loop(listener: TcpListener) {
    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                tokio::spawn(accept_connection(socket, addr));
            },
            Err(e) => {
                log::error!("error accepting connection: {}", e);
            },
        }
    }
}

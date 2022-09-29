use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use tokio::net::TcpStream;

mod connection;
mod tracker;

pub use connection::{Connection, ConnectionId};
pub use tracker::ConnectionTracker;

pub use crate::server::Context;

#[derive(Clone)]
pub struct ConnectionManager {
    latest_id: Arc<AtomicU64>,
    tracker: Arc<Mutex<ConnectionTracker>>,
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self {
            latest_id: Arc::new(AtomicU64::new(0)),
            tracker: Arc::new(Mutex::new(ConnectionTracker::default())),
        }
    }
}

impl ConnectionManager {
    pub async fn take_connection(
        &mut self,
        ctx: Context,
        socket: TcpStream,
        addr: SocketAddr,
    ) -> ConnectionId {
        let id = self.latest_id.fetch_add(1, Ordering::SeqCst);

        let mut connection = Connection::new(ctx, id, socket, addr);
        log::info!("accepted new connection. id={}, addr={}", id, addr);

        let tracker = self.tracker.clone();

        let handle = tokio::spawn(async move {
            match connection.handle().await {
                Ok(()) => {
                    log::info!("connection {}: terminated gracefully", id);
                    tracker.lock().unwrap().remove(id);
                }
                Err(e) => {
                    log::error!("connection {}: error {}", id, e);
                    tracker.lock().unwrap().remove(id);
                }
            };
        });

        self.tracker.lock().unwrap().add(id, handle);

        id
    }
}

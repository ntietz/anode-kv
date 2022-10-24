use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use tokio::net::TcpStream;

mod conn;
mod tracker;

pub use conn::{Connection, ConnectionId};
pub use tracker::ConnectionTracker;

use crate::server::Context;

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
        context: Context,
        socket: TcpStream,
        addr: SocketAddr,
    ) -> ConnectionId {
        let id = self.latest_id.fetch_add(1, Ordering::SeqCst);

        let mut connection = Connection::new(context, id, socket, addr);
        let span = tracing::debug_span!("ConnectionManager::take_connection:1", id=id, addr=?addr);
        let _guard = span.enter();

        let tracker = self.tracker.clone();

        let handle = tokio::spawn(async move {
            let span = tracing::debug_span!("ConnectionManager::take_connection::handle", id=id, addr=?addr);
            let _guard = span.enter();

            match connection.handle().await {
                Ok(()) => {
                    tracing::info!(id, "connection terminated gracefully");
                    tracker.lock().unwrap().remove(id);
                }
                Err(e) => {
                    tracing::error!(id, e=?e, "connection ended with error");
                    tracker.lock().unwrap().remove(id);
                }
            };
        });

        self.tracker.lock().unwrap().add(id, handle);

        id
    }
}

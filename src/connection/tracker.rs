use std::collections::{HashMap, HashSet};
use tokio::task::JoinHandle;

use super::ConnectionId;

pub struct ConnectionTracker {
    active_connections: HashSet<ConnectionId>,
    connection_handles: HashMap<ConnectionId, JoinHandle<()>>,
}

impl Default for ConnectionTracker {
    fn default() -> Self {
        Self {
            active_connections: HashSet::new(),
            connection_handles: HashMap::new(),
        }
    }
}

impl ConnectionTracker {
    pub fn add(&mut self, id: ConnectionId, handle: JoinHandle<()>) {
        self.active_connections.insert(id);
        self.connection_handles.insert(id, handle);
    }

    pub fn remove(&mut self, id: ConnectionId) {
        self.active_connections.remove(&id);

        if let Some(handle) = self.connection_handles.get(&id) {
            if !handle.is_finished() {
                handle.abort();
            }
            self.connection_handles.remove(&id);
        }
    }
}

use bytes::{Buf, BytesMut};
use std::collections::HashSet;
use std::io::Cursor;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::codec::{decode, encode, Atom};

type ConnectionId = u64;

pub struct Connection {
    id: ConnectionId,
    socket: TcpStream,
    addr: SocketAddr,
}

pub struct ConnectionManager {
    active_connections: HashSet<ConnectionId>,
    latest_id: u64,
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self {
            active_connections: HashSet::new(),
            latest_id: 0,
        }
    }
}

impl ConnectionManager {
    pub fn add_connection(&mut self, socket: TcpStream, addr: SocketAddr) -> Connection {
        let id = self.assign_next_id();
        let connection = Connection { id, socket, addr };
        log::info!("accepted new connection. id={}, addr={}", id, addr);

        self.active_connections.insert(id);

        connection
    }

    pub fn remove_connection(&mut self, id: ConnectionId) -> bool {
        self.active_connections.remove(&id)
    }

    fn assign_next_id(&mut self) -> u64 {
        self.latest_id += 1;
        self.latest_id
    }
}

impl Connection {
    pub async fn handle(&mut self) -> std::io::Result<()> {
        log::info!("Accepting connection from {}", self.addr);

        let mut buffer = BytesMut::with_capacity(4 * 1024);

        loop {
            if 0 == self.socket.read_buf(&mut buffer).await? {
                println!("done");
                break;
            }

            loop {
                let mut cursor = Cursor::new(&buffer[..]);
                if let Ok(token) = decode(&mut cursor) {
                    let pos = cursor.position() as usize;
                    println!("pos: {}, token: {:?}, buf: {:?}", pos, token, buffer);
                    buffer.advance(pos);

                    if buffer.is_empty() {
                        let resp = Atom::Error("ERR not implemented yet".to_string());
                        let mut write_buf: Vec<u8> = vec![];
                        encode(&mut write_buf, &resp)
                            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?; // TODO: handle error
                        println!("{:?}", write_buf);

                        self.socket.write_all(&write_buf).await?;
                    }
                } else {
                    break;
                }

                println!("buf: {:?}", buffer);
            }
        }

        Ok(())
    }
}

pub async fn run_connection(mut connection: Connection) -> std::io::Result<()> {
    connection.handle().await
}

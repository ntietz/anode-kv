use bytes::{Buf, BytesMut};
use std::io::Cursor;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::codec::{decode, encode, Token};

pub type ConnectionId = u64;

pub struct Connection {
    id: ConnectionId,
    socket: TcpStream,
    addr: SocketAddr,
}

impl Connection {
    pub fn new(id: ConnectionId, socket: TcpStream, addr: SocketAddr) -> Self {
        Connection { id, socket, addr }
    }

    pub async fn handle(&mut self) -> std::io::Result<()> {
        log::info!("(id={}) accepting connection from {}", self.id, self.addr);

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
                        let resp = Token::Error("ERR not implemented yet".to_string());
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

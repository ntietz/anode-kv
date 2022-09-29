use std::io::Cursor;
use std::net::SocketAddr;

use bytes::{Buf, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::codec::{decode, encode, Token};
use crate::command::{Command, CommandError, CommandProcessor};
use crate::server::Context;

pub type ConnectionId = u64;

pub struct Connection {
    id: ConnectionId,
    socket: TcpStream,
    addr: SocketAddr,
    context: Context,
}

impl Connection {
    pub fn new(context: Context, id: ConnectionId, socket: TcpStream, addr: SocketAddr) -> Self {
        Connection {
            id,
            socket,
            addr,
            context,
        }
    }

    pub async fn handle(&mut self) -> std::io::Result<()> {
        log::info!("(id={}) accepting connection from {}", self.id, self.addr);

        let mut buffer = BytesMut::with_capacity(4 * 1024);
        let cp = CommandProcessor::new(self.context.clone());

        let mut tokens: Vec<Token> = vec![];

        loop {
            if 0 == self.socket.read_buf(&mut buffer).await? {
                break;
            }

            loop {
                let mut cursor = Cursor::new(&buffer[..]);
                if let Ok(token) = decode(&mut cursor) {
                    let pos = cursor.position() as usize;
                    buffer.advance(pos);
                    tokens.push(token);

                    if buffer.is_empty() && tokens.len() > 0 {
                        let (command, consumed) = match Command::from_tokens(&tokens) {
                            Ok(x) => x,
                            Err(CommandError::InsufficientTokens) => continue,
                            Err(e) => {
                                return Err(std::io::Error::new(std::io::ErrorKind::Other, e))
                            }
                        };

                        if consumed < tokens.len() {
                            tokens = tokens.split_off(consumed);
                        } else {
                            tokens.clear();
                        }

                        let resp = cp.execute_command(&command).await;
                        for token in resp {
                            let mut write_buf: Vec<u8> = vec![];
                            encode(&mut write_buf, &token)
                                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?; // TODO: handle error

                            self.socket.write_all(&write_buf).await?;
                        }
                    }
                } else {
                    break;
                }
            }
        }

        Ok(())
    }
}

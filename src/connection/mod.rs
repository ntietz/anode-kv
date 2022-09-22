use bytes::{Buf, BytesMut};
use std::io::Cursor;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::codec::{decode, encode, Atom};

pub async fn accept_connection(mut socket: TcpStream, addr: SocketAddr) -> std::io::Result<()> {
    log::info!("Accepting connection from {}", addr);

    let mut buffer = BytesMut::with_capacity(4 * 1024);

    loop {
        if 0 == socket.read_buf(&mut buffer).await? {
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
                    let resp = Atom::Error("Not implemented yet".to_string());
                    let mut write_buf: Vec<u8> = vec![];
                    encode(&mut write_buf, &resp); // TODO: handle error

                    socket.write_all(&write_buf).await?;
                }
            } else {
                break;
            }

            println!("buf: {:?}", buffer);
        }
    }

    Ok(())
}

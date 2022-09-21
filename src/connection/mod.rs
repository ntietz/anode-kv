use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt};
use std::net::SocketAddr;
use std::io::Cursor;
use bytes::{Buf, BytesMut};

use crate::codec::decode;

pub async fn accept_connection(mut socket: TcpStream, addr: SocketAddr) -> std::io::Result<()> {
    log::info!("Accepting connection from {}", addr);

    let mut buffer = BytesMut::with_capacity(4*1024);

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
            } else {
                break;
            }

            println!("buf: {:?}", buffer);
        }
    }

    Ok(())
}

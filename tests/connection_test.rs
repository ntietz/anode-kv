use anode_kv::server::launch;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::Duration;

#[tokio::test]
async fn it_can_accept_connections() {
    let (local_addr, _handle) = launch("127.0.0.1:0").await.expect("should successfully launch");

    let addr = format!("127.0.0.1:{}", local_addr.port());

    let connection1 = connect_and_request(addr);
    connection1.await;
}

#[tokio::test]
async fn it_can_accept_multiple_connections() {
    let (local_addr, _handle) = launch("127.0.0.1:0").await.expect("should successfully launch");

    let addr = format!("127.0.0.1:{}", local_addr.port());

    let connection1 = tokio::spawn(connect_and_request(addr.clone()));
    let connection2 = tokio::spawn(connect_and_request(addr));
    let stream1 = connection1.await.expect("connection1 should finish");
    let stream2 = connection2.await.expect("connection2 should finish");

    // keep both connections alive until both are done reading
    drop(stream1);
    drop(stream2);
}


async fn connect_and_request(addr: String) -> TcpStream {
    let mut stream = tokio::net::TcpStream::connect(&addr)
        .await
        .expect("failed to connect to server");

    let command_string = b"$7\r\nCOMMAND\r\n";

    stream
        .write_all(command_string)
        .await
        .expect("failed write into stream");

    let expected_response = b"-ERR not implemented yet\r\n";
    let mut buffer = vec![0; expected_response.len()];

    let stream_read_promise = stream.read_exact(&mut buffer[..]);

    if let Err(_) = tokio::time::timeout(Duration::from_millis(100), stream_read_promise).await {
        panic!("response did not return within 100ms");
    }

    assert_eq!(buffer, expected_response);

    stream
}

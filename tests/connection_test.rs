use anode_kv::server::launch;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn it_can_accept_connections() {
    let addr = "127.0.0.1:11311"; // TODO: random port?

    let _server_handle = tokio::spawn(launch(addr));
    // naughty test, let's try to get rid of this later TODO
    sleep(Duration::from_millis(100)).await;

    let mut stream = tokio::net::TcpStream::connect(addr)
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
}

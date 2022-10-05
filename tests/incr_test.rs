use anode_kv::server::Server;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::Duration;

#[tokio::test]
async fn it_can_incr_and_decr_keys() {
    let mut server = Server::create("127.0.0.1:0").await.unwrap();
    let addr = server.addr();
    tokio::spawn(async move {
        server.run().await;
    });

    test_command_response(&addr, cmd_incr("x").as_bytes(), resp_bulk("1").as_bytes()).await;
    test_command_response(&addr, cmd_incr("x").as_bytes(), resp_bulk("2").as_bytes()).await;

    test_command_response(
        &addr,
        cmd_set("y", "3").as_bytes(),
        resp_simple("OK").as_bytes(),
    )
    .await;
    test_command_response(&addr, cmd_incr("y").as_bytes(), resp_bulk("4").as_bytes()).await;
    test_command_response(&addr, cmd_incr("y").as_bytes(), resp_bulk("5").as_bytes()).await;

    test_command_response(
        &addr,
        cmd_set("z", "-4").as_bytes(),
        resp_simple("OK").as_bytes(),
    )
    .await;
    test_command_response(&addr, cmd_decr("z").as_bytes(), resp_bulk("-5").as_bytes()).await;
    test_command_response(&addr, cmd_decr("z").as_bytes(), resp_bulk("-6").as_bytes()).await;

    test_command_response(&addr, cmd_decr("a").as_bytes(), resp_bulk("-1").as_bytes()).await;
    test_command_response(&addr, cmd_decr("a").as_bytes(), resp_bulk("-2").as_bytes()).await;
}

async fn test_command_response(addr: &str, command: &[u8], expected: &[u8]) {
    let mut stream = tokio::net::TcpStream::connect(addr)
        .await
        .expect("failed to connect to server");

    stream
        .write_all(command)
        .await
        .expect("failed write into stream");

    let mut buffer = vec![0; expected.len()];

    let stream_read_promise = stream.read_exact(&mut buffer[..]);

    if let Err(_) = tokio::time::timeout(Duration::from_millis(100), stream_read_promise).await {
        panic!("response did not return within 100ms");
    }

    assert_eq!(buffer, expected);
}

fn cmd_set(key: &str, value: &str) -> String {
    format!("*3\r\n+SET\r\n+{}\r\n+{}\r\n", key, value)
}

fn cmd_incr(key: &str) -> String {
    format!("*2\r\n+INCR\r\n+{}\r\n", key)
}

fn cmd_decr(key: &str) -> String {
    format!("*2\r\n+DECR\r\n+{}\r\n", key)
}

fn resp_simple(value: &str) -> String {
    format!("+{}\r\n", value)
}

fn resp_bulk(value: &str) -> String {
    format!("${}\r\n{}\r\n", value.len(), value)
}

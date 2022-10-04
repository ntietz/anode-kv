use anode_kv::server::Server;

#[tokio::main(flavor= "multi_thread", worker_threads=8)]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let addr = "127.0.0.1:11311";
    let mut server = Server::create(addr).await.expect("should launch");
    let handle = tokio::spawn(async move {
        server.run().await;
    });
    handle.await.expect("should shut down gracefully");
    Ok(())
}

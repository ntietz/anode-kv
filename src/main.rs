use anode_kv::server::launch;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let addr = "127.0.0.1:11311";
    let (_addr, handle) = launch(addr).await?;
    handle.await.expect("should shut down gracefully");
    Ok(())
}

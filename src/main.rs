use anode_kv::server::launch;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let addr = "127.0.0.1:11311";
    launch(addr).await?;
    Ok(())
}

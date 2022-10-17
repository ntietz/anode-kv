use anode_kv::config::Config;
use anode_kv::server::Server;
use clap::Parser;

fn main() {
    env_logger::init();
    let config: Config = Config::parse();
    println!("args: {:?}", config);

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(config.worker_threads)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let mut server = Server::create(config).await.expect("should launch");
            let handle = tokio::spawn(async move {
                server.run().await;
            });
            handle.await.expect("should shut down gracefully");
        });
}

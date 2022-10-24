use anode_kv::config::Config;
use anode_kv::server::Server;
use clap::Parser;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_span_events(FmtSpan::ACTIVE)
        .init();

    let config: Config = Config::parse();
    tracing::info!(config=?config, "Starting server");

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

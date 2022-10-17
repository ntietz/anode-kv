use clap::Parser;

#[derive(Debug, Parser, Clone)]
pub struct Config {
    // How many worker threads to use
    #[arg(short, long, default_value_t = 8)]
    pub worker_threads: usize,

    // Size channel for sending to the storage processor
    #[arg(long, default_value_t = 8)]
    pub storage_queue_size: usize,

    // Address to bind server to
    #[arg(short, long, default_value = "127.0.0.1:11311")]
    pub address: String,

    // Base filepath for durable storage
    #[arg(short, long, default_value = "./tmp/log")]
    pub storage_basepath: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            worker_threads: 8,
            storage_queue_size: 10,
            address: "127.0.0.1:11311".to_string(),
            storage_basepath: "./tmp/log".to_string(),
        }
    }
}

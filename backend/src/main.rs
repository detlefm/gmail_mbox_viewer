use backend::run_server;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting Rust backend (binary)...");

    // 1. Load Settings Path from args
    let args: Vec<String> = std::env::args().collect();
    let custom_settings_path = args.get(1).map(PathBuf::from);

    // 2. Define shutdown signal
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C signal handler");
        println!("Shutting down backend...");
    };

    // 3. Run Server
    run_server(custom_settings_path, None, shutdown_signal, None, None).await
}

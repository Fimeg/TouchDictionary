use touchdictionary_core::cli;

#[tokio::main]
async fn main() {
    // CLI mode only
    if let Err(e) = cli::run_cli().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

use rusty_web_development::{config, run, setup_store};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), handle_errors::Error> {
    // Handler for ENV variables
    dotenv::dotenv().ok();
    // Config object
    let config = config::Config::new().expect("Config cannot be set!");
    // DB Connection
    let store = setup_store(&config).await?;
    // Build ID
    info!("Q&A service build ID:{}", env!("RUSTY_WEB_DEV_VERSION"));
    // Run warp server
    run(config, store).await;

    Ok(())
}

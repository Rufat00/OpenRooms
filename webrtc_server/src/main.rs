use dotenv::dotenv;
use open_rooms::sfu::SFU;
use std::{env, sync::Arc};
use tokio::time::Duration;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let addr = format!("[::1]:{}", env::var("RPC_PORT").expect("RPC_PORT not set"));

    let sfu = Arc::new(SFU::default().unwrap());

    tokio::spawn(async move {
        let sfu_clone = Arc::clone(&sfu);
        sfu_clone
            .clean_empty_rooms(Duration::from_secs(
                env::var("ROOM_CLEAN_DURATION")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(120),
            ))
            .await;
    });

    // Server::builder().add_service().serve(addr).await?;

    Ok(())
}

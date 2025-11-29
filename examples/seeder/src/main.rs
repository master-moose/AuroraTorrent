use anyhow::Result;
use mainline::Dht;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting Seeder...");
    let _dht = Dht::client()?;
    println!("Seeder DHT initialized. Waiting for peers...");

    // Keep alive
    tokio::signal::ctrl_c().await?;
    Ok(())
}

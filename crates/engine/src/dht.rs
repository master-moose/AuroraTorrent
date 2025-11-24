use anyhow::Result;
use mainline::Dht;

pub async fn init_dht() -> Result<Dht> {
    // Initialize the Mainline DHT
    let dht = Dht::client()?;
    println!("DHT Initialized");
    Ok(dht)
}

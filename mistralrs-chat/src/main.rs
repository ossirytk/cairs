mod logic;
use anyhow::Result;
use logic::run_chat;

#[tokio::main]
async fn main() -> Result<()> {
    run_chat().await?;
    Ok(())
}

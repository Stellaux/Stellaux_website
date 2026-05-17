use stellaux_server::{common::bootstrap, server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = bootstrap::init().await?;
    server::run(state).await
}

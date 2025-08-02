use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info,orpheus=debug")
        .init();

    let client = Orpheus::from_env()?;

    let response = client
        .chat("hiii")
        .model("openai/gpt-4o")
        .send()?
        .into_content()?;

    tracing::info!("Response: {}", response);

    Ok(())
}

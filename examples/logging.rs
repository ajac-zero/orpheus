use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info,orpheus=debug")
        .init();

    let client = Orpheus::from_env()?;

    let response = client.respond("hiii").model("openai/gpt-4o-mini").send()?;

    if let Some(text) = response.output_text() {
        tracing::info!("Response: {}", text);
    }

    Ok(())
}

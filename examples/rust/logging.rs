#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! anyhow = "1.0.98"
//! tracing = "0.1.41"
//! tracing-subscriber = { version = "0.3.19", features = ["env-filter"] }
//! orpheus = "0.1.1"
//! ```
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

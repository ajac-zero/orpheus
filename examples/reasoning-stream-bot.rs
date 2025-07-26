use std::io::Write;

use colored::Colorize;
use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;

    let mut response = client
        .chat("Are zebras black with white stripes, or white with black stripes?")
        .model("google/gemini-2.5-flash-lite-preview-06-17")
        .with_reasoning(|reasoning| reasoning.effort(Effort::Low))
        .stream()?;

    println!("Reasoning tokens are in {}", "green".green().bold());

    while let Some(Ok(chunk)) = response.next() {
        if let Some(reasoning) = chunk.reasoning()? {
            print!("{}", reasoning.green());
            std::io::stdout().flush()?;
        }

        println!("{}", chunk.content()?);
        std::io::stdout().flush()?;
    }

    Ok(())
}

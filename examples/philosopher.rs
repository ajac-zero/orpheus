use std::io::Write;

use clap::Parser;
use colored::Colorize;
use orpheus::{models::Effort, prelude::*};

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    stream: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let client = Orpheus::from_env()?;

    let request = client
        .chat("Are zebras black with white stripes, or white with black stripes?")
        .model("google/gemini-2.5-flash-lite-preview-06-17")
        .with_reasoning(|reasoning| reasoning.effort(Effort::Low));

    if cli.stream {
        let mut response = request.stream()?;

        while let Some(Ok(chunk)) = response.next() {
            if let Some(reasoning) = chunk.reasoning()? {
                print!("{}", reasoning.green());
                std::io::stdout().flush()?;
            }

            println!("{}", chunk.content()?);
            std::io::stdout().flush()?;
        }
    } else {
        let response = request.send()?;

        if let Some(reasoning) = response.reasoning()? {
            println!("{}", "Reasoning:".green().bold());
            println!("{}", reasoning);
        }

        let content = response.content()?;
        println!("{}", "Response:".blue());
        println!("{}", content);
    }

    Ok(())
}

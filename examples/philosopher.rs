use std::io::Write;

use clap::Parser;
use colored::Colorize;
use orpheus::prelude::*;

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    stream: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let client = Orpheus::from_env()?;

    let input = "Are zebras black with white stripes, or white with black stripes?";

    if cli.stream {
        let stream = client
            .respond(input)
            .model("openai/gpt-4o-mini")
            .stream()?;

        for event in stream {
            let event = event?;
            if let Some(text) = event.as_text_delta() {
                print!("{}", text);
                std::io::stdout().flush()?;
            }
        }
        println!();
    } else {
        let response = client
            .respond(input)
            .model("openai/gpt-4o-mini")
            .send()?;

        if let Some(text) = response.output_text() {
            println!("{}", "Response:".blue());
            println!("{}", text);
        }
    }

    Ok(())
}

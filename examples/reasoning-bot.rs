use colored::Colorize;
use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;

    let response = client
        .chat("Are zebras black with white stripes, or white with black stripes?")
        .model("google/gemini-2.5-flash-lite-preview-06-17")
        .with_reasoning(|reasoning| reasoning.effort(Effort::Low))
        .send()?;

    if let Some(reasoning) = response.reasoning()? {
        println!("{}", "Reasoning:".green().bold());
        println!("{}", reasoning);
    }

    let content = response.content()?;
    println!("{}", "Response:".blue());
    println!("{}", content);

    Ok(())
}

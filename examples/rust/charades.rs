#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! anyhow = "1.0.98"
//! colored = "3.0.0"
//! rand = "0.9.1"
//! serde = "1.0.217"
//! serde_json = "1.0.134"
//! orpheus = "0.1.1"
//! ```
use colored::Colorize;
use orpheus::prelude::*;
use rand::seq::IndexedRandom;

// Config
const PROMPT: &str = "You are a charades game master.
Choose one animal, and the user must guess what it is.
The user is only allowed three guesses.
Before every guess, chat a riddle about your animal.
If the user guesses correctly withint those three guesses, they win.
If the user runs out of guesses, you win.
Use the 'player_win' tool if the player guesses correctly.
Use the 'game_over' tool if the player runs out of guesses.
Don't hold back.";

const MODELS: [&str; 5] = [
    "google/gemini-2.0-flash-001",
    "anthropic/claude-3.5-haiku",
    "openai/gpt-4o-mini",
    "x-ai/grok-3-mini-beta",
    "mistralai/mistral-small-3.2-24b-instruct",
];

#[derive(serde::Deserialize)]
struct GameOverArgs {
    answer: String,
}

// Logic
fn main() -> anyhow::Result<()> {
    // Start orpheus client from environment variables
    let orpheus = Orpheus::from_env()?;

    // Choose a random model from the list
    let mut rng = rand::rng();
    let model = *MODELS.choose(&mut rng).expect("is not empty");
    println!("Using model: {}", model.yellow());

    // Make the messages vec, starting with the system prompt
    let mut messages = vec![Message::system(PROMPT)];

    // Define the game control tools
    let tools = vec![
        Tool::function("player_win").empty(),
        Tool::function("game_over")
            .with_parameters(|p| p.property("answer", Param::string().end()))
            .build(),
    ];

    // Start infinite loop that will only stop once we win or lose
    loop {
        // Call the llm
        let response = orpheus
            .chat(messages.clone())
            .model(model)
            .tools(tools.clone())
            .temperature(0.9) // For more creative responses
            .send()?;

        // Check if the llm used a tool
        if let Some(ToolCall::Function { function, .. }) = response.tool_call()? {
            if function.name == "player_win" {
                println!("{}", FINISH_BANNER.green().bold())
            }

            if function.name == "game_over" {
                println!("{}", GAME_OVER_BANNER.red().bold());

                let args: GameOverArgs = serde_json::from_str(&function.arguments)?;
                println!("Real answer: {}", args.answer);
            }

            // Exit the loop
            return Ok(());
        }

        // Get a reference to the content from the llm response
        let content = response.content()?;

        // Print the llm response content
        println!("{}", "Game Master ================".yellow());
        println!("{}", content);
        // Add the llm response content to the conversation array
        messages.push(response.into_message()?);

        // Ask the user for input
        println!("{}", "Answer =====================".blue());
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        println!();
        // Add the user input to the conversation array
        messages.push(Message::user(input));
    }
}

// Assets
const FINISH_BANNER: &str = "
██╗   ██╗ ██████╗ ██╗   ██╗    ██╗    ██╗ ██████╗ ███╗   ██╗    ██╗██╗██╗
╚██╗ ██╔╝██╔═══██╗██║   ██║    ██║    ██║██╔═══██╗████╗  ██║    ██║██║██║
 ╚████╔╝ ██║   ██║██║   ██║    ██║ █╗ ██║██║   ██║██╔██╗ ██║    ██║██║██║
  ╚██╔╝  ██║   ██║██║   ██║    ██║███╗██║██║   ██║██║╚██╗██║    ╚═╝╚═╝╚═╝
   ██║   ╚██████╔╝╚██████╔╝    ╚███╔███╔╝╚██████╔╝██║ ╚████║    ██╗██╗██╗
   ╚═╝    ╚═════╝  ╚═════╝      ╚══╝╚══╝  ╚═════╝ ╚═╝  ╚═══╝    ╚═╝╚═╝╚═╝
";

const GAME_OVER_BANNER: &str = "
 ██████╗  █████╗ ███╗   ███╗███████╗     ██████╗ ██╗   ██╗███████╗██████╗          ██╗
██╔════╝ ██╔══██╗████╗ ████║██╔════╝    ██╔═══██╗██║   ██║██╔════╝██╔══██╗    ██╗ ██╔╝
██║  ███╗███████║██╔████╔██║█████╗      ██║   ██║██║   ██║█████╗  ██████╔╝    ╚═╝██╔╝
██║   ██║██╔══██║██║╚██╔╝██║██╔══╝      ██║   ██║╚██╗ ██╔╝██╔══╝  ██╔══██╗    ██╗╚██╗
╚██████╔╝██║  ██║██║ ╚═╝ ██║███████╗    ╚██████╔╝ ╚████╔╝ ███████╗██║  ██║    ╚═╝ ╚██╗
╚═════╝ ╚═╝  ╚═╝╚═╝     ╚═╝╚══════╝     ╚═════╝   ╚═══╝  ╚══════╝╚═╝  ╚═╝         ╚═╝
";

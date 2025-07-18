use std::io::{self, Write};

use colored::Colorize;
use orpheus::{AsyncOrpheus, Message, ToolCall, mcp::ModelContext};

#[tokio::main]
async fn main() -> orpheus::Result<()> {
    let client = AsyncOrpheus::from_env()?;

    // Connect to a server running as a child process
    let context = ModelContext::builder()
        .command("npx")
        .args(["-y", "@modelcontextprotocol/server-filesystem", "."])
        .run()
        .await?;

    let tools = context.get_tools().await?;

    // Welcome message
    println!("{}", "🤖 Filesystem Assistant Chat".bold().cyan());
    println!(
        "{}",
        "Connected to filesystem tools. Ask me anything about your files!".dimmed()
    );
    println!(
        "{}",
        "Commands: /bye (exit), /help (show commands)".dimmed()
    );
    println!("{}", "═".repeat(50).dimmed());

    let mut messages = vec![Message::system(
        "You are a filesystem assistant. Help the user with any questions they might have about the files available to you. When the conversation begins, use the list available directories tool to check the path you can work on and answer questions about.",
    )];

    loop {
        loop {
            print!("{}", "🤔 Thinking...".yellow().dimmed());
            io::stdout().flush().unwrap();

            let response = client
                .chat(messages.clone())
                .model("google/gemini-2.0-flash-001")
                .tools(tools.clone())
                .send()
                .await?;

            print!("\r{}", " ".repeat(20)); // Clear the thinking message
            print!("\r");
            io::stdout().flush().unwrap();

            let message = response.message()?.clone();
            messages.push(message.clone());

            if let Some(ToolCall::Function { id, function }) = response.tool_call()? {
                println!(
                    "{} {}",
                    "🔧".yellow(),
                    format!("Using tool: {}", function.name).dimmed()
                );

                let tool_message = context
                    .call(&function.name)
                    .literal_arguments(&function.arguments)?
                    .send()
                    .await?
                    .into_message(id);

                messages.push(tool_message);

                continue;
            }

            println!("{}", "\nAssistant:".green());
            println!("{}", message.content);

            break;
        }

        print!("\n{} ", "❯".blue().bold());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if let Err(e) = io::stdin().read_line(&mut input) {
            println!("{} Failed to read input: {}", "❌".red(), e);
            continue;
        }

        let trimmed_input = input.trim();
        match trimmed_input {
            "/bye" => break,
            "/help" => {
                println!("{}", "Available commands:".bold());
                println!("  {} - Exit the chat", "/bye".cyan());
                println!("  {} - Show this help", "/help".cyan());
                continue;
            }
            _ => {}
        }

        messages.push(Message::user(input));
    }

    // Clean exit
    println!("{}", "\n👋 Goodbye!".cyan());
    context.close().await?;
    Ok(())
}

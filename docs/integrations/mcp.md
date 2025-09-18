---
icon: plug
---

# MCP

The [Model Context Protocol](https://modelcontextprotocol.io/docs/getting-started/intro) is the leading mechanism via which LLMs can interact with external systems. If you are building an AI application, more likely than not, you will want to use MCP.

Orpheus includes integrated support for MCPs via the `mcp` feature.

```bash
cargo add orpheus -F mcp
```

> NOTE: The MCP feature is a work-in-progress, and currently only local MCPs via stdio are supported.

### Quickstart

As an example, let's make a chat CLI that uses a filesystem MCP to provide information about the Orpheus repo.

{% code title="filesystem_mcp_chat.rs" %}
```rust
use std::io::{self, Write};

use orpheus::{mcp::Mcp, prelude::*};

#[tokio::main]
async fn main() -> orpheus::Result<()> {
    // MCP requires tokio, you have to use the async client!
    let client = AsyncOrpheus::from_env()?;

    // Connect to a server running as a child process
    let fs_mcp = Mcp::stdio()
        .command("npx")
        .args(["-y", "@modelcontextprotocol/server-filesystem", "."])
        .run()
        .await?;

    // Fetch tool schemas as Orpheus-compatible tool objects
    let tools = fs_mcp.get_tools().await?;

    // Welcome message
    println!("🤖 Filesystem Assistant Chat");
    println!("Connected to filesystem tools. Ask me anything about your files!");
    println!("Commands: /bye (exit), /help (show commands)");

    let mut messages = vec![Message::system(
        "You are a filesystem assistant. Help the user with any questions they might have about the files available to you. When the conversation begins, use the list available directories tool to check the path you can work on and answer questions about.",
    )];

    // Start the chat loop
    loop {
        // Start the agent loop
        loop {
            print!("🤔 Thinking...");
            io::stdout().flush().unwrap();

            let response = client
                .chat(messages.clone())
                .model("google/gemini-2.5-flash")
                .tools(tools.clone()) // Add mcp tools to chat request
                .send()
                .await?;

            print!("\r{}", " ".repeat(20)); // Clear the thinking message
            print!("\r");
            io::stdout().flush().unwrap();

            let message = response.message()?.clone();
            messages.push(message.clone());

            // Check if agent called a tool
            if let Some(ToolCall::Function { id, function }) = response.tool_call()? {
                println!("🔧 Using tool: {}", function.name);

                // Call the tool remotely through the MCP
                let tool_message = fs_mcp
                    .call(&function.name) // Call function by name
                    .literal_arguments(&function.arguments)? // Pass JSON args from the model response
                    .send()
                    .await?
                    .into_message(id); // Turn result into a `tool` message with related call id

                messages.push(tool_message);

                continue;
            }

            println!("\nAssistant:\n{}", message.content);

            break;
        }

        print!("\n❯ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if let Err(e) = io::stdin().read_line(&mut input) {
            println!("❌ Failed to read input: {}", e);
            continue;
        }

        let trimmed_input = input.trim();
        match trimmed_input {
            "/bye" => break,
            "/help" => {
                println!("Available commands:");
                println!("  /bye - Exit the chat");
                println!("  /help - Show this help");
                continue;
            }
            _ => {}
        }

        messages.push(Message::user(input));
    }

    // Clean exit
    println!("{}", "\n👋 Goodbye!");
    fs_mcp.close().await?;
    Ok(())
}
```
{% endcode %}

{% code title="output" %}
```
Secure MCP Filesystem Server running on stdio
Client does not support MCP Roots, using allowed directories set from server args: [ '/Users/ajac-zero/Developer/Rustonomicon/orpheus' ]
🤖 Filesystem Assistant Chat
Connected to filesystem tools. Ask me anything about your files!
Commands: /bye (exit), /help (show commands)
🔧 Using tool: list_allowed_directories

Assistant:
I have access to the `/Users/ajac-zero/Developer/Rustonomicon/orpheus` directory. How can I help you?

❯ What is the name of this project and who made it?
🔧 Using tool: list_directory

Assistant:
The name of the project appears to be `orpheus`. I can try to find information about the author in files like `README.md`, `Cargo.toml`, or `LICENSE`. Which file would you like me to check first?

❯ the toml
🔧 Using tool: read_text_file

Assistant:
The `Cargo.toml` file indicates the project name is "orpheus" and it's a "blazing fast OpenRouter SDK".

While it lists a `repository` URL (`https://github.com/ajac-zero/orpheus`), it doesn't explicitly state the author's personal name within this file. However, "ajac-zero" is likely the GitHub username of the creator or primary maintainer.
```
{% endcode %}

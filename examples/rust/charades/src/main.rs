use colored::Colorize;
use orpheus::{
    Orpheus,
    models::chat::{ChatMessage, Tool, ToolCall},
};

// Config
const PROMPT: &str = "You are a charades game master.
Choose an animal, and help the user guess what it is.
When the user guesses correctly, use the end game tool.
Give clues only in riddles.
Only allow the user three guesses before using the game over tool.";

// Logic
fn main() -> anyhow::Result<()> {
    // Set an OpenRouter model ID
    let model = "google/gemini-2.0-flash-001";

    // Start orpheus router from environment variables
    let orpheus = Orpheus::from_env()?;

    // Define the game tools with the builder pattern
    // These help the llm control the flow of the program
    // Since no parameters are needed, we can just provide the names of the tools
    // NOTE: Not all providers accept tools without parameters
    let end_game_tool = Tool::function("end_game").build();
    let game_over_tool = Tool::function("game_over").build();
    let tools = Vec::from([end_game_tool, game_over_tool]);

    // Start the conversation array, starting with the system prompt
    // Let's make this mutable as we will update it on every turn.
    let system_message = ChatMessage::system(PROMPT);
    let mut messages = Vec::from([system_message]);

    // Start infinite loop that will only stop once we win or lose
    loop {
        // Call the llm with the given models, messages, and tools
        let response = orpheus
            .chat()
            .model(model)
            .messages(messages.clone())
            .tools(tools.clone())
            .temperature(2.0) // Set temperature to 2.0 for more creative responses
            .send()?;

        // Check if the llm used a tool
        if let Some(ToolCall::Function { function, .. }) = response.tool_call()? {
            // If llm calls this tool, the player won
            if function.is("end_game") {
                // Print win message
                println!("{}", FINISH_BANNER.green().bold());
            }

            // If llm calls this tool, the player lost
            if function.is("game_over") {
                // Print game over message
                println!("{}", GAME_OVER_BANNER.red().bold());
            }

            // Either way, the game is over so let's exit the loop
            return Ok(());
        }

        // Extract the content from the llm response
        let content = response.into_content()?;

        // Print the llm response content
        println!("{}", "Game Master ================".yellow());
        println!("{}", content);
        // Add the llm response content to the conversation array
        messages.push(ChatMessage::assistant(content));

        // Ask the user for input
        println!("{}", "Answer =====================".blue());
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        println!();
        // Add the user input to the conversation array
        messages.push(ChatMessage::user(input));
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

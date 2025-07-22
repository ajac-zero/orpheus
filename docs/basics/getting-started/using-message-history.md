# Using message history

Simple, right? Let's take it up a notch by making the CLI program interactive and adding a conversation history so the model can remember our previous messages.

We also probably don't want to hardcode our API key into the program, so let's initialize our client from environment variables instead.

#### With Message History Example

```rust
use orpheus::prelude::*;

fn main() {
    // Start the client by reading the key in the ORPHEUS_API_KEY environment variable
    let client = Orpheus::from_env().expect("ORPHEUS_API_KEY is set");
    
    // Create a vector that we will continually update with our message history.
    let mut messages = Vec::new();
    
    loop {
        // Boilerplate to read user input from the terminal into a variable
        let mut user_input = String::new();
        println!("User:");
        std::io::stdin().read_line(&mut user_input).unwrap();
        
        // Let's turn our user input into a proper message and add it to our message history
        messages.push(Message::user(user_input));

        let response = client
            .chat(&messages) // The chat method accepts our list of messages directly
            .model("openai/gpt-4o")
            .send()
            .unwrap();

        // The response from the model can be turned into a message in the same format as the user message.
        let ai_message = response.into_message().unwrap();

        println!("Assistant:");
        println!("{}", ai_message.content);

        // Add the response message to our list
        messages.push(ai_message);
    }
}
```

Output:

```
User:
hi!
Assistant:
Hello! How can I assist you today?
User:
who are you?
Assistant:
I'm an AI language model created by OpenAI, designed to assist with a wide range of inquiries by providing information, answering questions, and engaging in conversation. How can I help you today?
User:
that's awesome
Assistant:
Thank you! If there's anything specific you’d like to know or discuss, feel free to ask.
```

# Streaming responses

In AI apps, it is common to stream the response to reduce the perceived latency of the program. Let's see how we can use response streaming with Orpheus.

#### Streaming Response Example

```rust
use std::io::Write;

use orpheus::prelude::*;

fn main() {
    let client = Orpheus::from_env().expect("ORPHEUS_API_KEY is set");

    let mut messages = Vec::new();

    loop {
        let mut user_input = String::new();
        println!("User:");
        std::io::stdin().read_line(&mut user_input).unwrap();

        messages.push(Message::user(user_input));

        let mut response = client
            .chat(&messages)
            .model("openai/gpt-4o")
            .stream() // By calling `stream` instead of `send`, we get an iterable over the response chunks
            .unwrap();

        // Create a buffer that we will continuously update with the content of each chunk
        let mut buffer = String::new();

        println!("Assistant:");
        // Loop until the iterator runs out of chunks
        while let Some(Ok(chunk)) = response.next() {
            // Get the content of the chunk and add it to the buffer
            let content = chunk.content().unwrap();
            buffer.push_str(content);

            // Boilerplate to print the response as it comes in
            print!("{}", content);
            std::io::stdout().flush().unwrap();
        }
        println!();

        // Add the completed buffer to the message history
        messages.push(Message::assistant(buffer));
    }
}
```

```
>> User:
>> hi
>> Assistant:
>> Hello! How can I assist you today?
```

Note: You'll have to run it yourself to see the stream effect

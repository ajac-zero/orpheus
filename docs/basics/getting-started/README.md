---
icon: person-chalkboard
---

# Getting Started

## Installation

Add Orpheus to your project with cargo:

```bash
cargo add orpheus
```

## Basic Usage

Let's learn how to use Orpheus with a practical example. Here, we will create a CLI program that allows us to send a chat request to an LLM.

#### One-shot Prompt Example

```rust
// The prelude includes everything you will need to use Orpheus  
use orpheus::prelude::*;

fn main() {
    // First, we have to start our central client
    let client = Orpheus::new("Your-API-Key");
    
    // With our client, we can call the `chat` method to begin a chat completion request
    // The request follows a builder pattern, iteratively adding arguments before finally sending it with `send`
    let response = client
        .chat("Hello!") // The chat method takes your prompt as an initial argument
        .model("openai/gpt-4o") // Select a model by passing an OpenRouter model ID
        .send() // Finally, send the request with the arguments set thus far to the model
        .unwrap();
        
    // Get the content of the response (if any) and print it to the console
    let content = response.content().unwrap();
    println!("GPT-4o says: {}", content);
}
```

Output:

```
GPT-4o says: Hello! How can I assist you today?
```

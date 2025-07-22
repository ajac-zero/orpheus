---
description: Learn the basic interface of tool calling within Orpheus.
---

# Tool Calling

For more advanced use cases, you will probably want to give the LLM a way to interact with its environment. Tool calling enables this by specifying a set of actions the model can take.

Orpheus supports tool calling via the `Tool` object builder, which you can then pass to the `tools` parameter of your request.

The response will then, if the model decides to use a tool, hold a vector of `ToolCall` objects that we can then use within our code.&#x20;

```rust
use orpheus::prelude::*;

fn main() {
    let client = Orpheus::from_env().unwrap();

    // Define a tool
    // For this example, it's just an empty tool we can use as a "switch".
    let my_tool = Tool::function("my_tool").empty();

    let res = client
        .chat("Call my tool")
        .model("openai/gpt-4o")
        .tools([my_tool]) // Pass an iterable of tools to the request
        .send()
        .unwrap();

    // `tool_calls` is a convenience method to
    // access tool calls in the response, if any
    if let Some(tool_calls) = res.tool_calls().unwrap() {
        for tool_call in tool_calls {
            println!("Tool call: {:?}", tool_call);
        }
    }
}
```

Output:

```
Tool call: Function { id: "call_dTmBXTw3BXhIbCL5r2dVvnnU", function: Function { name: "my_tool", arguments: "{}" } }
```

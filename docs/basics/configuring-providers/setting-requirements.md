# Setting requirements

Not all providers support every feature. Some might not support tool calling, others structured output, or file uploads.
It's a good practice to ensure that you only send requests to providers that support all the features your request requires.
To do this, just make sure to set the `require_parameters` parameter to `true` in your preferences.

```rust
use orpheus::prelude::*;

fn main() {
    let client = Orpheus::from_env().unwrap();

    let calculator_tool = Tool::function("calculator")
        .description("Do an operation on two items")
        .with_parameters(|params| {
            params
                .property("x", Param::number().end())
                .property("y", Param::number().end())
                .property("op", Param::string().r#enum(["+", "-", "/", "*"]).end())
                .required(["x", "y", "op"])
        })
        .build();

    let res = client
        .chat("Who is the greatest general of all time?")
        .model("qwen/qwen3-32b")
        .tools([calculator_tool])
        .with_preferences(|pref| pref.require_parameters(true)) // Will only send to providers that support tool calls
        .send()
        .unwrap();

    println!("Model says: {}", res.content().unwrap());
}
```

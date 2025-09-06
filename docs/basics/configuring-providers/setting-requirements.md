# Setting requirements

Not all providers support every feature. Some might not support tool calling, others structured output, or file uploads. It's a good practice to ensure that you only send requests to providers that support all the features your request requires. To do this, just make sure to set the `require_parameters` parameter to `true` in your preferences.

{% code title="require_param.rs" %}
```rust
use orpheus::prelude::*;

fn main() {
    let client = Orpheus::from_env().unwrap();

    let calculator_tool = Tool::function("calculator")
        .description("Do an operation on two items")
        .with_parameters(|params| {
            params
                .property("x", Param::number())
                .property("y", Param::number())
                .property("op", Param::string().enums(["+", "-", "/", "*"]))
                .required(["x", "y", "op"])
        })
        .build();

    let res = client
        .chat("What is 42 + 39?")
        .model("qwen/qwen3-32b")
        .tools([calculator_tool])
        .with_preferences(|pref| pref.require_parameters(true)) // Will only send to providers that support tool calls
        .send()
        .unwrap();

    println!("Model says: {}", res.content().unwrap());
}
```
{% endcode %}

```
Model says:
Okay, the user is asking what is 42 plus 39. Let me see. I need to use the calculator function here. The function requires two numbers, x and y, and an operation op. The operation can be +, -, /, or *. In this case, the user is adding 42 and 39, so op should be "+". I'll plug those values into the function. Let me double-check: x is 42, y is 39, op is "+". That should give the correct result. I think that's all. Let me make sure there's no other operations involved. Nope, just addition. So the tool call should be straightforward.
```

# Model Routers

There is a set of special model IDs that, instead of calling a model directly, actually call a "model router":

1. [Auto Router](https://openrouter.ai/openrouter/auto), powered by [NotDiamond](https://www.notdiamond.ai/).
2. [Switchpoint Router](https://openrouter.ai/switchpoint/router), powered by [Switchpoint AI](https://www.switchpoint.dev/).

These model routers will automatically call the best model for the request. You can check the model that was chosen by the model router in the `model` field of the response object.

```rust
use orpheus::prelude::*;

fn main() {
    let client = Orpheus::new("Your-API-Key");
    
    let res = client
        .chat("Who is the president of Tasmania?")
        .model("openrouter/auto")
        .send()
        .unwrap();
    println!("Model picked: {}", res.model);
    println!("Response: {}", res.content().unwrap());
}
```

Output:

```
Model picked: openai/chatgpt-4o-latest
Response: Tasmania is not an independent country, so it does not have a president. It is a state of Australia. The head of government in Tasmania is the **Premier**, not a president.
```

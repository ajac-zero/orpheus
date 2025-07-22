# Allow/Deny data collection

If data privacy is a concern, you can disable all providers that collect/train on your data.
You just have to set the `data_collection` parameter in preferences to false.

```rust
use orpheus::prelude::*;

fn main() {
    let client = Orpheus::from_env().unwrap();

    let res = client
        .chat("Do you train on user inputs?")
        .model("moonshotai/kimi-k2")
        .with_preferences(|pref| pref.data_collection(false)) // false means deny; true means allow
        .send()
        .unwrap();

    println!("Provider used: {}", res.provider);
    println!("Model says: {}", res.content().unwrap());
}
```

Output:

```
Provider used: BaseTen
Model says: No, I don't. I retain and reuse only what is needed to keep our current conversation coherent. Your individual messages are not stored, added to any permanent training set, or otherwise used to improve the model behind the scenes.
```

# Latency vs Price vs Throughput

You can explicitly prioritize a specific attribute to disable load-balancing and target providers that best suit your needs. The three attributes you can target are:

1. `Latency` sorts providers by the lowest latency
2. `Price` sorts providers by the lowest price
3. `Throughput` sorts providers by the highest throughput

#### Example

```rust
use orpheus::prelude::*;

fn main() {
    let client = Orpheus::new("Your-API-Key");
    
    let prompt = "What is 23 + 47?";
    let model = "moonshotai/kimi-k2";

    let res = client
        .chat(prompt)
        .model(model)
        .with_preferences(|pref| pref.sort(Sort::Latency))
        .send()
        .unwrap();
    println!("Provider picked: {}", res.provider);
    // Provider picked: DeepInfra

    let res = client
        .chat(prompt)
        .model(model)
        .with_preferences(|pref| pref.sort(Sort::Price))
        .send()
        .unwrap();
    println!("Provider picked: {}", res.provider);
    // Provider picked: Targon

    let res = client
        .chat(prompt)
        .model(model)
        .with_preferences(|pref| pref.sort(Sort::Throughput))
        .send()
        .unwrap();
    println!("Provider picked: {}", res.provider);
    // Provider picked: Groq
}
```

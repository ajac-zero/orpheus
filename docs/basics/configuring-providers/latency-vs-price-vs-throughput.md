# Latency vs Price vs Throughput

You can explicitly prioritize a specific attribute to disable load-balancing and target providers that best suit your needs. The three attributes you can target are:

1. `Latency` sorts providers by the lowest latency
2. `Price` sorts providers by the lowest price
3. `Throughput` sorts providers by the highest throughput

{% code title="set_priority.rs" %}
```rust
use orpheus::prelude::*;

fn main() {
    let client = Orpheus::new("Your-API-Key");

    let prompt = "What is 23 + 47?";
    let model = "moonshotai/kimi-k2";

    for priority in [Sort::Latency, Sort::Price, Sort::Throughput] {
        let res = client
            .chat(prompt)
            .model(model)
            .with_preferences(|pref| pref.sort(priority))
            .send()
            .unwrap();

        println!(
            "Provider picked with priority '{:?}': {}",
            priority, res.provider
        );
    }
}
```
{% endcode %}

```bash
Provider picked with priority 'Latency': DeepInfra
Provider picked with priority 'Price': Targon
Provider picked with priority 'Throughput': Groq
```

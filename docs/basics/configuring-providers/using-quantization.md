# Using quantization

Quantization can help both reduce cost and latency, albeit with the drawback of losing some precision in generations.

Regardless, quantization can be a very useful tool if your use case allows it.

You can select allowed levels of quantization for models with the `quantization` parameter in the preferences object, which accepts an iterable of `Quantization` objects.

The valid quantization variants are as follows:
- int4: Integer (4 bit)
- int8: Integer (8 bit)
- fp4: Floating point (4 bit)
- fp6: Floating point (6 bit)
- fp8: Floating point (8 bit)
- fp16: Floating point (16 bit)
- bf16: Brain floating point (16 bit)
- fp32: Floating point (32 bit)

If no provider is offering the requested model at the select quantization level, an error will be returned.

```rust
use orpheus::prelude::*;

fn main() {
    let client = Orpheus::from_env().unwrap();

    let res = client
        .chat("Who is the greatest general of all time?")
        .model("qwen/qwen3-32b")
        .with_preferences(|pref| pref.quantizations([Quantization::Fp8])) // allows only providers that have 8 bit float quants
        .send()
        .unwrap();

    println!("Model says: {}", res.content().unwrap());
}
```

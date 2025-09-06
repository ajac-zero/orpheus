---
icon: image-polaroid
---

# Multimodality

Orpheus supports multimodal inputs, allowing you to include images, files, and audio alongside text in your messages.

## Basic Usage

### Image Input

OpenRouter supports both **direct URLs** and **base64-encoded data** for images:

* **URLs**: More efficient for publicly accessible images as they don’t require local encoding
* **Base64**: Required for local files or private images that aren’t publicly accessible

Supported image content types are: `png`, `jpeg`, `webp`, `gif`.

{% code title="image_input.rs" %}
```rust
use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;

    let image_url = "https://misanimales.com/wp-content/uploads/2022/03/Shih-Poo-Shih-Tzu-1024x680-1-768x510.jpg";

    let message = Message::user("Describe this image").with_image(image_url);

    let response = client.chat(message).model("openai/gpt-4o").send()?;

    println!("{}", response.content()?);
    Ok(())
}
```
{% endcode %}

```
The image features a small dog with a fluffy coat, primarily white with brown markings. The dog's fur is particularly fluffy around its face, and it has a cute, expressive face with dark eyes. The background appears to be an indoor setting, possibly a living room, with furniture visible but out of focus.
```

The None argument in `with_image` states the&#x20;

### File Input

Attach files with a filename and content data.

OpenRouter supports both **direct URLs** and **base64-encoded data** for files:

* **URLs**: More efficient for publicly accessible files as they don’t require local encoding
* **Base64**: Required for local or private files that aren’t publicly accessible

Supported image content types are: `pdf`.

{% code title="file_input.rs" %}
```rust
use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;

    let pdf_url = "https://bitcoin.org/bitcoin.pdf";

    let message = Message::user("What are the main points in this document?")
        .with_file("bitcoin.pdf", pdf_url);

    let response = client.chat(message).model("openai/gpt-4o").send()?;

    println!("{}", response.content()?);
    Ok(())
}
```
{% endcode %}

```
The document outlines a peer-to-peer electronic cash system called Bitcoin. Here are the main points:

1. **Introduction**:
   - Proposes an electronic payment system that allows transactions directly between parties without a trusted third party.
   - Digital signatures are used to prevent double-spending, relying on a peer-to-peer network.

2. **Transactions**:
   - Describes how electronic coins are transferred and verified using digital signatures.
   - Eliminates the need for a central authority by making all transactions public, with the longest chain of transactions being the most trusted.

3. **Timestamp Server**:
   - Uses a distributed system to prevent double-spending through a timestamp server that hashes transactions into a chain.

4. **Proof-of-Work**:
   - Introduces proof-of-work to secure the network, ensuring that the longest chain is the valid one. Attacks are prevented as long as a majority of CPU power is honest.

5. **Network**:
   - Describes the steps to run the network, emphasizing broadcasting transactions and the creation of blocks.

6. **Incentive**:
   - Incentivizes nodes by rewarding them with new coins or transaction fees for supporting the network.

7. **Reclaiming Disk Space**:
   - Utilizes Merkle Trees to save disk space by allowing old transaction data to be discarded.

8. **Simplified Payment Verification**:
   - Allows users to verify transactions without running a full network node by maintaining a copy of the longest chain's block headers.

9. **Combining and Splitting Value**:
   - Explains how transactions can handle multiple inputs and outputs, facilitating the transfer of varied values.

10. **Privacy**:
    - Maintains privacy by keeping public keys anonymous and using a new key pair for each transaction.

11. **Calculations**:
    - Analyzes the probability of an attacker's success in altering the transaction chain, diminishing as more blocks are added.

12. **Conclusion**:
    - The system is proposed as a new method for electronic transactions without trust, relying on consensus and proof-of-work to maintain security and integrity.

This system outlines the foundation of what would become Bitcoin, focusing on security, decentralization, and privacy.
```

### Audio Input

Include audio content with base64-encoded data and format specification. You can search for models that support audio [here](https://openrouter.ai/models?fmt=cards\&input_modalities=audio).

Supported image content types are: `wav`, `mp3`.

> **Note**: Audio files must be **base64-encoded** - direct URLs are not supported for audio content.

{% code title="audio_input.rs" %}
```rust
use base64::prelude::*;
use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;

    // Read audio file from disk
    let audio_path = "audio_sample.wav"; // Change this to your audio file path
    let audio_bytes = std::fs::read(audio_path)?;
    let audio_data = BASE64_STANDARD.encode(&audio_bytes);

    // Determine file extension from path
    let extension = audio_path.split('.').last().unwrap_or("wav");

    let message =
        Message::user("What do you hear in this audio?").with_audio(audio_data, extension);

    let response = client
        .chat(message)
        .model("google/gemini-2.5-flash-lite")
        .send()?;

    println!("{}", response.content()?);
    Ok(())
}
```
{% endcode %}

```
The audio contains a repeating beep sound.
```

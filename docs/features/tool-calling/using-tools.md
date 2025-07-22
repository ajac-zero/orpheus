# Tool Calling

For more advanced use cases, you will probably want to give the LLM a way to interact with its environment. Tool calling enables this by specifying a set of actions the model can take.

#### Model with Tools Example

> Note: To run this example you'll need to add these crates:
>
> * `cargo add reqwest -F blocking`&#x20;
> * `cargo add serde`
> * `cargo add serde_json`
> * `cargo add orpheus -F anyhow`

```rust
use orpheus::prelude::*;
use reqwest::blocking::get;
use serde::Deserialize;
use serde_json::{Value, from_str, to_string};

#[derive(Deserialize)]
struct Coordinates {
    latitude: String,
    longitude: String,
}

#[derive(Deserialize)]
struct GetWeather {
    location: Coordinates,
    units: String,
}

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;

    // Define your tool
    let get_weather_tool = Tool::function("get_weather")
        .description("Retrieve current weather for the given location.")
        .with_parameters(|params| {
            params
                .property(
                    "location",
                    Param::object()
                        .description("Coordinates of the location")
                        .property("latitude", Param::string().end())
                        .property("longitude", Param::string().end())
                        .required(["latitude", "longitude"])
                        .end(),
                )
                .property(
                    "units",
                    Param::string()
                        .description("Units the temperature will be returned in.")
                        .r#enum(["celsius", "fahrenheit"])
                        .end(),
                )
                .required(["location", "units"])
        })
        .build();

    let prompt = "How's the weather in New York?";
    println!("Prompt: {}", prompt);

    let mut messages = vec![
        Message::system(
            "You are a weather bot. You can assume coordinates if not provided by the user. Default to celsius.",
        ),
        Message::user(prompt),
    ];

    let res = client
        .chat(&messages)
        .model("anthropic/claude-sonnet-4")
        .tools([get_weather_tool]) // Add your tools to the chat request
        .send()?;

    messages.push(res.message()?.clone());

    // `tool_call` is a convenience method to extract the first function call in a response, if is some
    if let Some(ToolCall::Function { id, function }) = res.tool_call()? {
        println!("Tool function used: {}", function.name);

        // The name field can be used to route to the correct logic
        if function.name == "get_weather" {
            // The arguments field holds a JSON string following the schema, so we can deserializize it with serde
            let args: GetWeather = from_str(&function.arguments)?;

            // Inner tool logic that uses function arguments
            let request_url = format!(
                "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m&temperature_unit={}",
                args.location.latitude, args.location.longitude, args.units
            );
            let weather_data: Value = get(request_url)?.json()?;
            let content = to_string(&weather_data)?;

            // We can then turn the tool result data into a message so the model can see it
            messages.push(Message::tool(id, content));
        }
    }

    // Let the model generate a final answer, now with the weather data
    let res = client
        .chat(&messages)
        .model("anthropic/claude-sonnet-4")
        .send()?;
    println!("Response: {}", res.content()?);

    Ok(())
}

```

```
>> Prompt: How's the weather in New York?
Tool function used: get_weather
Response: Based on the current weather data for New York City, here's what the weather looks like:

**Current Temperature (July 21st):**
- Around 29-31°C during the day (quite warm!)
- Cooling down to about 22-29°C in the evening

**Today's Forecast:**
- Starting around 29°C in the early morning
- Peak temperatures reaching about 31°C in the afternoon/evening
- Pleasant temperatures in the low-to-mid 20s°C overnight

**Looking ahead:**
- **Tomorrow (July 22nd):** Cooler, with highs around 26-27°C
- **July 23rd:** Similar temperatures, mid-20s°C
- **July 24th-25th:** Getting warmer again, with temperatures climbing into the 30s°C, reaching up to 36°C by July 25th
- **July 26th-27th:** Cooling back down to the mid-20s°C

It's quite a pleasant summer day in New York with warm but not excessive temperatures. Perfect weather for outdoor activities! The forecast shows some variation over the week with a hot spell mid-week before cooling down again.
```

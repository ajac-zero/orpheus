# Tool Calling

For more advanced use cases, you will probably want to give the LLM a way to interact with its environment. Tool calling enables this by specifying a set of actions the model can take.

#### Defining tools

Most providers expect tools to be defined using JSON Schema. While this allows for flexibility, it is prone to errors and requires you to remember the specific schema that is expected for a tool definition.

To combat this, Orpheus has full, type-safe support for tool calling in chat requests via the `Tool` and `Param` objects. These objects are then transformed into valid JSON Schema under the hood.

Say you previously had a Tool definition like this, in JSON Schema.

```json
{
    "type": "function",
    "name": "get_weather",
    "description": "Get current temperature for provided coordinates.",
    "parameters": {
        "type": "object",
        "properties": {
            "latitude": {
              "type": "number"
            },
            "longitude": {
              "type": "number"
            }
        },
        "required": ["latitude", "longitude"]
    }
}
```

Lots of room to mistype a key or forget a coma, so let's use the Tool builder instead. The equivalent with Orpheus would be this.

```rust
Tool::function("get_weather")
    .description("Get current temperature for provided coordinates.")
    .with_parameters(|params| {
        params
            .property("latitude", Param::number().end())
            .property("longitude", Param::number().end())
            .required(["latitude", "longitude"])
    })
    .build()
```

As you can see, each Param also implements the builder pattern, so you can specify any nested fields.

Let's look at more examples, like this one that uses the enum key to limit the response to a set of values.

```json
{
  "type": "function",
  "function": {
    "name": "get_weather",
    "description": "Retrieves current weather for the given location.",
    "parameters": {
      "type": "object",
      "properties": {
        "location": {
          "type": "string",
          "description": "City and country e.g. Bogotá, Colombia"
        },
        "units": {
          "type": "string",
          "description": "Units the temperature will be returned in.",
          "enum": ["celsius", "fahrenheit"]
        }
      },
      "required": ["location", "units"]
    }
  }
}
```

```rust
Tool::function("get_weather")
    .description("Retrieve current weather for the given location.")
    .with_parameters(|params| {
        params
            .property("location", Param::string()
                .description("The city to get weather for.")
                .end()
            )
            .property("units", Param::string()
                .description("Units the temperature will be returned in.")
                .r#enum(["celsius", "fahrenheit"])
                .end()
            )
            .required(["location", "units"])
    })
    .build()
```

This is a more complex example that uses array and object parameters.

```json
{
  "type": "function",
  "function": {
    "name": "get_weather",
    "description": "Retrieve current weather for a set of given locations.",
    "parameters": {
      "type": "object",
      "properties": {
        "locations": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "city": {
                "type": "string",
                "description": "The city to get weather for."
              },
              "country": {
                "type": "string",
                "description": "The country the city is in."
              }
            },
            "required": ["city", "country"]
          }
        },
        "units": {
          "type": "string",
          "description": "Units the temperature will be returned in.",
          "enum": ["celsius", "fahrenheit"]
        }
      },
      required: ["locations", "units"]
    }
  }
}
```

```rust
Tool::function("get_weather")
    .description("Retrieve current weather for a set of given locations.")
    .with_parameters(|params| {
        params
            .property("locations", Param::array().items(
                Param::object()
                    .property("city", Param::string().description("The city to get weather for."))
                    .property("country", Param::string().description("The country the city is in."))
                    .required(["city", "country"])
                    .end(),
                )
                .end()
            )
            .property("units", Param::string()
                .description("Units the temperature will be returned in.")
                .r#enum(["celsius", "fahrenheit"])
                .end()
            )
    })
    .build()
```

If you have a Tool definition that is not supported by the current Tool builder, please submit an issue! The Tool builder aims to have complete support for all possible tool definitions (Though a word of warning, the more complex a tool is, the worse models will be at using it).

Now that we know how to define our tools, let's see how to use them.

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

use orpheus::prelude::*;
use serde::Deserialize;
use serde_json::Value;

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
                        .property("latitude", Param::string())
                        .property("longitude", Param::string())
                        .required(["latitude", "longitude"]),
                )
                .property(
                    "units",
                    Param::string()
                        .description("Units the temperature will be returned in.")
                        .enums(["celsius", "fahrenheit"]),
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
            let args: GetWeather = serde_json::from_str(&function.arguments)?;

            // Inner tool logic that uses function arguments
            let request_url = format!(
                "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m&temperature_unit={}",
                args.location.latitude, args.location.longitude, args.units
            );
            let weather_data: Value = reqwest::blocking::get(request_url)?.json()?;
            let content = serde_json::to_string(&weather_data)?;

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

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

    let res = client
        .respond(vec![
            Message::system("You are a weather bot. You can assume coordinates if not provided by the user. Default to celsius."),
            Message::user(prompt),
        ])
        .model("openai/gpt-4o-mini")
        .tools([get_weather_tool])
        .send()?;

    let function_calls = res.function_calls();

    if let Some(fc) = function_calls.first() {
        println!("Tool function used: {}", fc.name);

        if fc.name == "get_weather" {
            let args: GetWeather = serde_json::from_str(&fc.arguments)?;

            let request_url = format!(
                "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m&temperature_unit={}",
                args.location.latitude, args.location.longitude, args.units
            );
            let weather_data: Value = reqwest::blocking::get(request_url)?.json()?;
            let content = serde_json::to_string(&weather_data)?;

            let mut followup = Input::from(Vec::<Message>::new());
            followup.push_function_output(&fc.call_id, content);

            let res = client
                .respond(followup)
                .model("openai/gpt-4o-mini")
                .previous_response_id(&res.id)
                .send()?;

            if let Some(text) = res.output_text() {
                println!("Response: {}", text);
            }
        }
    }

    Ok(())
}

use orpheus_agent::prelude::*;
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

    let get_weather = AgentTool::new(
        Tool::function("get_weather")
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
            .build(),
        |call| {
            let args: GetWeather = serde_json::from_str(&call.arguments)?;
            let request_url = format!(
                "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m&temperature_unit={}",
                args.location.latitude, args.location.longitude, args.units
            );
            let weather_data: Value = reqwest::blocking::get(request_url)?.json()?;
            Ok(serde_json::to_string(&weather_data)?)
        },
    );

    let run = Agent::new(&client)
        .model("openai/gpt-4o-mini")
        .instructions(
            "You are a weather bot. You can assume coordinates if not provided by the user. Default to celsius.",
        )
        .tool(get_weather)
        .run("How's the weather in New York?")?;

    if let Some(text) = run.response.output_text() {
        println!("Response: {text}");
    }

    Ok(())
}

use futures_util::StreamExt;
use orpheus::prelude::*;

const TEST_MODEL: &str = "openai/gpt-4o-mini";

#[test]
fn simple_request() {
    let client = Orpheus::from_env().unwrap();

    let response = client
        .respond(vec![
            Message::system("You are a friend"),
            Message::user("Hello!"),
        ])
        .model(TEST_MODEL)
        .send();

    assert!(response.is_ok());

    let res = response.unwrap();
    assert!(res.output_text().is_some());
}

#[test]
fn stream_request() {
    let client = Orpheus::from_env().unwrap();

    let stream = client
        .respond(vec![
            Message::system("You are a friend"),
            Message::user("Hello!"),
        ])
        .model(TEST_MODEL)
        .stream();

    assert!(stream.is_ok());

    let mut stream = stream.unwrap();
    let mut got_text = false;
    let mut got_completed = false;

    while let Some(Ok(event)) = Iterator::next(&mut stream) {
        if event.as_text_delta().is_some() {
            got_text = true;
        }
        if event.as_completed().is_some() {
            got_completed = true;
        }
    }

    assert!(got_text);
    assert!(got_completed);
}

#[tokio::test]
async fn async_stream_request() {
    let client = AsyncOrpheus::from_env().unwrap();

    let stream = client
        .respond(vec![
            Message::system("You are a friend"),
            Message::user("Hello!"),
        ])
        .model(TEST_MODEL)
        .stream()
        .await;

    assert!(stream.is_ok());

    let mut stream = stream.unwrap();
    let mut accumulated_text = String::new();
    let mut got_completed = false;

    while let Some(event) = StreamExt::next(&mut stream).await {
        let event = event.unwrap();
        if let Some(text) = event.as_text_delta() {
            accumulated_text.push_str(text);
        }
        if event.as_completed().is_some() {
            got_completed = true;
        }
    }

    assert!(got_completed);
    assert!(!accumulated_text.is_empty());
}

#[test]
fn image_request() {
    let client = Orpheus::from_env().unwrap();

    let image_url = "https://misanimales.com/wp-content/uploads/2022/03/Shih-Poo-Shih-Tzu-1024x680-1-768x510.jpg";

    let response = client
        .respond(vec![
            Message::system("You are a photography connoisseur."),
            Message::user("What do you think of this image?").with_image(image_url),
        ])
        .model(TEST_MODEL)
        .send();

    assert!(response.is_ok());
    assert!(response.unwrap().output_text().is_some());
}

#[test]
fn tool_request() {
    let client = Orpheus::from_env().unwrap();

    let tool = Tool::function("extract_info")
        .description("extract some data fields from a sentence")
        .parameters(
            Param::object()
                .property("name", Param::string().end())
                .property("age", Param::integer().end())
                .required(["name", "age"])
                .end(),
        )
        .build();

    let response = client
        .respond("Isabella is 12 years old.")
        .model(TEST_MODEL)
        .tools([tool])
        .send();

    assert!(response.is_ok());

    let res = response.unwrap();
    let function_calls = res.function_calls();
    assert!(!function_calls.is_empty());
}

#[test]
fn structured_request() {
    let client = Orpheus::from_env().unwrap();

    let response_format = Format::json("weather")
        .with_schema(|schema| {
            schema
                .property(
                    "location",
                    Param::string().description("City or location name"),
                )
                .property(
                    "temperature",
                    Param::number().description("Temperature in Celsius"),
                )
                .property(
                    "conditions",
                    Param::string().description("Weather conditions description"),
                )
                .required(["location", "temperature", "conditions"])
        })
        .build();

    let response = client
        .respond("What is the weather like in New York City?")
        .model("openai/gpt-4o-mini")
        .text_format(response_format)
        .send()
        .unwrap();

    #[derive(Debug, serde::Deserialize)]
    struct WeatherResponse {
        #[serde(rename = "location")]
        _location: String,
        #[serde(rename = "temperature")]
        _temperature: f64,
        #[serde(rename = "conditions")]
        _conditions: String,
    }

    let content = response.output_text().unwrap();
    let weather_response: WeatherResponse = serde_json::from_str(&content).unwrap();
    dbg!(weather_response);
}

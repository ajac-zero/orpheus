use futures_lite::StreamExt;
use orpheus::{models::Plugin, prelude::*};

const TEST_MODEL: &str = "google/gemini-2.5-flash-lite";

#[test]
fn simple_request() {
    let client = Orpheus::from_env().unwrap();

    let response = client
        .chat([Message::system("You are a friend"), Message::user("Hello!")])
        .model(TEST_MODEL)
        .send();
    println!("{:?}", response);

    assert!(response.is_ok());

    let chat_response = response.unwrap();

    let choices = chat_response.choices;
    assert!(!choices.is_empty());
}

#[test]
fn stream_request() {
    let client = Orpheus::from_env().unwrap();

    let response = client
        .chat([Message::system("You are a friend"), Message::user("Hello!")])
        .model(TEST_MODEL)
        .stream();
    println!("{:?}", response);

    assert!(response.is_ok());

    let mut chat_response = response.unwrap();
    let mut is_finished = false;

    while let Some(Ok(chunk)) = chat_response.next() {
        assert_eq!(chunk.object, "chat.completion.chunk");
        assert_eq!(chunk.choices.len(), 1);

        let choice = &chunk.choices[0];

        if choice.finish_reason.is_some() {
            is_finished = true;
            assert_eq!(choice.finish_reason, Some("stop".to_string()));
        }
    }

    assert!(is_finished);
}

#[tokio::test]
async fn async_stream_request() {
    let client = AsyncOrpheus::from_env().unwrap();

    let response = client
        .chat([Message::system("You are a friend"), Message::user("Hello!")])
        .model(TEST_MODEL)
        .stream()
        .await;
    println!("{:?}", response);

    assert!(response.is_ok());

    let mut chat_response = response.unwrap();

    let mut accumulated_content = String::new();
    let mut is_finished = false;

    let mut count = 0;
    while let Some(chunk) = chat_response.next().await {
        println!("{:?}", chunk);
        count = count + 1;
        let chunk = chunk.unwrap();
        assert_eq!(chunk.object, "chat.completion.chunk");
        assert_eq!(chunk.choices.len(), 1);

        let choice = &chunk.choices[0];
        accumulated_content.push_str(&choice.delta.content.to_string());

        if choice.finish_reason.is_some() {
            is_finished = true;
            assert_eq!(choice.finish_reason, Some("stop".to_string()));
        }
    }

    println!("Processed chunks: {}", count);
    assert!(is_finished);
    println!(
        "Successfully processed streaming chat completion: '{}'",
        accumulated_content
    );
}

#[test]
fn image_request() {
    let client = Orpheus::from_env().unwrap();

    let image_url = "https://misanimales.com/wp-content/uploads/2022/03/Shih-Poo-Shih-Tzu-1024x680-1-768x510.jpg";

    let response = client
        .chat([
            Message::system("You are a photography connoseur."),
            Message::user("What do you think of this image?").with_image(image_url),
        ])
        .model(TEST_MODEL)
        .send();

    println!("{:?}", response);
    assert!(response.is_ok());

    let chat_response = response.unwrap();

    let choices = chat_response.choices;
    assert!(!choices.is_empty());
}

#[test]
fn file_request() {
    let client = Orpheus::from_env().unwrap();

    let pdf_url = "https://bitcoin.org/bitcoin.pdf";

    let response = client
        .chat([Message::user("Can you tell me the contents of this pdf?")
            .with_file("bitcoin.pdf", pdf_url)])
        .model(TEST_MODEL)
        .send();

    assert!(response.is_ok());

    let chat_response = response.unwrap();

    let choices = chat_response.choices;
    assert!(!choices.is_empty());
}

#[test]
fn web_plugin_request() {
    let client = Orpheus::from_env().unwrap();

    let web_plugin = Plugin::web().build();

    let response = client
        .chat("What are the latest crypto news?")
        .model(TEST_MODEL)
        .plugins([web_plugin])
        .send();

    assert!(response.is_ok());

    let chat_response = response.unwrap();

    let choices = chat_response.choices;
    assert!(!choices.is_empty());
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
        .chat("Isabella is 12 years old.")
        .model(TEST_MODEL)
        .tools([tool])
        .send();
    println!("{:?}", response);

    assert!(response.is_ok());

    let chat_response = response.unwrap();

    let choices = chat_response.choices;
    assert!(!choices.is_empty());
}

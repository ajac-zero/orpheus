use colored::Colorize;
use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    // Initialize the client using the ORPHEUS_API_KEY environment variable
    let client = Orpheus::from_env()?;

    // Define the expected JSON structure for extracted person data
    // This schema ensures the model returns consistent, parseable data
    let person_format = Format::json("person")
        .with_schema(|schema| {
            schema
                // String property for the person's name
                .property("name", Param::string())
                // Number property for the person's age (allows decimals)
                .property("age", Param::number())
                // Both name and age are required fields
                .required(["name", "age"])
        })
        .build();

    // Sample text containing person information to extract
    let prompt = "Jessica is a 20 year old college student.";
    println!("{}", "Prompt:".blue());
    println!("{}", prompt);

    // Create a conversation with a system message that sets the extraction context
    let messages = vec![
        Message::system("You are a data extraction bot."),
        Message::user(prompt),
    ];

    // Send the request with structured output format
    // The model will be constrained to return JSON matching our schema
    let res = client
        .chat(&messages)
        .model("mistralai/mistral-medium-3.1")
        .response_format(person_format) // Enforce the JSON schema we defined
        .send()?;

    // Display the structured JSON response
    println!("{}", "Response:".green());
    println!("{}", res.content()?);
    // Expected output: {"name": "Jessica", "age": 20}

    Ok(())
}

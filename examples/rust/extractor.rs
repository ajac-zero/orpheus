#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! anyhow = "1.0.98"
//! colored = "3.0.0"
//! orpheus = { path = "../.." }
//! ```
use colored::Colorize;
use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;

    // Define your response format
    let person_format = Format::json("person")
        .with_schema(|schema| {
            schema
                .property("name", Param::string())
                .property("age", Param::number())
                .required(["name", "age"])
        })
        .build();

    let prompt = "Jessica is a 20 year old college student.";
    println!("{}", "Prompt:".blue());
    println!("{}", prompt);

    let messages = vec![
        Message::system("You are a data extraction bot."),
        Message::user(prompt),
    ];

    let res = client
        .chat(&messages)
        .model("mistralai/mistral-medium-3.1")
        .response_format(person_format) // Add your response format to the chat request
        .send()?;

    println!("{}", "Response:".green());
    println!("{}", res.content()?);

    Ok(())
}

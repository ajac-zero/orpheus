use colored::Colorize;
use orpheus::prelude::*;

fn main() -> anyhow::Result<()> {
    let client = Orpheus::from_env()?;

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

    let res = client
        .respond(vec![
            Message::system("You are a data extraction bot."),
            Message::user(prompt),
        ])
        .model("openai/gpt-4o-mini")
        .text_format(person_format)
        .send()?;

    println!("{}", "Response:".green());
    if let Some(text) = res.output_text() {
        println!("{}", text);
    }

    Ok(())
}

use colored::Colorize;
use orpheus::prelude::*;
use rand::seq::IndexedRandom;

const PROMPT: &str = "You are a charades game master.
Choose one animal, and the user must guess what it is.
The user is only allowed three guesses.
Before every guess, chat a riddle about your animal.
If the user guesses correctly within those three guesses, they win.
If the user runs out of guesses, you win.
Use the 'player_win' tool if the player guesses correctly.
Use the 'game_over' tool if the player runs out of guesses.
Don't hold back.";

const MODELS: [&str; 3] = [
    "openai/gpt-4o-mini",
    "openai/gpt-4o",
    "openai/gpt-4.1-nano",
];

#[derive(serde::Deserialize)]
struct GameOverArgs {
    answer: String,
}

fn main() -> anyhow::Result<()> {
    let orpheus = Orpheus::from_env()?;

    let mut rng = rand::rng();
    let model = *MODELS.choose(&mut rng).expect("is not empty");
    println!("Using model: {}", model.yellow());

    let tools = vec![
        Tool::function("player_win").empty(),
        Tool::function("game_over")
            .with_parameters(|p| p.property("answer", Param::string().end()))
            .build(),
    ];

    let mut previous_response_id: Option<String> = None;
    let mut input = Input::from(vec![Message::system(PROMPT)]);

    loop {
        let mut builder = orpheus
            .respond(&input)
            .model(model)
            .tools(tools.clone())
            .temperature(0.9);

        if let Some(ref prev_id) = previous_response_id {
            builder = builder.previous_response_id(prev_id);
        }

        let response = builder.send()?;
        previous_response_id = Some(response.id.clone());

        let function_calls = response.function_calls();
        if let Some(fc) = function_calls.first() {
            if fc.name == "player_win" {
                println!("{}", FINISH_BANNER.green().bold());
            }

            if fc.name == "game_over" {
                println!("{}", GAME_OVER_BANNER.red().bold());
                if let Some(ref args_str) = fc.arguments {
                    let args: GameOverArgs = serde_json::from_str(args_str)?;
                    println!("Real answer: {}", args.answer);
                }
            }

            return Ok(());
        }

        if let Some(text) = response.output_text() {
            println!("{}", "Game Master ================".yellow());
            println!("{}", text);
        }

        println!("{}", "Answer =====================".blue());
        let mut user_input = String::new();
        std::io::stdin().read_line(&mut user_input)?;
        println!();

        input = Input::from(Message::user(user_input));
    }
}

const FINISH_BANNER: &str = "
██╗   ██╗ ██████╗ ██╗   ██╗    ██╗    ██╗ ██████╗ ███╗   ██╗    ██╗██╗██╗
╚██╗ ██╔╝██╔═══██╗██║   ██║    ██║    ██║██╔═══██╗████╗  ██║    ██║██║██║
 ╚████╔╝ ██║   ██║██║   ██║    ██║ █╗ ██║██║   ██║██╔██╗ ██║    ██║██║██║
  ╚██╔╝  ██║   ██║██║   ██║    ██║███╗██║██║   ██║██║╚██╗██║    ╚═╝╚═╝╚═╝
   ██║   ╚██████╔╝╚██████╔╝    ╚███╔███╔╝╚██████╔╝██║ ╚████║    ██╗██╗██╗
   ╚═╝    ╚═════╝  ╚═════╝      ╚══╝╚══╝  ╚═════╝ ╚═╝  ╚═══╝    ╚═╝╚═╝╚═╝
";

const GAME_OVER_BANNER: &str = "
 ██████╗  █████╗ ███╗   ███╗███████╗     ██████╗ ██╗   ██╗███████╗██████╗          ██╗
██╔════╝ ██╔══██╗████╗ ████║██╔════╝    ██╔═══██╗██║   ██║██╔════╝██╔══██╗    ██╗ ██╔╝
██║  ███╗███████║██╔████╔██║█████╗      ██║   ██║██║   ██║█████╗  ██████╔╝    ╚═╝██╔╝
██║   ██║██╔══██║██║╚██╔╝██║██╔══╝      ██║   ██║╚██╗ ██╔╝██╔══╝  ██╔══██╗    ██╗╚██╗
╚██████╔╝██║  ██║██║ ╚═╝ ██║███████╗    ╚██████╔╝ ╚████╔╝ ███████╗██║  ██║    ╚═╝ ╚██╗
╚═════╝ ╚═╝  ╚═╝╚═╝     ╚═╝╚══════╝     ╚═════╝   ╚═══╝  ╚══════╝╚═╝  ╚═╝         ╚═╝
";

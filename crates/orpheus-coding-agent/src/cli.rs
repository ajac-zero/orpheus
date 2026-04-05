use std::{env, path::PathBuf};

use anyhow::{Context, bail};
use clap::{Parser, Subcommand};

use crate::{
    Result,
    config::OutputMode,
    runtime::{CodingAgentRuntime, RunOptions},
};

#[derive(Debug, Parser)]
#[command(name = "orpheus-code")]
#[command(about = "A coding agent built on top of Orpheus", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    #[arg(long)]
    cwd: Option<PathBuf>,

    #[arg(long)]
    model: Option<String>,

    #[arg(long)]
    session: Option<String>,

    #[arg(long)]
    system_prompt: Option<String>,

    #[arg(long)]
    max_turns: Option<usize>,

    #[arg(long)]
    no_tools: bool,

    #[arg(long)]
    no_stream: bool,

    #[arg(long)]
    json: bool,
}

#[derive(Debug, Subcommand)]
enum Command {
    Ask {
        prompt: Vec<String>,
    },
    Resume {
        session_id: Option<String>,
    },
    Sessions {
        #[command(subcommand)]
        command: SessionsCommand,
    },
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    Models {
        #[command(subcommand)]
        command: ModelsCommand,
    },
    Tools {
        #[command(subcommand)]
        command: ToolsCommand,
    },
}

#[derive(Debug, Subcommand)]
enum SessionsCommand {
    List,
    Show {
        session_id: String,
    },
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    Show,
}

#[derive(Debug, Subcommand)]
enum ModelsCommand {
    List,
}

#[derive(Debug, Subcommand)]
enum ToolsCommand {
    List,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let cwd = cli
        .cwd
        .clone()
        .unwrap_or(env::current_dir().context("failed to determine current working directory")?);
    let runtime = CodingAgentRuntime::from_working_dir(cwd)?;
    let options = run_options(&cli);

    match cli.command {
        None => runtime.run_interactive(cli.session.clone(), options),
        Some(Command::Ask { prompt }) => {
            if prompt.is_empty() {
                bail!("ask requires a prompt")
            }

            runtime.run_print(&prompt.join(" "), options)?;
            Ok(())
        }
        Some(Command::Resume { session_id }) => {
            runtime.run_interactive(session_id.or(cli.session.clone()), options)
        }
        Some(Command::Sessions { command: SessionsCommand::List }) => {
            for session in runtime.session_manager().list()? {
                println!(
                    "{}\t{}\t{}\t{}",
                    session.id,
                    session.model,
                    session.updated_at,
                    session.cwd.display()
                );
            }
            Ok(())
        }
        Some(Command::Sessions {
            command: SessionsCommand::Show { session_id },
        }) => {
            let session = runtime.session_manager().load(&session_id)?;
            println!("{}", serde_json::to_string_pretty(&session)?);
            Ok(())
        }
        Some(Command::Config { command: ConfigCommand::Show }) => {
            println!(
                "{}",
                serde_json::to_string_pretty(runtime.settings())?
            );
            println!("global_settings_path: {}", runtime.loaded_settings().global_path.display());
            println!("project_settings_path: {}", runtime.loaded_settings().project_path.display());
            println!("session_dir: {}", runtime.session_manager().root().display());
            Ok(())
        }
        Some(Command::Models { command: ModelsCommand::List }) => {
            for model in runtime.available_models() {
                println!("{model}");
            }
            Ok(())
        }
        Some(Command::Tools { command: ToolsCommand::List }) => {
            for name in runtime.available_tools() {
                println!("{name}");
            }
            Ok(())
        }
    }
}

fn run_options(cli: &Cli) -> RunOptions {
    RunOptions {
        session_id: cli.session.clone(),
        model: cli.model.clone(),
        system_prompt: cli.system_prompt.clone(),
        max_turns: cli.max_turns,
        stream: !cli.no_stream,
        use_tools: !cli.no_tools,
        output_mode: cli.json.then_some(OutputMode::Json),
    }
}

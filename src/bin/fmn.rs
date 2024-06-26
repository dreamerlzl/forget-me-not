#![forbid(unsafe_code)]

use std::env;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use task_reminder::client::send_request;
use task_reminder::comm::{
    get_local_now, parse_at, parse_date, parse_duration, ContextCommand, Request, Response,
};
use task_reminder::format::tabular_output;
use task_reminder::task_manager::ClockType;

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Add {
        description: String,
        #[command(subcommand)]
        command: AddCommand,

        #[arg(short, long)]
        image_path: Option<String>,

        #[arg(short, long)]
        sound_path: Option<String>,
    },
    Rm {
        task_id: String,
    },
    List,
    Context {
        #[command(subcommand)]
        command: ContextCommand,
    },
}

#[derive(Subcommand)]
enum AddCommand {
    After {
        duration: String,
    },
    At {
        time: String,
        #[arg(short, long)]
        per_day: bool,
    },
    Per {
        duration: String,
    },
    On {
        date: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let request = match cli.command {
        Command::Add {
            description,
            command,
            mut image_path,
            mut sound_path,
        } => {
            let clock_type = match command {
                AddCommand::At { time, per_day } => {
                    let next_fire = parse_at(&time)?;
                    if per_day {
                        ClockType::OncePerDay(next_fire.hour(), next_fire.minute())
                    } else {
                        ClockType::Once(next_fire)
                    }
                }
                AddCommand::After { duration } => {
                    let duration = parse_duration(&duration)?;
                    if duration.as_secs() == 0 {
                        return Err(anyhow!("after <duration> should not be 0"));
                    }
                    let next_fire = get_local_now() + duration;
                    ClockType::Once(next_fire)
                }
                AddCommand::Per { duration } => {
                    let _ = parse_duration(&duration)?;
                    ClockType::Period(duration)
                }
                AddCommand::On { date } => {
                    let next_fire = parse_date(&date)?;
                    ClockType::Once(next_fire)
                }
            };
            if image_path.is_none() {
                if let Ok(system_image_path) = env::var("FMN_IMAGE_PATH") {
                    image_path = Some(system_image_path);
                }
            }
            if sound_path.is_none() {
                if let Ok(system_sound_path) = env::var("FMN_SOUND_PATH") {
                    sound_path = Some(system_sound_path);
                }
            }
            Request::Add(description, clock_type, image_path, sound_path)
        }
        Command::Rm { task_id } => Request::Cancel(task_id),
        Command::List => Request::Show,
        Command::Context { command } => Request::ContextRequest(command),
    };

    //println!("request is {:?}", request);
    #[cfg(feature = "tcp")]
    let dest = env::var("FMN_DAEMON_ADDR").unwrap_or_else(|_| "127.0.0.1:8082".to_owned());

    #[cfg(feature = "unix_socket")]
    let dest = env::var("FMN_DAEMON_ADDR").unwrap_or_else(|_| "/tmp/fmn.sock".to_owned());
    match send_request(request.clone(), &dest) {
        Ok(response) => match response {
            Response::GetTasks(tasks) => {
                println!("{}", tabular_output(&tasks));
            }
            Response::GetContexts(contexts) => {
                println!(" * {}", contexts.join("\n   "));
            }
            Response::Fail(error_string) => {
                eprintln!("request \"{request:?}\" failed: {error_string}");
            }
            _ => println!("success: {response:?}"),
        },
        Err(e) => {
            eprintln!("request \"{request:?}\" failed: {e}");
        }
    }
    Ok(())
}

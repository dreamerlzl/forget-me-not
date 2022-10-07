use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use regex::Regex;
use serde_json::{from_slice, to_string};
use time::OffsetDateTime;

use std::env;
use std::net::{Ipv4Addr, UdpSocket};
use std::time::Duration;

use task_reminder::comm::{Request, Response};
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
    },
    Rm {
        index: usize,
    },
    Show,
}

#[derive(Subcommand)]
enum AddCommand {
    After { duration: String },
    At { time: String },
    Per { duration: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let request = match cli.command {
        Command::Add {
            description,
            command,
        } => match &command {
            AddCommand::At { time } => {
                let next_fire = parse_at(time)?;
                Request::Add(description, ClockType::Once(next_fire))
            }
            AddCommand::After { duration } => {
                let duration = parse_duration(duration)?;
                let next_fire = OffsetDateTime::now_local()? + duration;
                Request::Add(description, ClockType::Once(next_fire))
            }
            AddCommand::Per { duration } => {
                let duration = parse_duration(duration)?;
                Request::Add(description, ClockType::Period(duration))
            }
        },
        Command::Rm { index } => Request::Cancel(index),
        Command::Show => Request::Show,
    };

    //println!("request is {:?}", request);
    let dest = env::var("REMINDER_DAEMON_ADDR").unwrap_or_else(|_| "127.0.0.1:8082".to_owned());
    match send_request(request.clone(), &dest) {
        Ok(response) => match response {
            Response::GetTasks(tasks) => {
                for (i, task) in tasks.into_iter().enumerate() {
                    println!("{} {}", i, task);
                }
            }
            _ => println!("success: {:?}", response),
        },
        Err(e) => {
            println!("fail to remind task {:?}: {}", request, e);
        }
    }
    Ok(())
}

fn send_request(request: Request, dest: &str) -> Result<Response> {
    let socket =
        UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).context("fail to bind to a random port")?;
    let serialized = to_string(&request).expect("fail to serialize request");
    socket
        .send_to(serialized.as_bytes(), dest)
        .context(format!("fail to send to {}", dest))?;
    let mut buf = [0; 1024];
    let amt = socket.recv(&mut buf)?;
    let response: Response = from_slice(&buf[..amt]).context("fail to deserialize response")?;
    Ok(response)
}

// only used for at
fn parse_at(next_fire: &str) -> Result<OffsetDateTime> {
    let re = Regex::new(r"(?P<hour>\d+):(?P<minute>\d+):(?P<second>\d+)").unwrap();
    let mut components = [0 as u8; 3];
    if let Some(captures) = re.captures(next_fire) {
        for (i, component) in ["hour", "minute", "second"].into_iter().enumerate() {
            components[i] = captures
                .name(component)
                .map(|m| {
                    // dbg!(component, m.as_str());
                    m.as_str()
                })
                .ok_or(anyhow!(
                    "invalid time! correct examples: 13:11:04, 23:01:59"
                ))?
                .parse()
                .context(format!("invalid {}", component))?;
        }
        let now = OffsetDateTime::now_local()?;
        let hour = components[0];
        let minute = components[1];
        let second = components[2];
        //if (components[0], components[1], components[2]) < (now.hour(), now.minute(), now.second())
        //{
        //    return Err(anyhow!("next_fire should not be in the past!"));
        //}
        Ok(now
            .replace_millisecond(0)?
            .replace_nanosecond(0)?
            .replace_microsecond(0)?
            .replace_hour(hour)?
            .replace_minute(minute)?
            .replace_second(second)?)
    } else {
        Err(anyhow!("fail to parse next_fire!"))
    }
}

// duration looks like this 1d1h1m1s
fn parse_duration(duration: &str) -> Result<Duration> {
    let re = Regex::new(
        r"^(?:(?P<day>\d+)d)?(?:(?P<hour>\d+)h)?(?:(?P<minute>\d+)m)?(?:(?P<second>\d+)s)?$",
    )
    .unwrap();
    if !re.is_match(duration) {
        return Err(anyhow!(
            "invalid duration format; valid examples: 1d1h1m1s, 2h, 30s, 55m"
        ));
    }
    if let Some(captures) = re.captures(duration) {
        let mut components = [0 as u64; 4];
        for (i, component) in ["day", "hour", "minute", "second"].into_iter().enumerate() {
            components[i] = captures
                .name(component)
                .map(|m| {
                    // dbg!(component, m.as_str());
                    m.as_str()
                })
                .unwrap_or_else(|| "0")
                .parse()
                .context(format!("invalid {}", component))?;
        }

        let secs =
            components[0] * 3600 * 24 + components[1] * 3600 + components[2] * 60 + components[3];
        Ok(Duration::from_secs(secs))
    } else {
        Ok(Duration::from_secs(0))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use time::OffsetDateTime;

    use anyhow::Result;

    use super::parse_at;
    use super::parse_duration;

    #[test]
    fn test_duration() -> Result<()> {
        let test_cases = vec![
            ("0h0m10s", 10),
            ("1h0m0s", 3600),
            ("0h1m0s", 60),
            ("0h0m0s", 0),
            ("1d", 3600 * 24),
            ("1h", 3600),
            ("1d1s", 3600 * 24 + 1),
        ];

        for (duration, expected_seconds) in test_cases {
            let expected_duration = Duration::from_secs(expected_seconds);
            assert_eq!(parse_duration(duration)?, expected_duration);
        }
        Ok(())
    }

    #[test]
    fn test_duration_err() {
        let test_cases = vec!["1f", "abc", "@341", "1d2@3"];
        for duration in test_cases {
            //dbg!("testing {}", duration);
            assert!(parse_duration(duration).is_err());
        }
    }

    #[test]
    fn test_next_fire() -> Result<()> {
        let test_cases = vec![
            ("13:24:20", 13, 24, 20),
            ("23:01:01", 23, 1, 1),
            ("01:59:59", 1, 59, 59),
        ];
        for (next_fire, hour, minute, second) in test_cases {
            let now = OffsetDateTime::now_utc()
                .replace_millisecond(0)?
                .replace_nanosecond(0)?
                .replace_microsecond(0)?
                .replace_hour(hour)?
                .replace_minute(minute)?
                .replace_second(second)?;
            assert_eq!(now, parse_at(next_fire)?);
        }

        Ok(())
    }

    #[test]
    fn text_next_fire_err() {
        let test_cases = vec!["123:24", "11:34", "098", ""];
        for next_fire in test_cases {
            assert!(parse_at(next_fire).is_err());
        }
    }
}

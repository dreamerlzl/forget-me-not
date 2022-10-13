use crate::task_manager::{ClockType, Task, TaskID};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Request {
    Add(String, ClockType, Option<String>, Option<String>),
    Cancel(TaskID),
    Show,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    AddSuccess,
    CancelSuccess,
    Fail(String),
    GetTasks(Vec<Task>),
}

pub fn parse_duration(duration: &str) -> Result<Duration> {
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

use std::fmt::Display;
use std::time::Duration;

use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use serde_json::to_vec;
use time::OffsetDateTime;

pub type TaskID = String;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    created_at: OffsetDateTime, // just a metadata
    pub description: String,
    pub task_id: TaskID, // used as the unique id of the task
    pub clock_type: ClockType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClockType {
    Once(OffsetDateTime),
    Period(Duration),
}

impl Display for ClockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClockType::Once(next_fire) => {
                write!(f, "once at {}", next_fire)
            }
            ClockType::Period(period) => {
                write!(f, "every {} secs", period.as_secs())
            }
        }
    }
}

impl Task {
    pub fn new(description: String, clock_type: ClockType) -> Self {
        Task {
            description,
            clock_type,
            created_at: OffsetDateTime::now_utc(),
            task_id: nanoid!(),
            // task_id: Uuid::new_v4(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        to_vec(self).expect(&format!("fail to serialize task {:?}", &self))
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:8} {: <15} {: <15} {: <15}",
            self.task_id, self.description, self.clock_type, self.created_at
        )
    }
}

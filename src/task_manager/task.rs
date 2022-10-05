use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde_json::to_vec;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

pub type TaskID = Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    created_at: OffsetDateTime, // just a metadata
    description: String,
    pub task_id: TaskID,           // used as the unique id of the task
    pub next_fire: OffsetDateTime, // for at, after, on
    period: Option<Duration>,      // for per
}

impl Task {
    pub fn new(description: String, next_fire: OffsetDateTime) -> Self {
        Task {
            description,
            next_fire,
            created_at: OffsetDateTime::now_local().expect("fail to create local datetime"),
            period: None,
            task_id: Uuid::new_v4(),
        }
    }

    pub fn with_period(self, period: Duration) -> Self {
        Task {
            period: Some(period),
            ..self
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        to_vec(self).expect(&format!("fail to serialize task {:?}", &self))
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(period) = self.period {
            write!(
                f,
                "{0: <30} {1: <15} {2: <15}",
                self.description, self.next_fire, period
            )
        } else {
            write!(
                f,
                "{0: <30} {1: <15} {2: <15}",
                self.description, self.next_fire, ""
            )
        }
    }
}

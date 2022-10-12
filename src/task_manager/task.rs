use std::fmt::Display;
use std::time::Duration;

use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use serde_json::to_vec;
use time::{format_description, OffsetDateTime};

pub type TaskID = String;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    created_at: OffsetDateTime, // just a metadata
    pub description: String,
    pub task_id: TaskID, // used as the unique id of the task
    pub clock_type: ClockType,

    // media shown when the notification fires
    image_path: Option<String>,
    sound_path: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClockType {
    Once(OffsetDateTime),
    OncePerDay(u8, u8), // hour(0-24), minute(0-59)
    Period(Duration),
}

impl Display for ClockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let format = format_description::parse("[year]-[month]-[day] [hour]:[minute]")
            .expect("fail to parse display format for OffsetDatetime");
        match self {
            ClockType::Once(next_fire) => {
                write!(
                    f,
                    "once {}",
                    next_fire
                        .format(&format)
                        .expect("fail to display custom OffsetDatetime format")
                )
            }
            ClockType::Period(period) => {
                write!(f, "every {}", period.as_secs())
            }
            ClockType::OncePerDay(hour, minute) => {
                write!(f, "everyday {}:{}", hour, minute)
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
            image_path: None,
            sound_path: None,
            // task_id: Uuid::new_v4(),
        }
    }

    pub fn add_image(&mut self, image_path: String) {
        self.image_path = Some(image_path);
    }

    pub fn add_sound(&mut self, sound_path: String) {
        self.sound_path = Some(sound_path);
    }

    pub fn get_image(&self) -> Option<&str> {
        self.image_path.as_deref()
    }

    pub fn get_sound(&self) -> Option<&str> {
        self.sound_path.as_deref()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        to_vec(self).expect(&format!("fail to serialize task {:?}", &self))
    }
}

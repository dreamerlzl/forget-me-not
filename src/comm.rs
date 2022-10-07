use crate::task_manager::{ClockType, Task, TaskID};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Request {
    Add(String, ClockType),
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

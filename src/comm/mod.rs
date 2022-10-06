use crate::task_manager::ClockType;
use serde::{Deserialize, Serialize};

mod listener;

#[derive(Debug, Serialize, Deserialize)]
enum Request {
    Add(String, ClockType),
    Cancel(usize),
}

#[derive(Debug, Serialize, Deserialize)]
enum Response {
    AddSuccess(usize),
    CancelSuccess,
    Fail(String),
}

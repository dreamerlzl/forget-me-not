use crate::task_manager::ClockType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Request {
    Add(String, ClockType),
    Cancel(usize),
    // Show,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    AddSuccess(usize),
    CancelSuccess,
    Fail(String),
}

use crate::task_manager::{Task, TaskID};
use anyhow::Result;

pub struct Scheduler {}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {}
    }

    pub fn add_task(&mut self, task: &Task) -> Result<()> {
        Ok(())
    }

    pub fn cancel_task(&mut self, task: &Task) -> Result<()> {
        Ok(())
    }
}

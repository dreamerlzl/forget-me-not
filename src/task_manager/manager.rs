use anyhow::{anyhow, Context, Result};
use serde_json::from_str;

use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, Write};
use std::path::Path;

use crate::scheduler::Scheduler;
use crate::task_manager::Task;

pub struct TaskManager<P: AsRef<Path>> {
    scheduler: Scheduler,
    tasks: Vec<Task>,
    task_appender: File,
    path: P,
}

impl<P: AsRef<Path>> TaskManager<P> {
    pub fn add_task(&mut self, task: Task) -> Result<usize> {
        // push the task to the scheduler
        // and returns back a unique id
        // which would be later used to cancel a periodic task
        let bytes = task.to_bytes();
        let task_id = self.tasks.len();
        self.tasks.push(task);
        self.task_appender.write_all(bytes.as_slice())?;
        self.task_appender.write_all("\n".as_bytes())?;
        self.task_appender.flush()?;
        self.scheduler
            .add_task(self.tasks.last().unwrap().clone())?;
        Ok(task_id)
    }

    pub fn cancel_task(&mut self, index: usize) -> Result<()> {
        // to avoid panics
        if index >= self.tasks.len() {
            return Err(anyhow!("task_id doesn't exist: {}", index));
        }
        let task = self.tasks.remove(index);
        // rewrite the whole task file
        self.task_appender = OpenOptions::new()
            .truncate(true)
            .write(true)
            .open(&self.path)?;
        for task in self.tasks.iter() {
            self.task_appender.write_all(task.to_bytes().as_slice())?;
            self.task_appender.write_all("\n".as_bytes())?;
        }
        self.task_appender.flush()?;
        self.task_appender = OpenOptions::new().append(true).open(&self.path)?;
        self.scheduler.cancel_task(task.clone())?;
        Ok(())
    }

    // new returns a new TaskManager
    pub fn new(path: P, scheduler: Scheduler) -> Result<Self> {
        let task_appender = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .with_context(|| format!("fail to open task store"))?;
        let tasks = read_tasks(&path)?;
        Ok(TaskManager {
            scheduler,
            tasks,
            task_appender,
            path,
        })
    }
}

pub fn read_tasks<P>(path: P) -> Result<Vec<Task>>
where
    P: AsRef<Path>,
{
    let mut tasks = vec![];
    let file = File::open(&path).context(format!("fail to load persistent tasks"))?;
    for line in io::BufReader::new(file).lines() {
        let line = line?;
        if line.is_empty() {
            break;
        }
        let task: Task = from_str(&line)?;
        tasks.push(task);
    }
    Ok(tasks)
}

use std::fs::OpenOptions;
use std::io::{self, BufRead, Write};
use std::iter::Iterator;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::from_str;
use time::OffsetDateTime;

use super::task_context::default_context;
use super::{ClockType, TaskID};
use crate::scheduler::Scheduler;
use crate::task_manager::task_context::TaskContext;
use crate::task_manager::Task;

pub struct TaskManager {
    scheduler: Scheduler,
    tasks: SimpleStore<Task>,
    contexts: SimpleStore<TaskContext>,
}

impl TaskManager {
    pub fn add_task(&mut self, task: Task) -> Result<()> {
        // push the task to the scheduler
        // and returns back a unique id
        // which would be later used to cancel a periodic task
        self.tasks.push(task.clone());
        self.scheduler.add_task(task)?;
        Ok(())
    }

    pub fn get_tasks(&self) -> Vec<Task> {
        self.tasks.inner()
    }

    pub fn cancel_task(&mut self, task_id: TaskID) -> Result<()> {
        self.cancel_tasks(vec![task_id])
    }

    fn cancel_tasks(&mut self, task_ids: Vec<TaskID>) -> Result<()> {
        for task_id in task_ids {
            if let Some(task) = self.tasks.remove_first(|t| t.task_id.starts_with(&task_id)) {
                self.scheduler.cancel_task(task)?;
            } else {
                return Err(anyhow!(format!("no such task found: {task_id}")));
            }
        }
        Ok(())
    }

    pub fn refresh(&mut self) -> Result<()> {
        let now = OffsetDateTime::now_utc();
        let before = self.tasks.len();
        self.tasks.retain(|task| match task.clock_type {
            ClockType::Once(next_fire) => next_fire > now,
            _ => true,
        });
        if self.tasks.len() != before {
            self.tasks
                .refresh_storage()
                .context("fail to refresh task store")?;
        }
        self.contexts
            .refresh_storage()
            .context("fail to refresh context store")?;
        Ok(())
    }

    // new returns a new TaskManager
    pub fn new<P>(path: P, mut scheduler: Scheduler) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let task_store_path = path.as_ref().join("task.data");
        let tasks: Vec<Task> = read_items(&task_store_path)
            .context(format!("fail to open task store {task_store_path:?}"))?;

        let context_store_path = path.as_ref().join("task_context.data");
        let mut contexts: Vec<TaskContext> = read_items(&context_store_path)
            .context(format!("fail to open context store {context_store_path:?}"))?;
        if contexts.is_empty() {
            contexts.push(default_context());
        }
        let current_context = current_context(&contexts);
        for task in tasks.iter().filter(|t| t.context == current_context) {
            scheduler.add_task(task.clone())?;
        }
        let tasks = SimpleStore::new(tasks, task_store_path);
        let contexts = SimpleStore::new(contexts, context_store_path);
        let tm = TaskManager {
            scheduler,
            tasks,
            contexts,
        };
        Ok(tm)
    }

    pub fn switch_context(&mut self, new_context: TaskContext) -> Result<()> {
        let current_context = self.current_context();
        if new_context == current_context {
            return Ok(());
        }
        let position = self.contexts.iter().position(|c| c == &new_context);
        if position.is_none() {
            return Err(anyhow!("no such context: {}", &new_context));
        }
        let task_ids: Vec<TaskID> = self
            .tasks
            .iter()
            .filter(|t| t.context == current_context)
            .map(|t| t.task_id.clone())
            .collect();
        self.cancel_tasks(task_ids)?;
        for task in self.tasks.iter().filter(|t| t.context == new_context) {
            self.scheduler.add_task(task.clone())?;
        }
        let index = position.unwrap();
        self.contexts.swap(0, index);
        Ok(())
    }

    pub fn current_context(&self) -> TaskContext {
        current_context(&self.contexts.mem)
    }

    pub fn define_context(&mut self, context: TaskContext) -> Result<()> {
        self.contexts.push(context);
        Ok(())
    }

    pub fn list_context(&self) -> Vec<TaskContext> {
        self.contexts.inner()
    }

    pub fn remove_context(&mut self, context: TaskContext) -> Result<()> {
        let current_context = self.current_context();
        if current_context == context {
            self.switch_context(default_context())?;
        }
        self.contexts.remove_first(|c| c == &context);
        self.tasks.retain(|t| t.context != context);
        Ok(())
    }
}

fn current_context(contexts: &[TaskContext]) -> TaskContext {
    contexts.first().unwrap().clone()
}

pub fn read_items<P, T>(path: P) -> Result<Vec<T>>
where
    P: AsRef<Path>,
    T: DeserializeOwned,
{
    let mut items = vec![];
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&path)?;
    for line in io::BufReader::new(file).lines() {
        let line = line?;
        if line.is_empty() {
            break;
        }
        let item: T = from_str(&line)?;
        items.push(item);
    }
    Ok(items)
}

#[derive(Clone)]
struct SimpleStore<T: Clone + Serialize> {
    mem: Vec<T>,
    persist_path: PathBuf,
}

impl<T: Clone + Serialize> SimpleStore<T> {
    pub fn new(mem: Vec<T>, persist_path: PathBuf) -> Self {
        Self { mem, persist_path }
    }

    pub fn inner(&self) -> Vec<T> {
        self.mem.clone()
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        self.mem.swap(a, b)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.mem.iter()
    }

    pub fn len(&self) -> usize {
        self.mem.len()
    }

    //pub fn first(&self) -> Option<&T> {
    //    self.mem.first()
    //}

    //pub fn last(&self) -> Option<&T> {
    //    self.mem.last()
    //}

    pub fn remove_first<F>(&mut self, filter: F) -> Option<T>
    where
        F: for<'a> Fn(&'a T) -> bool,
    {
        if let Some(index) = self.mem.iter().position(filter) {
            Some(self.mem.swap_remove(index))
        } else {
            None
        }
    }

    pub fn push(&mut self, item: T) {
        self.mem.push(item);
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.mem.retain(f)
    }

    pub fn refresh_storage(&mut self) -> Result<()> {
        let mut writer = OpenOptions::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open(&self.persist_path)
            .context(self.persist_path.to_string_lossy().to_string())?;
        for item in self.mem.iter() {
            writer.write_all(serde_json::to_vec(item)?.as_slice())?;
        }
        writer.flush()?;
        Ok(())
    }
}

use std::sync::Once;
use std::thread::sleep;

use anyhow::{Context, Result};
use task_reminder::scheduler::Scheduler;
use task_reminder::setup_logger;
use task_reminder::task_manager::{manager::TaskManager, ClockType, Task};
use tempfile::tempdir;
use time::{Duration, OffsetDateTime};

static INIT: Once = Once::new();

pub fn setup() {
    INIT.call_once(|| {
        setup_logger();
    });
}

#[test]
fn once_clock() -> Result<()> {
    setup();
    // create a temp empty task store
    let dir = tempdir()?;
    let path = dir.path().join("empty");
    let scheduler = Scheduler::new();
    let mut tm = TaskManager::new(path, scheduler)?;
    let clock_type = ClockType::Once(OffsetDateTime::now_utc() + Duration::seconds(1));
    let new_task = Task::new("just a test".to_owned(), clock_type);
    tm.add_task(new_task)?;

    sleep(std::time::Duration::from_secs(2));

    Ok(())
}

#[test]
fn periodic_clock() -> Result<()> {
    setup();
    // create a temp empty task store
    let dir = tempdir()?;
    let path = dir.path().join("empty");
    let scheduler = Scheduler::new();
    let mut tm = TaskManager::new(path, scheduler)?;
    let clock_type = ClockType::Period("1s".to_owned());
    let new_task = Task::new("".to_owned(), clock_type);
    tm.add_task(new_task)?;

    sleep(std::time::Duration::from_secs(3));

    Ok(())
}

#[test]
fn cancel_clock() -> Result<()> {
    setup();
    // create a temp empty task store
    let dir = tempdir()?;
    let path = dir.path().join("empty");
    let scheduler = Scheduler::new();
    let mut tm = TaskManager::new(path, scheduler)?;
    let clock_type = ClockType::Period("1s".to_owned());
    let new_task = Task::new("".to_owned(), clock_type);
    let task_id = new_task.task_id.clone();
    tm.add_task(new_task)?;

    sleep(std::time::Duration::from_secs(2));

    tm.cancel_task(task_id)
        .context("fail to cancel periodic_clock in test")?;

    sleep(std::time::Duration::from_secs(2));
    Ok(())
}

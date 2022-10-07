use task_reminder::scheduler::Scheduler;
use task_reminder::setup_logger;
use task_reminder::task_manager::{manager::TaskManager, ClockType, Task};

use anyhow::{Context, Result};
use tempfile::tempdir;
use time::{Duration, OffsetDateTime};

use std::thread::sleep;

#[test]
fn once_clock() -> Result<()> {
    setup_logger();
    // create a temp empty task store
    let dir = tempdir()?;
    let path = dir.path().join("empty");
    let scheduler = Scheduler::new();
    let mut tm = TaskManager::new(&path, scheduler)?;
    let clock_type = ClockType::Once(OffsetDateTime::now_utc() + Duration::seconds(1));
    let new_task = Task::new("just a test".to_owned(), clock_type.clone());
    tm.add_task(new_task)?;

    sleep(std::time::Duration::from_secs(2));

    Ok(())
}

#[test]
fn periodic_clock() -> Result<()> {
    setup_logger();
    // create a temp empty task store
    let dir = tempdir()?;
    let path = dir.path().join("empty");
    let scheduler = Scheduler::new();
    let mut tm = TaskManager::new(&path, scheduler)?;
    let clock_type = ClockType::Period(Duration::seconds(1).unsigned_abs());
    let new_task = Task::new("".to_owned(), clock_type.clone());
    tm.add_task(new_task)?;

    sleep(std::time::Duration::from_secs(3));

    Ok(())
}

#[test]
fn cancel_clock() -> Result<()> {
    setup_logger();
    // create a temp empty task store
    let dir = tempdir()?;
    let path = dir.path().join("empty");
    let scheduler = Scheduler::new();
    let mut tm = TaskManager::new(&path, scheduler)?;
    let clock_type = ClockType::Period(Duration::seconds(1).unsigned_abs());
    let new_task = Task::new("".to_owned(), clock_type.clone());
    let task_id = new_task.task_id;
    tm.add_task(new_task)?;

    sleep(std::time::Duration::from_secs(2));

    tm.cancel_task(task_id)
        .context("fail to cancel periodic_clock in test")?;

    sleep(std::time::Duration::from_secs(2));
    Ok(())
}

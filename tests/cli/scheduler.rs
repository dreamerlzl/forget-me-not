use crate::cli::helpers::rm_task;

use super::helpers::{add_task, list_tasks, spawn_test_daemon, TestTask};
use anyhow::Result;
use std::thread::sleep;

#[test]
fn once_clock() -> Result<()> {
    let _guard = spawn_test_daemon("once_clock")?;
    let task = TestTask::new().after("1s".to_owned());
    add_task(&task);
    sleep(std::time::Duration::from_secs(2));
    // the task should have fired and no tasks left
    list_tasks(&vec![]);
    Ok(())
}

#[test]
fn periodic_clock() -> Result<()> {
    let guard = spawn_test_daemon("periodic_clock")?;
    let task = TestTask::new().per("1s".to_owned());
    add_task(&task);
    let tasks = guard.read_tasks()?;
    assert_eq!(tasks.len(), 1);
    sleep(std::time::Duration::from_secs(2));
    // the clock is periodic so the it's still in the list
    list_tasks(&vec![tasks[0].clone()]);
    Ok(())
}

#[test]
fn cancel_periodic_clock() -> Result<()> {
    let guard = spawn_test_daemon("cancel_periodic_clock")?;
    let task = TestTask::new().per("1s".to_owned());
    add_task(&task);
    let tasks = guard.read_tasks()?;
    assert_eq!(tasks.len(), 1);
    sleep(std::time::Duration::from_secs(1));
    // the clock is periodic so the it's still in the list
    list_tasks(&vec![tasks[0].clone()]);
    rm_task(&tasks[0].task_id);
    list_tasks(&vec![]);
    Ok(())
}

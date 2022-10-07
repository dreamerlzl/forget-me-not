use task_reminder::scheduler::Scheduler;
use task_reminder::task_manager::{
    manager::{read_tasks, TaskManager},
    ClockType, Task,
};

use anyhow::Result;
use tempfile::tempdir;
use time::macros::datetime;

// must be run with one thread; o.w. time-rs would fail
#[test]
fn empty_store() -> Result<()> {
    // create a temp empty task store
    let dir = tempdir()?;
    let path = dir.path().join("empty");
    let scheduler = Scheduler::new();
    let mut tm = TaskManager::new(&path, scheduler)?;
    let clock_type = ClockType::Once(datetime!(2022-01-02 11:12:13 +8));
    let new_task = Task::new("".to_owned(), clock_type.clone());
    let new_task_id = new_task.task_id;
    tm.add_task(new_task)?;
    let tasks = read_tasks(&path)?;
    assert_eq!(tasks[0].task_id, new_task_id);
    assert_eq!(tasks[0].clock_type, clock_type);
    Ok(())
}

#[test]
fn remove_task() -> Result<()> {
    // create a temp empty task store
    let dir = tempdir()?;
    let path = dir.path().join("empty");
    let scheduler = Scheduler::new();
    let mut tm = TaskManager::new(&path, scheduler)?;

    let clock_type = ClockType::Once(datetime!(2022-01-02 11:12:13 +8));
    let new_task = Task::new("".to_owned(), clock_type);
    let new_task_id_1 = new_task.task_id;
    tm.add_task(new_task)?;
    let tasks = read_tasks(&path)?;
    assert_eq!(tasks[0].task_id, new_task_id_1);

    let clock_type = ClockType::Once(datetime!(2022-01-02 11:12:14 +8));
    let new_task = Task::new("".to_owned(), clock_type);
    let new_task_id = new_task.task_id;
    tm.add_task(new_task)?;
    let tasks = read_tasks(&path)?;
    assert_eq!(tasks[1].task_id, new_task_id);

    let clock_type = ClockType::Once(datetime!(2022-01-02 11:12:15 +8));
    let new_task = Task::new("".to_owned(), clock_type);
    let new_task_id_3 = new_task.task_id;
    tm.add_task(new_task)?;
    let tasks = read_tasks(&path)?;
    assert_eq!(tasks[2].task_id, new_task_id_3);

    tm.cancel_task(new_task_id)?;

    let tasks = read_tasks(&path)?;
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].task_id, new_task_id_1);
    assert_eq!(tasks[1].task_id, new_task_id_3);
    Ok(())
}

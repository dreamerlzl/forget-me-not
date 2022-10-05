use task_reminder::scheduler::Scheduler;
use task_reminder::task_manager::{
    manager::{read_tasks, TaskManager},
    Task,
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
    let new_task = Task::new("".to_owned(), datetime!(2022-01-02 11:12:13 +8));
    let new_task_id = new_task.task_id;
    tm.add_task(new_task)?;
    let tasks = read_tasks(&path)?;
    assert_eq!(tasks[0].task_id, new_task_id);
    assert_eq!(tasks[0].next_fire, datetime!(2022-01-02 11:12:13 +8));
    Ok(())
}

#[test]
fn remove_task() -> Result<()> {
    // create a temp empty task store
    let dir = tempdir()?;
    let path = dir.path().join("empty");
    let scheduler = Scheduler::new();
    let mut tm = TaskManager::new(&path, scheduler)?;

    let new_task = Task::new("".to_owned(), datetime!(2022-01-02 11:12:13 +8));
    let new_task_id_1 = new_task.task_id;
    tm.add_task(new_task)?;
    let tasks = read_tasks(&path)?;
    assert_eq!(tasks[0].task_id, new_task_id_1);

    let new_task = Task::new("".to_owned(), datetime!(2022-01-02 11:12:14 +8));
    let new_task_id = new_task.task_id;
    let task_id = tm.add_task(new_task)?;
    let tasks = read_tasks(&path)?;
    assert_eq!(tasks[1].task_id, new_task_id);

    let new_task = Task::new("".to_owned(), datetime!(2022-01-02 11:12:15 +8));
    let new_task_id_3 = new_task.task_id;
    tm.add_task(new_task)?;
    let tasks = read_tasks(&path)?;
    assert_eq!(tasks[2].task_id, new_task_id_3);

    tm.remove_task(task_id)?;

    let tasks = read_tasks(&path)?;
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].task_id, new_task_id_1);
    assert_eq!(tasks[1].task_id, new_task_id_3);
    Ok(())
}

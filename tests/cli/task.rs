use anyhow::{anyhow, Result};
use task_reminder::{comm::get_local_now, task_manager::ClockType};

use crate::cli::helpers::list_tasks;

use super::helpers::{add_task, rm_task, spawn_test_daemon, TestTask};

#[test]
fn remove_task() -> Result<()> {
    let guard = spawn_test_daemon("remove_task")?;
    let task1 = TestTask::new().description("rm1");
    let task2 = TestTask::new().description("rm2");
    let task3 = TestTask::new().description("rm3");
    add_task(&task1);
    add_task(&task2);
    add_task(&task3);
    let tasks = guard.read_tasks()?;
    assert_eq!(tasks.len(), 3);
    rm_task(&tasks[1].task_id);
    let tasks = guard.read_tasks()?;
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].description, task1.description.unwrap());
    assert_eq!(tasks[1].description, task3.description.unwrap());
    list_tasks(&tasks);
    Ok(())
}

#[test]
fn check_clock_type_after() -> Result<()> {
    let guard = spawn_test_daemon("check_clock_type")?;
    let task = TestTask::new()
        .description("foo")
        .after("1h0m3s".to_owned());
    add_task(&task);
    let tasks = guard.read_tasks()?;
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].description, task.description.unwrap());
    if let ClockType::Once(moment) = tasks[0].clock_type {
        let now = get_local_now();
        assert_eq!(now.hour() + 1, moment.hour());
    } else {
        return Err(anyhow!("wrong clock type"));
    }
    Ok(())
}

#[test]
fn check_clock_type_at() -> Result<()> {
    let guard = spawn_test_daemon("check_clock_type")?;
    let now = get_local_now();
    let hour = match now.hour() {
        v @ 0..=22 => v + 1,
        _ => 1,
    };
    let task = TestTask::new()
        .description("foo")
        .at(format!("{}:0", hour), false);
    add_task(&task);
    let tasks = guard.read_tasks()?;
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].description, task.description.unwrap());
    if let ClockType::Once(moment) = tasks[0].clock_type {
        assert_eq!(moment.hour(), hour);
        assert_eq!(moment.minute(), 0);
    } else {
        return Err(anyhow!("wrong clock type"));
    }
    Ok(())
}

#[test]
fn check_clock_type_per() -> Result<()> {
    let guard = spawn_test_daemon("check_clock_type")?;
    let input_period = "1h";
    let task = TestTask::new()
        .description("foo")
        .per(input_period.to_owned());
    add_task(&task);
    let tasks = guard.read_tasks()?;
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].description, task.description.unwrap());
    if let ClockType::Period(period) = &tasks[0].clock_type {
        assert_eq!(input_period, period);
    } else {
        return Err(anyhow!("wrong clock type"));
    }
    Ok(())
}

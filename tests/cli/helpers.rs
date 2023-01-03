use std::{io, net::TcpListener, sync::mpsc::SyncSender};

use anyhow::Result;

use assert_cmd::Command;
use log::{error, info};
use predicates::str::diff;
use task_reminder::format::tabular_output;
use task_reminder::task_manager::{read_items, Task, TaskContext};
use task_reminder::{daemon::serve, scheduler::Scheduler, task_manager::TaskManager};
use tempfile::{tempdir, TempDir};

const BINARY_NAME: &str = "fmn";

pub fn fmn(args: &[&str]) -> Command {
    let mut command = Command::cargo_bin(BINARY_NAME).expect("no such binary");
    command.args(args);
    command
}

pub struct DaemonGuard {
    id: String,
    _temp_dir: TempDir,
    stop_chan: Option<SyncSender<()>>,
}

impl DaemonGuard {
    pub fn read_tasks(&self) -> Result<Vec<Task>> {
        read_items(self._temp_dir.path().join("task.data"))
    }

    pub fn read_contexts(&self) -> Result<Vec<TaskContext>> {
        read_items(self._temp_dir.path().join("task_context.data"))
    }

    fn new(id: String, temp_dir: TempDir) -> Self {
        Self {
            stop_chan: None,
            id,
            _temp_dir: temp_dir,
        }
    }
}

impl Drop for DaemonGuard {
    fn drop(&mut self) {
        if let Some(stop_chan) = self.stop_chan.take() {
            match stop_chan.try_send(()) {
                Ok(_) => {
                    info!("successfuly stop fmn-daemon for {}", self.id);
                }
                Err(e) => error!("fail to stop fmn-daemon for {}: {}", self.id, e),
            }
        }
    }
}

pub fn spawn_test_daemon(id: &str) -> Result<DaemonGuard> {
    let id = id.to_owned();
    let fmn_dir = tempdir()?;
    let addr = "127.0.0.1:0";
    std::fs::create_dir_all(&fmn_dir)?;
    let scheduler = Scheduler::new();

    let listener = TcpListener::bind(addr)?;
    listener
        .set_nonblocking(true)
        .expect("can't set tcp listener as non-blocking");
    let dest = listener.local_addr()?.to_string();
    std::env::set_var("FMN_DAEMON_ADDR", &dest);
    info!("creating fmn-daemon for {} at {}", id, dest);
    let mut tm = TaskManager::new(&fmn_dir, scheduler)?;
    let mut guard = DaemonGuard::new(id, fmn_dir);
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if let Err(e) = serve(stream, &mut tm) {
                        error!("error processing stream: {}", e);
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    if rx.try_recv().is_ok() {
                        return;
                    }
                    continue;
                }
                Err(e) => error!("error getting stream:{}", e),
            }
        }
    });
    guard.stop_chan = Some(tx);
    Ok(guard)
}

pub fn add_task(task: &TestTask) {
    fmn(&task.to_args()).assert().success();
}

pub fn rm_task(id: &str) {
    fmn(&["rm", id]).assert().success();
}

pub fn list_tasks(tasks: &Vec<Task>) {
    let expected_output = format!("{}\n", tabular_output(tasks));
    fmn(&["list"]).assert().stdout(diff(expected_output));
}

enum AddCommand {
    After { duration: String },
    At { time: String, per_day: bool },
    Per { duration: String },
}

pub struct TestTask<'a> {
    pub description: Option<&'a str>,
    clock_type: AddCommand,
}

const DEFAULT_TASK_NAME: &str = "foo";

impl<'a> TestTask<'a> {
    pub fn to_args(&self) -> Vec<&str> {
        match &self.clock_type {
            AddCommand::After { duration } => {
                vec![
                    "add",
                    self.description.unwrap_or(DEFAULT_TASK_NAME),
                    "after",
                    duration,
                ]
            }
            AddCommand::At { time, per_day } => {
                if *per_day {
                    vec![
                        "add",
                        self.description.unwrap_or(DEFAULT_TASK_NAME),
                        "at",
                        time,
                        "-p",
                    ]
                } else {
                    vec![
                        "add",
                        self.description.unwrap_or(DEFAULT_TASK_NAME),
                        "at",
                        time,
                    ]
                }
            }
            AddCommand::Per { duration } => {
                vec![
                    "add",
                    self.description.unwrap_or(DEFAULT_TASK_NAME),
                    "per",
                    duration,
                ]
            }
        }
    }

    pub fn new() -> Self {
        Self {
            description: None,
            clock_type: AddCommand::After {
                duration: "1h".to_owned(),
            },
        }
    }

    pub fn after(mut self, duration: String) -> Self {
        self.clock_type = AddCommand::After { duration };
        self
    }

    pub fn at(mut self, time: String, per_day: bool) -> Self {
        self.clock_type = AddCommand::At { time, per_day };
        self
    }

    pub fn per(mut self, duration: String) -> Self {
        self.clock_type = AddCommand::Per { duration };
        self
    }

    pub fn description(mut self, name: &'a str) -> Self {
        self.description = Some(name);
        self
    }
}

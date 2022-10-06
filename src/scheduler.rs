use crate::notify::desktop_notification;
use crate::task_manager::{ClockType, Task, TaskID};

use std::collections::HashMap;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use log::{debug, error, info, warn};
use time::OffsetDateTime;
use tokio::runtime::{Builder, Runtime};
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::mpsc;
use tokio::time::sleep;

const SUMMARY: &str = "task-reminder";

pub struct Scheduler {
    task_sender: mpsc::Sender<SchedulerCommand>,
}

pub struct InnerScheduler {
    cancel_channels: HashMap<TaskID, broadcast::Sender<TaskCommand>>,
}

#[derive(Debug)]
enum SchedulerCommand {
    Add(Task),
    Cancel(Task),
}

#[derive(Clone, Debug)]
enum TaskCommand {
    Stop,
}

impl Scheduler {
    pub fn new() -> Self {
        // if we use multi-threaded runtime, time-rs or chrono's now method is not reliable
        let (sender, receiver) = mpsc::channel(8);
        std::thread::spawn(
            move || match Builder::new_current_thread().enable_all().build() {
                Ok(rt) => {
                    let mut inner = InnerScheduler::new();
                    inner.start(rt, receiver);
                }
                Err(e) => {
                    error!("fail to create async runtime: {}", e)
                }
            },
        );
        Scheduler {
            task_sender: sender,
        }
    }
    pub fn add_task(&mut self, task: Task) -> Result<()> {
        if self.check_inner_scheduler_crashed() {
            panic!("the inner scheduler has paniced!");
        }
        let clock_type = task.clock_type.clone();
        match self.task_sender.blocking_send(SchedulerCommand::Add(task)) {
            Ok(()) => {
                debug!(
                    "successfully send new task to inner scheduler: {}",
                    clock_type
                );
                Ok(())
            }
            Err(e) => Err(anyhow!("fail to send new task to inner scheduler: {}", e)),
        }
    }

    pub fn cancel_task(&self, task: Task) -> Result<()> {
        if self.check_inner_scheduler_crashed() {
            panic!("the inner scheduler has paniced!");
        }
        let task_id = task.task_id.clone();
        match self
            .task_sender
            .blocking_send(SchedulerCommand::Cancel(task))
        {
            Ok(()) => {
                debug!(
                    "successfully cancel new task to inner scheduler: {}",
                    task_id
                );
                Ok(())
            }
            Err(e) => Err(anyhow!(
                "fail to send cancel task to inner scheduler: {}",
                e
            )),
        }
    }

    fn check_inner_scheduler_crashed(&self) -> bool {
        self.task_sender.is_closed()
    }
}

impl InnerScheduler {
    fn new() -> Self {
        InnerScheduler {
            cancel_channels: HashMap::new(),
        }
    }

    fn start(&mut self, rt: Runtime, mut task_receiver: mpsc::Receiver<SchedulerCommand>) {
        rt.block_on(async {
            while let Some(scheduler_command) = task_receiver.recv().await {
                match scheduler_command {
                    SchedulerCommand::Add(task) => {
                        self.add_task(task);
                    }
                    SchedulerCommand::Cancel(task_id) => {
                        if let Err(e) = self.cancel_task(task_id) {
                            // no active receivers
                            // https://docs.rs/tokio/latest/tokio/sync/broadcast/error/struct.SendError.html
                            error!("fail to cancel task: {}", e);
                        }
                    }
                }
            }
        });
    }

    pub fn add_task(&mut self, task: Task) {
        let task_id = task.task_id;
        let clock_type = task.clock_type;
        let description = task.description;
        info!("add new clock task: {}, {}", task_id, clock_type);
        let (sender, receiver) = broadcast::channel(1);
        self.cancel_channels.insert(task_id, sender);
        // enter the tokio rt context so that we can use tokio::spawn
        match clock_type {
            ClockType::Once(next_fire) => {
                tokio::spawn(once_clock(description, next_fire, receiver))
            }
            ClockType::Period(period) => tokio::spawn(period_clock(description, period, receiver)),
        };
    }

    pub fn cancel_task(&mut self, task: Task) -> Result<()> {
        let task_id = task.task_id;
        if let Some(sender) = self.cancel_channels.get(&task_id) {
            if let Err(e) = sender
                .send(TaskCommand::Stop)
                .context("fail to send stop to clock")
            {
                // no active receivers
                return Err(anyhow!("fail to send cancel task: {}", e));
            }
        } else {
            warn!("fail to find sender channel for task id: {}", task_id);
        }
        Ok(())
    }
}

async fn period_clock(
    description: String,
    period: Duration,
    mut receiver: broadcast::Receiver<TaskCommand>,
) {
    loop {
        tokio::select! {
            val = receiver.recv() => {
                if is_canceled(val) {
                    info!("periodic task with period {:?} is cancelled!", period);
                    return
                }
            }
            _ = sleep(period) => {
                info!("a periodic clock fire!");
                if let Err(e) = desktop_notification(SUMMARY, &description) {
                    error!("fail to send de notification: {}", e);
                    return
                }
            }
        }
    }
}

async fn once_clock(
    description: String,
    next_fire: OffsetDateTime,
    mut receiver: broadcast::Receiver<TaskCommand>,
) {
    let now = OffsetDateTime::now_utc();
    if now >= next_fire {
        warn!(
            "clock next_fire time {} shouldn't be in the past!",
            next_fire
        );
        return;
    }
    let duration = (next_fire - now).unsigned_abs();
    tokio::select! {
        val = receiver.recv() => {
            if is_canceled(val) {
                info!("once clock with next_fire {:?} is cancelled!", next_fire);
                return
            }
        }
        _ = sleep(duration) => {
            info!("a clock fire!");
            if let Err(e) = desktop_notification(SUMMARY, &description) {
                error!("fail to send notification: {}", e);
            }
        }
    }
}

fn is_canceled(val: std::result::Result<TaskCommand, RecvError>) -> bool {
    match val {
        Ok(command) => match command {
            TaskCommand::Stop => true,
        },
        Err(e) => {
            error!("fail to receive command: {}", e);
            true
        }
    }
}

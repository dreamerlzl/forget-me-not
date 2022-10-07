use task_reminder::comm::{Request, Response};
use task_reminder::scheduler::Scheduler;
use task_reminder::task_manager::{Task, TaskManager};

use anyhow::{Context, Result};
use log::{error, info};
use serde_json::{from_slice, to_string};

use std::env;
use std::{net::UdpSocket, path::Path};

fn main() -> Result<()> {
    task_reminder::setup_logger();
    let addr = env::var("REMINDER_DAEMON_ADDR").unwrap_or_else(|_| "127.0.0.1:8082".to_owned());
    let scheduler = Scheduler::new();
    let tm = TaskManager::new("/home/wright/reminder", scheduler)?;
    start_listen(&addr, tm)?;
    Ok(())
}

fn start_listen<P: AsRef<Path>>(addr: &str, mut tm: TaskManager<P>) -> Result<()> {
    let socket = UdpSocket::bind(addr).context("fail to create udp server socket")?;
    let mut buf = [0; 128];
    loop {
        let (amt, src) = socket
            .recv_from(&mut buf)
            .context("fail to receive udp packet")?;
        let request: Request =
            from_slice(&buf[..amt]).context("fail to deserialize a udp request")?;
        let response = match request {
            Request::Add(description, clock_type) => {
                match tm.add_task(Task::new(description, clock_type)) {
                    Err(e) => {
                        error!("fail to add new task in udp server: {}", e);
                        Response::Fail(e.to_string())
                    }
                    Ok(index) => {
                        info!("successfully add task with index: {}", index);
                        Response::AddSuccess(index)
                    }
                }
            }
            Request::Cancel(index) => {
                if let Err(e) = tm.cancel_task(index) {
                    error!("fail to cancel task with index %d: {}", e);
                    Response::Fail(e.to_string())
                } else {
                    Response::CancelSuccess
                }
            }
        };
        let serialized = to_string(&response).expect("fail to serialize response");
        match socket.send_to(serialized.as_bytes(), &src) {
            Ok(_) => {
                info!("successful response: {}", serialized);
            }
            Err(e) => {
                error!("fail to send back response: {}", e);
            }
        }
    }
}

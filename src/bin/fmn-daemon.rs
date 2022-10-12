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
    let addr = env::var("FMN_DAEMON_ADDR").unwrap_or_else(|_| "127.0.0.1:8082".to_owned());
    let scheduler = Scheduler::new();
    let path = env::var("FMN_TASK_STORE")
        .unwrap_or_else(|_| format!("{}/reminder", env::var("HOME").unwrap()));
    let tm = TaskManager::new(&path, scheduler)?;
    start_listen(&addr, tm)?;
    Ok(())
}

fn start_listen<P: AsRef<Path>>(addr: &str, mut tm: TaskManager<P>) -> Result<()> {
    let socket = UdpSocket::bind(addr).context("fail to create udp server socket")?;
    let mut buf = [0; 1024];
    loop {
        let (amt, src) = socket
            .recv_from(&mut buf)
            .context("fail to receive udp packet")?;

        let request: Request = from_slice(&buf[..amt]).context(format!(
            "fail to deserialize a udp request {:?}",
            &buf[..amt]
        ))?;
        let response = if let Err(e) = tm.refresh() {
            error!("fail to refresh task store: {}", e);
            Response::Fail(format!("fail to refresh task store: {}", e))
        } else {
            match request {
                Request::Add(description, clock_type, image_path, sound_path) => {
                    let mut task = Task::new(description, clock_type);
                    if let Some(image_path) = image_path {
                        task.add_image(image_path);
                    }
                    if let Some(sound_path) = sound_path {
                        task.add_sound(sound_path);
                    }
                    match tm.add_task(task) {
                        Err(e) => {
                            error!("fail to add new task in udp server: {}", e);
                            Response::Fail(e.to_string())
                        }
                        Ok(_) => Response::AddSuccess,
                    }
                }
                Request::Cancel(task_id) => {
                    if let Err(e) = tm.cancel_task(task_id) {
                        error!("fail to cancel task with index %d: {}", e);
                        Response::Fail(e.to_string())
                    } else {
                        Response::CancelSuccess
                    }
                }
                Request::Show => Response::GetTasks(tm.get_tasks()),
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

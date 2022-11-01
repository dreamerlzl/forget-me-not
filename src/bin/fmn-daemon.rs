use task_reminder::comm::{Request, Response};
use task_reminder::scheduler::Scheduler;
use task_reminder::task_manager::{Task, TaskManager};

use anyhow::{Context, Result};
use log::{error, info};
use serde_json::{to_string, Deserializer};

use std::env;
use std::io::{BufReader, BufWriter, Write};
use std::net::TcpListener;
use std::{net::TcpStream, path::Path};

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
    let listener = TcpListener::bind(addr).context("fail to create udp server socket")?;
    for stream in listener.incoming() {
        serve(stream?, &mut tm)?;
    }
    Ok(())
}

fn serve<P: AsRef<Path>>(stream: TcpStream, tm: &mut TaskManager<P>) -> Result<()> {
    let reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    let requests = Deserializer::from_reader(reader).into_iter::<Request>();
    for request in requests {
        let request = request?;
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
        match writer.write_all(serialized.as_bytes()) {
            Ok(_) => {
                info!("successful response: {}", serialized);
            }
            Err(e) => {
                error!("fail to send back response: {}", e);
            }
        }
        writer
            .flush()
            .context("fail to flush fmn-daemon tcp writer")?;
    }
    Ok(())
}

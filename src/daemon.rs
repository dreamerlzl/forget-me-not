use std::io::{BufReader, BufWriter, Write};
use std::net::TcpStream;

use anyhow::{Context, Result};
use log::{error, info};
use serde_json::{to_string, Deserializer};

use crate::comm::{ContextCommand, Request, Response};
use crate::task_manager::{Task, TaskManager};

pub fn serve(stream: TcpStream, tm: &mut TaskManager) -> Result<()> {
    let reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    let requests = Deserializer::from_reader(reader).into_iter::<Request>();
    for request in requests {
        let request = request?;
        info!("receive a request: {:?}", request);
        tm.refresh_before();
        let response = {
            match request {
                Request::Add(description, clock_type, image_path, sound_path) => {
                    let mut task =
                        Task::new(description, clock_type).with_context(tm.current_context());
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
                        Response::RemoveSuccess
                    }
                }
                Request::Show => Response::GetTasks(tm.get_tasks()),
                Request::ContextRequest(command) => handle_context_command(command, tm),
            }
        };
        if let Err(e) = tm.refresh_after() {
            error!("fail to flush changes to persistent storage: {e}");
        }
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

fn handle_context_command(command: ContextCommand, tm: &mut TaskManager) -> Response {
    match command {
        ContextCommand::Define { context } => {
            if let Err(e) = tm.define_context(context) {
                Response::Fail(e.to_string())
            } else {
                Response::AddSuccess
            }
        }
        ContextCommand::List => Response::GetContexts(tm.list_context()),
        ContextCommand::Rm { context } => {
            if let Err(e) = tm.remove_context(context) {
                Response::Fail(e.to_string())
            } else {
                Response::RemoveSuccess
            }
        }
        ContextCommand::Set { context } => {
            if let Err(e) = tm.switch_context(context) {
                Response::Fail(e.to_string())
            } else {
                Response::SetContextSuccess
            }
        }
    }
}

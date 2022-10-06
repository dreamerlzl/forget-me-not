// for convience of cross-platform
// we use udp socket for the daemon
use crate::task_manager::{Task, TaskManager};

use std::{net::UdpSocket, path::Path};

use anyhow::{Context, Result};
use log::{error, info};
use serde_json::{from_slice, to_string};

use super::{Request, Response};

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

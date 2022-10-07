use anyhow::{Context, Result};
use serde_json::{from_slice, to_string};
use std::env;
use std::net::{Ipv4Addr, UdpSocket};
use time::macros::datetime;
// use time::OffsetDateTime;

use task_reminder::comm::{Request, Response};
use task_reminder::task_manager::ClockType;

fn main() {
    let dest = env::var("REMINDER_DAEMON_ADDR").unwrap_or_else(|_| "127.0.0.1:8082".to_owned());
    let request = Request::Add(
        "fuck".to_owned(),
        ClockType::Once(datetime!(2022-10-07 10:32:20 +8)),
    );
    match send_request(request.clone(), &dest) {
        Ok(response) => {
            println!("success: {:?}", response);
        }
        Err(e) => {
            println!("fail to remind task {:?}: {}", request, e);
        }
    }
}

fn send_request(request: Request, dest: &str) -> Result<Response> {
    let socket =
        UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).context("fail to bind to a random port")?;
    let serialized = to_string(&request).expect("fail to serialize request");
    socket
        .send_to(serialized.as_bytes(), dest)
        .context(format!("fail to send to {}", dest))?;
    let mut buf = [0; 128];
    let amt = socket.recv(&mut buf)?;
    let response: Response = from_slice(&buf[..amt]).context("fail to deserialize response")?;
    Ok(response)
}

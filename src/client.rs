use std::io::{BufReader, Write};
use std::net::TcpStream;

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{to_string, Deserializer};

use crate::comm::{Request, Response};

pub fn send_request(request: Request, dest: &str) -> Result<Response> {
    let mut stream =
        TcpStream::connect(dest).context(format!("fail to connect to fmn-deamon: {dest}"))?;
    let serialized = to_string(&request).expect("fail to serialize request");
    stream
        .write_all(serialized.as_bytes())
        .context("fail to send requests to fmn-daemon")?;

    let mut reader = Deserializer::from_reader(BufReader::new(stream.try_clone()?));
    let response: Response =
        Response::deserialize(&mut reader).context("fail to deserialize response")?;
    Ok(response)
}

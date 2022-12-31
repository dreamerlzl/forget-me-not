use std::env;
use std::net::TcpListener;

use anyhow::{Context, Result};
use task_reminder::daemon::serve;
use task_reminder::scheduler::Scheduler;
use task_reminder::task_manager::TaskManager;

fn main() -> Result<()> {
    task_reminder::setup_logger();
    let addr = env::var("FMN_DAEMON_ADDR").unwrap_or_else(|_| "127.0.0.1:8082".to_owned());
    let fmn_dir =
        env::var("FMN_DIR").unwrap_or_else(|_| format!("{}/.fmn", env::var("HOME").unwrap()));
    spawn_daemon(addr, fmn_dir)
}

pub fn spawn_daemon(addr: String, fmn_dir: String) -> Result<()> {
    std::fs::create_dir_all(&fmn_dir)?;
    let scheduler = Scheduler::new();
    let tm = TaskManager::new(&fmn_dir, scheduler)?;
    start_listen(&addr, tm)?;
    Ok(())
}

fn start_listen(addr: &str, mut tm: TaskManager) -> Result<()> {
    let listener = TcpListener::bind(addr).context("fail to create server socket")?;
    for stream in listener.incoming() {
        serve(stream?, &mut tm)?;
    }
    Ok(())
}

use std::{io, net::TcpListener, sync::mpsc::SyncSender};

use anyhow::Result;
use assert_cmd::Command;
use log::{error, info};
use predicates::str::diff;
use task_reminder::{daemon::serve, scheduler::Scheduler, task_manager::TaskManager};
use tempfile::{tempdir, TempDir};

const BINARY_NAME: &str = "fmn";

#[test]
fn test_define_context() -> Result<()> {
    let _guard = spawn_test_daemon("test_define_context")?;
    define_context("foo");
    set_context("foo");
    list_context(vec!["foo", "default"]);
    Ok(())
}

#[test]
fn test_rm_context() -> Result<()> {
    let _guard = spawn_test_daemon("test_rm_context")?;
    define_context("foo");
    set_context("foo");
    rm_context("foo");
    list_context(vec!["default"]);
    Ok(())
}

fn fmn(args: &[&str]) -> Command {
    let mut command = Command::cargo_bin(BINARY_NAME).expect("no such binary");
    command.args(args);
    command
}

fn define_context(context: &str) {
    fmn(&["context", "define", context]).assert().success();
}

fn set_context(context: &str) {
    fmn(&["context", "set", context]).assert().success();
}

fn list_context(context: Vec<&str>) {
    let output = format!(" * {}\n", context.join("\n   "));
    fmn(&["context", "list"]).assert().stdout(diff(output));
}

fn rm_context(context: &str) {
    fmn(&["context", "rm", context]).assert().success();
}

struct DaemonGuard {
    id: String,
    _temp_dir: TempDir,
    stop_chan: Option<SyncSender<()>>,
}

impl DaemonGuard {
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

fn spawn_test_daemon(id: &str) -> Result<DaemonGuard> {
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

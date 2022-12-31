#![forbid(unsafe_code)]

pub mod client;
pub mod comm;
pub mod daemon;
pub mod notify;
pub mod scheduler;
pub mod task_manager;

use env_logger::Env;
use log::debug;

pub fn setup_logger() {
    let env = Env::default().filter_or("FMN_DAEMON_LOG_LEVEL", "debug");
    env_logger::init_from_env(env);
    debug!("logger start");
}

#![forbid(unsafe_code)]

pub mod client;
pub mod comm;
pub mod daemon;
pub mod format;
pub mod notify;
pub mod scheduler;
pub mod task_manager;

use comm::get_local_now;
use log::{debug, LevelFilter};
use std::{io::Write, str::FromStr};
use time::format_description::well_known::Rfc3339;

pub fn setup_logger() {
    let log_level_str =
        std::env::var("FMN_DAEMON_LOG_LEVEL").unwrap_or_else(|_| "debug".to_owned());
    let log_level = LevelFilter::from_str(&log_level_str)
        .unwrap_or_else(|_| panic!("unknown log level: {}", log_level_str));
    env_logger::Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{}:{} {} [{}] - {}",
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                get_local_now().format(&Rfc3339).unwrap(),
                record.level(),
                record.args()
            )
        })
        .filter(None, log_level)
        .init();
    debug!("logger start");
}

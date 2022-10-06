use anyhow::{anyhow, Result};
use notify_rust::Notification;

pub fn desktop_notification(summary: &str, body: &str) -> Result<()> {
    Notification::new()
        .summary(summary)
        .body(body)
        .appname("task-reminder")
        .show()
        .map_err(|e| anyhow!("fail to show notification to de: {}", e))
        .map(|_| ())
}

use anyhow::{anyhow, Result};
use log::info;
use notify_rust::Notification;

use std::env;
use std::process::Command;

pub fn desktop_notification(summary: &str, body: &str) -> Result<()> {
    let mut notification = Notification::new();
    notification.summary(summary).body(body);

    if let Ok(image_path) = env::var("REMINDER_IMAGE_PATH") {
        info!("add image path hint: {}", &image_path);
        add_image(&mut notification, &image_path);
    }
    if let Ok(sound_path) = env::var("REMINDER_SOUND_PATH") {
        info!("add sound path hint: {}", &sound_path);
        play_sound(&sound_path);
    }

    notification
        .show()
        .map_err(|e| anyhow!("fail to show notification to de: {}", e))
        .map(|_| ())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn add_image(notification: &mut Notification, image_path: &str) {
    notification.image_path(image_path);
}

#[cfg(all(unix, not(target_os = "macos")))]
fn play_sound(sound_path: &str) {
    Command::new("cvlc")
        .arg("--play-and-exit")
        .arg(sound_path)
        .spawn()
        .ok();
}

#[cfg(target_os = "macos")]
fn add_image(notification: &mut Notification, image_path: &str) {
    info!("macOS doesn't support attach images to notifications");
}

#[cfg(target_os = "macos")]
fn play_sound(sound_path: &str) {
    Command::new("afplay").arg(sound_path).spawn().ok();
}

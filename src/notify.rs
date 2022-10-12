use anyhow::{anyhow, Result};
use log::{error, info};
use notify_rust::Notification;

use std::process::Command;

pub fn desktop_notification(
    summary: &str,
    body: &str,
    image_path: Option<&str>,
    sound_path: Option<&str>,
) -> Result<()> {
    let mut notification = Notification::new();
    notification.summary(summary).body(body);

    if let Some(image_path) = image_path {
        info!("add image path hint: {}", &image_path);
        add_image(&mut notification, &image_path);
    }
    if let Some(sound_path) = sound_path {
        info!("add sound path hint: {}", &sound_path);
        if let Err(e) = play_sound(&sound_path) {
            error!("fail to play sound {}: {}", &sound_path, e);
        }
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
fn play_sound(sound_path: &str) -> Result<()> {
    Command::new("cvlc")
        .arg("--play-and-exit")
        .arg(sound_path)
        .spawn()?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn add_image(_notification: &mut Notification, _image_path: &str) {
    info!("macOS doesn't support attach images to notifications");
}

#[cfg(target_os = "macos")]
fn play_sound(sound_path: &str) -> Result<()> {
    Command::new("afplay").arg(sound_path).spawn()?;
    Ok(())
}

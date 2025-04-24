use anyhow::{anyhow, Result};
use notify_rust::{CloseReason, Hint, Notification, NotificationHandle, Timeout};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{sleep, spawn},
};

use crate::bz::pairing::PairingConfirmationHandler;
use crate::icons::Icons;

pub struct NotificationManager {
    icons: Arc<Icons>,
    handles: Arc<Mutex<HashMap<u32, NotificationHandle>>>,
}

impl PairingConfirmationHandler for NotificationManager {
    fn request_confirmation(
        &self,
        device_address: &str,
        passkey: &str,
        on_confirm: Box<dyn FnOnce() + Send>,
        on_reject: Box<dyn FnOnce() + Send>,
    ) -> Result<()> {
        self.send_pairing_confirmation(device_address, passkey, on_confirm, on_reject)
    }
}

impl NotificationManager {
    pub fn new(icons: Arc<Icons>) -> Self {
        Self {
            icons,
            handles: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_icons_default() -> Self {
        Self::new(Arc::new(Icons::default()))
    }

    pub fn send_notification(
        &self,
        summary: Option<String>,
        body: Option<String>,
        icon: Option<&str>,
        timeout: Option<Timeout>,
    ) -> Result<u32> {
        let icon_name = self.icons.get_xdg_icon(icon.unwrap_or("bluetooth"));

        let mut notification = Notification::new();
        notification
            .summary(summary.as_deref().unwrap_or("BlueZ Menu"))
            .body(body.as_deref().unwrap_or(""))
            .icon(&icon_name)
            .timeout(timeout.unwrap_or(Timeout::Milliseconds(3000)));

        let handle = notification.show()?;
        let id = handle.id();

        let mut handles = self
            .handles
            .lock()
            .map_err(|e| anyhow!("Failed to acquire lock on notification handles: {}", e))?;
        handles.insert(id, handle);

        Ok(id)
    }

    pub fn close_notification(&self, id: u32) -> Result<()> {
        let mut handles = self
            .handles
            .lock()
            .map_err(|e| anyhow!("Failed to acquire lock on notification handles: {}", e))?;

        if let Some(handle) = handles.remove(&id) {
            handle.close();
            Ok(())
        } else {
            Err(anyhow!("Notification ID {} not found", id))
        }
    }

    pub fn send_pairing_confirmation(
        &self,
        device_address: &str,
        passkey: &str,
        on_confirm: impl FnOnce() + Send + 'static,
        on_reject: impl FnOnce() + Send + 'static,
    ) -> Result<()> {
        let icon_name = self.icons.get_xdg_icon("bluetooth");

        let summary = t!("menus.bluetooth.pairing_request");
        let body = t!(
            "menus.bluetooth.confirm_passkey",
            device_name = device_address,
            passkey = passkey
        );
        let confirm_text = t!("menus.bluetooth.confirm");
        let cancel_text = t!("menus.bluetooth.cancel");

        let mut binding = Notification::new();
        let notification = binding
            .summary(&summary)
            .body(&body)
            .icon(&icon_name)
            .timeout(Timeout::Milliseconds(30000))
            .action("default", &confirm_text)
            .action("confirm", &confirm_text)
            .action("reject", &cancel_text);

        match notification.show() {
            Ok(handle) => {
                spawn(move || {
                    handle.wait_for_action(|action| match action {
                        "default" | "confirm" => on_confirm(),
                        "reject" | "__closed" => on_reject(),
                        _ => on_reject(),
                    });
                });
                Ok(())
            }
            Err(err) => Err(anyhow!("Failed to show notification: {}", err)),
        }
    }

    // TODO: Follow https://github.com/hoodie/notify-rust/issues/199
    // "Allow an on_close handler without consuming the NotificationHandle"
    // This would simplify our implementation by avoiding the need for a separate thread
    // and allowing us to update the notification directly after setting up the on_close handler.
    pub fn send_progress_notification(
        &self,
        duration_sec: u64,
        on_cancel: impl FnOnce() + Send + 'static,
        progress_summary: Option<String>,
        progress_body: Option<String>,
        progress_icon: Option<&str>,
        completion_summary: Option<String>,
        completion_body: Option<String>,
        completion_icon: Option<&str>,
    ) -> Result<u32> {
        let progress_icon = self
            .icons
            .get_xdg_icon(progress_icon.unwrap_or("bluetooth"));
        let progress_summary = progress_summary.unwrap_or_else(|| String::from("BlueZ Menu"));
        let progress_body = progress_body.unwrap_or_else(|| String::from(""));

        let completion_icon = self
            .icons
            .get_xdg_icon(completion_icon.unwrap_or("bluetooth"));
        let completion_summary = completion_summary.unwrap_or_else(|| String::from("BlueZ Menu"));
        let completion_body = completion_body.unwrap_or_else(|| String::from(""));

        let notification_handle = Notification::new()
            .summary(&progress_summary)
            .body(&progress_body)
            .icon(&progress_icon)
            .timeout(Timeout::Never)
            .hint(Hint::Transient(true))
            .hint(Hint::Category("progress".to_string()))
            .hint(Hint::CustomInt("value".to_string(), 0))
            .show()?;

        let id = notification_handle.id();

        spawn({
            let progress_summary = progress_summary.clone();
            let progress_body = progress_body.clone();
            let progress_icon = progress_icon.clone();

            let completion_icon = completion_icon.clone();
            let completion_summary = completion_summary.clone();
            let completion_body = completion_body.clone();

            let on_cancel_wrapped = Arc::new(Mutex::new(Some(Box::new(on_cancel))));

            move || {
                let start_time = std::time::Instant::now();
                let update_interval = std::time::Duration::from_millis(500);
                let total_duration = std::time::Duration::from_secs(duration_sec);

                let cancelled = Arc::new(AtomicBool::new(false));
                let cancelled_for_loop = cancelled.clone();

                let on_cancel_for_close = on_cancel_wrapped.clone();
                let cancelled_for_close = cancelled.clone();

                spawn(move || {
                    notification_handle.on_close(|reason| {
                        if let CloseReason::Dismissed = reason {
                            if let Ok(mut callback_opt) = on_cancel_for_close.lock() {
                                if let Some(callback) = callback_opt.take() {
                                    callback();
                                }
                            }
                            cancelled_for_close.store(true, Ordering::SeqCst);
                        }
                    });
                });

                while !cancelled_for_loop.load(Ordering::SeqCst) {
                    let elapsed = start_time.elapsed();
                    if elapsed >= total_duration {
                        break;
                    }

                    let progress =
                        ((elapsed.as_secs_f64() / total_duration.as_secs_f64()) * 100.0) as i32;

                    let update_result = Notification::new()
                        .id(id)
                        .summary(&progress_summary)
                        .body(&progress_body)
                        .icon(&progress_icon)
                        .timeout(Timeout::Never)
                        .hint(Hint::Transient(true))
                        .hint(Hint::Category("progress".to_string()))
                        .hint(Hint::CustomInt("value".to_string(), progress.clamp(0, 100)))
                        .show();

                    if update_result.is_err() {
                        if let Ok(mut callback_opt) = on_cancel_wrapped.lock() {
                            if let Some(callback) = callback_opt.take() {
                                callback();
                            }
                        }
                        break;
                    }

                    sleep(update_interval);
                }

                if start_time.elapsed() >= total_duration {
                    let _ = Notification::new()
                        .id(id)
                        .summary(&completion_summary)
                        .body(&completion_body)
                        .icon(&completion_icon)
                        .timeout(Timeout::Milliseconds(2000))
                        .show();
                }
            }
        });

        Ok(id)
    }
}

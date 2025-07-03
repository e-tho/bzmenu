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

impl Clone for NotificationManager {
    fn clone(&self) -> Self {
        Self {
            icons: Arc::clone(&self.icons),
            handles: Arc::clone(&self.handles),
        }
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
        id: Option<u32>,
    ) -> Result<u32> {
        let icon_name = self.icons.get_xdg_icon(icon.unwrap_or("bluetooth"));

        let mut notification = Notification::new();
        notification
            .summary(summary.as_deref().unwrap_or("BlueZ Menu"))
            .body(body.as_deref().unwrap_or(""))
            .icon(&icon_name)
            .timeout(timeout.unwrap_or(Timeout::Milliseconds(3000)));

        if let Some(notification_id) = id {
            notification.id(notification_id);
        }

        let handle = notification.show()?;
        let notification_id = handle.id();

        let mut handles = self
            .handles
            .lock()
            .map_err(|e| anyhow!("Failed to acquire lock on notification handles: {e}"))?;
        handles.insert(notification_id, handle);

        Ok(notification_id)
    }

    pub fn close_notification(&self, id: u32) -> Result<()> {
        let mut handles = self
            .handles
            .lock()
            .map_err(|e| anyhow!("Failed to acquire lock on notification handles: {e}"))?;

        if let Some(handle) = handles.remove(&id) {
            handle.close();
            Ok(())
        } else {
            Err(anyhow!("Notification ID {id} not found"))
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
            Err(err) => Err(anyhow!("Failed to show notification: {err}")),
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
        progress_body: String,
        progress_icon: Option<&str>,
    ) -> Result<u32> {
        let notification_handle = Notification::new()
            .summary("BlueZ Menu")
            .body(&progress_body)
            .icon(
                &self
                    .icons
                    .get_xdg_icon(progress_icon.unwrap_or("bluetooth")),
            )
            .timeout(Timeout::Never)
            .hint(Hint::Transient(true))
            .hint(Hint::Category("progress".to_string()))
            .hint(Hint::CustomInt("value".to_string(), 0))
            .show()?;

        let id = notification_handle.id();

        let notification_manager = self.clone();
        let progress_body_clone = progress_body.clone();
        let progress_icon_str = progress_icon.map(String::from);

        spawn(move || {
            notification_manager.track_progress(
                id,
                duration_sec,
                notification_handle,
                on_cancel,
                progress_body_clone,
                progress_icon_str.as_deref(),
            );
        });

        Ok(id)
    }

    fn track_progress(
        &self,
        id: u32,
        duration_sec: u64,
        notification_handle: NotificationHandle,
        on_cancel: impl FnOnce() + Send + 'static,
        progress_body: String,
        progress_icon: Option<&str>,
    ) {
        let start_time = std::time::Instant::now();
        let update_interval = std::time::Duration::from_millis(500);
        let total_duration = std::time::Duration::from_secs(duration_sec);

        let cancelled = Arc::new(AtomicBool::new(false));
        let cancelled_for_loop = cancelled.clone();

        let on_cancel_wrapped = Arc::new(Mutex::new(Some(Box::new(on_cancel))));
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

            let progress = ((elapsed.as_secs_f64() / total_duration.as_secs_f64()) * 100.0) as i32;

            let update_result = Notification::new()
                .id(id)
                .summary("BlueZ Menu")
                .body(&progress_body)
                .icon(
                    &self
                        .icons
                        .get_xdg_icon(progress_icon.unwrap_or("bluetooth")),
                )
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
    }
}

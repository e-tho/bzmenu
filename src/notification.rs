use anyhow::{anyhow, Result};
use notify_rust::{Notification, NotificationHandle, Timeout};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::spawn,
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

    pub fn send_scan_notification(&self, on_cancel: impl FnOnce() + Send + 'static) -> Result<u32> {
        let icon_name = self.icons.get_xdg_icon("scan");

        let body = t!("notifications.bt.scan_in_progress");
        let stop_text = t!("notifications.bt.scan_stop_action");

        let mut notification = Notification::new();
        notification
            .summary("BlueZ Menu")
            .body(&body)
            .icon(&icon_name)
            .timeout(Timeout::Never)
            .action("default", &stop_text);

        match notification.show() {
            Ok(handle) => {
                let id = handle.id();

                spawn(move || {
                    handle.wait_for_action(|action| {
                        if action == "default" {
                            on_cancel();
                        }
                    });
                });

                Ok(id)
            }
            Err(err) => Err(anyhow!("Failed to show notification: {}", err)),
        }
    }
}

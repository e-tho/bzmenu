use anyhow::Result;
use bluer::Adapter;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

use crate::bz::device::Device;

pub struct PairingManager {
    adapter: Arc<Adapter>,
    log_sender: UnboundedSender<String>,
}

impl PairingManager {
    pub fn adapter(&self) -> &Arc<Adapter> {
        &self.adapter
    }

    pub fn new(adapter: Arc<Adapter>, log_sender: UnboundedSender<String>) -> Self {
        Self {
            adapter,
            log_sender,
        }
    }

    pub async fn pair_device(&self, device: &Device) -> Result<()> {
        try_send_log!(
            self.log_sender,
            format!("Initiating pairing with {}: {}", device.addr, device.alias)
        );

        device.pair().await?;

        try_send_log!(
            self.log_sender,
            format!("Successfully paired with {}: {}", device.addr, device.alias)
        );

        Ok(())
    }

    pub async fn connect_device(&self, device: &Device) -> Result<()> {
        try_send_log!(
            self.log_sender,
            format!("Connecting to {}: {}", device.addr, device.alias)
        );

        device.connect().await?;

        try_send_log!(
            self.log_sender,
            format!(
                "Successfully connected to {}: {}",
                device.addr, device.alias
            )
        );

        Ok(())
    }

    pub async fn disconnect_device(&self, device: &Device) -> Result<()> {
        try_send_log!(
            self.log_sender,
            format!("Disconnecting from {}: {}", device.addr, device.alias)
        );

        device.disconnect().await?;

        try_send_log!(
            self.log_sender,
            format!(
                "Successfully disconnected from {}: {}",
                device.addr, device.alias
            )
        );

        Ok(())
    }

    pub async fn forget_device(&self, device: &Device) -> Result<()> {
        try_send_log!(
            self.log_sender,
            format!("Removing device {}: {}", device.addr, device.alias)
        );

        device.forget().await?;

        try_send_log!(
            self.log_sender,
            format!(
                "Successfully removed device {}: {}",
                device.addr, device.alias
            )
        );

        Ok(())
    }
}

pub trait PairingConfirmationHandler: Send + Sync {
    fn request_confirmation(
        &self,
        device_address: &str,
        passkey: &str,
        on_confirm: Box<dyn FnOnce() + Send>,
        on_reject: Box<dyn FnOnce() + Send>,
    ) -> Result<()>;
}

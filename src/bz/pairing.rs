use anyhow::Result;
use bluer::Adapter;
use log::{debug, info};
use std::sync::Arc;

use crate::bz::device::Device;

pub struct PairingManager {
    adapter: Arc<Adapter>,
}

impl PairingManager {
    pub fn adapter(&self) -> &Arc<Adapter> {
        &self.adapter
    }

    pub fn new(adapter: Arc<Adapter>) -> Self {
        Self { adapter }
    }

    pub async fn pair_device(&self, device: &Device) -> Result<()> {
        debug!("Initiating pairing with {}: {}", device.addr, device.alias);
        device.pair().await?;
        info!("Successfully paired with {}: {}", device.addr, device.alias);
        Ok(())
    }

    pub async fn connect_device(&self, device: &Device) -> Result<()> {
        debug!("Connecting to {}: {}", device.addr, device.alias);
        device.connect().await?;
        info!(
            "Successfully connected to {}: {}",
            device.addr, device.alias
        );
        Ok(())
    }

    pub async fn disconnect_device(&self, device: &Device) -> Result<()> {
        debug!("Disconnecting from {}: {}", device.addr, device.alias);
        device.disconnect().await?;
        info!(
            "Successfully disconnected from {}: {}",
            device.addr, device.alias
        );
        Ok(())
    }

    pub async fn forget_device(&self, device: &Device) -> Result<()> {
        debug!("Removing device {}: {}", device.addr, device.alias);
        device.forget().await?;
        info!(
            "Successfully removed device {}: {}",
            device.addr, device.alias
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

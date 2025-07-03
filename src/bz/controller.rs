use anyhow::{anyhow, Result};
use bluer::{Adapter, Session};
use std::sync::{atomic::AtomicBool, Arc};
use tokio::sync::mpsc::UnboundedSender;

use crate::bz::device::Device;

#[derive(Debug, Clone)]
pub struct Controller {
    pub adapter: Arc<Adapter>,
    pub name: String,
    pub alias: String,
    pub is_powered: bool,
    pub is_pairable: bool,
    pub is_discoverable: bool,
    pub is_scanning: Arc<AtomicBool>,
    pub paired_devices: Vec<Device>,
    pub new_devices: Vec<Device>,
}

impl Controller {
    pub async fn new(session: Arc<Session>, sender: UnboundedSender<String>) -> Result<Self> {
        let adapter_names = session.adapter_names().await?;
        let adapter_name = adapter_names
            .first()
            .ok_or_else(|| anyhow!("No Bluetooth adapter found"))?;

        let adapter = session.adapter(adapter_name)?;
        let adapter_arc = Arc::new(adapter);

        let name = adapter_arc.name().to_owned();
        let alias = adapter_arc.alias().await?;
        let is_powered = adapter_arc.is_powered().await?;
        let is_pairable = adapter_arc.is_pairable().await?;
        let is_discoverable = adapter_arc.is_discoverable().await?;
        let is_scanning = adapter_arc.is_discovering().await?;

        let (paired_devices, new_devices) = Self::get_devices(&adapter_arc).await?;

        try_send_log!(sender, format!("Bluetooth adapter {name} initialized"));

        Ok(Self {
            adapter: adapter_arc,
            name,
            alias,
            is_powered,
            is_pairable,
            is_discoverable,
            is_scanning: Arc::new(AtomicBool::new(is_scanning)),
            paired_devices,
            new_devices,
        })
    }

    pub async fn refresh(&mut self) -> Result<()> {
        self.is_powered = self.adapter.is_powered().await?;
        self.is_pairable = self.adapter.is_pairable().await?;
        self.is_discoverable = self.adapter.is_discoverable().await?;

        let (paired_devices, new_devices) = Self::get_devices(&self.adapter).await?;
        self.paired_devices = paired_devices;
        self.new_devices = new_devices;

        Ok(())
    }

    pub async fn power_on(&self) -> Result<()> {
        self.adapter.set_powered(true).await?;
        Ok(())
    }

    pub async fn power_off(&self) -> Result<()> {
        self.adapter.set_powered(false).await?;
        Ok(())
    }

    pub async fn set_discoverable(&self, discoverable: bool) -> Result<()> {
        self.adapter.set_discoverable(discoverable).await?;
        Ok(())
    }

    pub async fn set_pairable(&self, pairable: bool) -> Result<()> {
        self.adapter.set_pairable(pairable).await?;
        Ok(())
    }

    async fn get_devices(adapter: &Adapter) -> Result<(Vec<Device>, Vec<Device>)> {
        let mut paired_devices = Vec::new();
        let mut new_devices = Vec::new();

        let device_addresses = adapter.device_addresses().await?;

        for addr in device_addresses {
            if let Ok(device) = Device::new(adapter, &addr).await {
                if device.is_paired {
                    paired_devices.push(device);
                } else {
                    new_devices.push(device);
                }
            }
        }

        paired_devices.sort_by_key(|d| d.addr);
        new_devices.sort_by_key(|d| d.addr);

        Ok((paired_devices, new_devices))
    }
}

use anyhow::Result;
use bluer::Adapter;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::Duration;

pub struct Scanner {
    adapter: Arc<Adapter>,
    is_scanning: Arc<AtomicBool>,
    log_sender: UnboundedSender<String>,
}

impl Scanner {
    pub fn new(
        adapter: Arc<Adapter>,
        is_scanning: Arc<AtomicBool>,
        log_sender: UnboundedSender<String>,
    ) -> Self {
        Self {
            adapter,
            is_scanning,
            log_sender,
        }
    }

    pub async fn start_discovery(&self, timeout_sec: u64) -> Result<()> {
        if self.is_scanning.load(Ordering::Relaxed) {
            try_send_log!(
                self.log_sender,
                "Bluetooth discovery already in progress".to_string()
            );
            return Ok(());
        }

        try_send_log!(
            self.log_sender,
            format!(
                "Starting Bluetooth discovery for {} seconds...",
                timeout_sec
            )
        );

        let discovery_stream = self.adapter.discover_devices().await?;
        self.is_scanning.store(true, Ordering::Relaxed);

        let is_scanning = self.is_scanning.clone();
        let log_sender = self.log_sender.clone();

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(timeout_sec)).await;

            is_scanning.store(false, Ordering::Relaxed);
            try_send_log!(
                log_sender,
                format!("Discovery completed after {} seconds", timeout_sec)
            );

            drop(discovery_stream);
        });

        Ok(())
    }

    pub async fn is_discovery_completed(&self) -> bool {
        !self.is_scanning.load(Ordering::Relaxed)
    }

    pub async fn wait_for_discovery_completion(&self) -> Result<()> {
        if !self.is_scanning.load(Ordering::Relaxed) {
            return Ok(());
        }

        try_send_log!(
            self.log_sender,
            "Waiting for discovery to complete...".to_string()
        );

        while self.is_scanning.load(Ordering::Relaxed) {
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        try_send_log!(self.log_sender, "Discovery process completed".to_string());
        Ok(())
    }
}

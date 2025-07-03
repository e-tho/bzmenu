use anyhow::Result;
use bluer::Adapter;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::{spawn, sync::Mutex, task::JoinHandle, time::Duration};

#[derive(Clone)]
pub struct Scanner {
    adapter: Arc<Adapter>,
    is_scanning: Arc<AtomicBool>,
    log_sender: UnboundedSender<String>,
    scan_task: Arc<Mutex<Option<JoinHandle<()>>>>,
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
            scan_task: Arc::new(Mutex::new(None)),
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
            format!("Starting Bluetooth discovery for {timeout_sec} seconds...")
        );

        let discovery_stream = self.adapter.discover_devices().await?;
        self.is_scanning.store(true, Ordering::Relaxed);

        let is_scanning = self.is_scanning.clone();
        let log_sender = self.log_sender.clone();

        let task = spawn(async move {
            tokio::time::sleep(Duration::from_secs(timeout_sec)).await;

            is_scanning.store(false, Ordering::Relaxed);
            try_send_log!(
                log_sender,
                format!("Discovery completed after {timeout_sec} seconds")
            );

            drop(discovery_stream);
        });

        let mut scan_task_guard = self.scan_task.lock().await;
        *scan_task_guard = Some(task);

        Ok(())
    }

    pub async fn stop_discovery(&self) -> Result<()> {
        if !self.is_scanning.load(Ordering::Relaxed) {
            try_send_log!(
                self.log_sender,
                "No Bluetooth discovery in progress to stop".to_string()
            );
            return Ok(());
        }

        try_send_log!(
            self.log_sender,
            "Stopping Bluetooth discovery...".to_string()
        );

        self.is_scanning.store(false, Ordering::Relaxed);

        let mut scan_task_guard = self.scan_task.lock().await;
        if let Some(task) = scan_task_guard.take() {
            task.abort();
            try_send_log!(self.log_sender, "Discovery task aborted".to_string());
        }

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

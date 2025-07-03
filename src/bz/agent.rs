use crate::bz::pairing::PairingConfirmationHandler;
use anyhow::Result;
use bluer::agent::{Agent, AgentHandle, ReqError};
use bluer::Session;
use futures_util::FutureExt;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedSender},
    Mutex,
};
use tokio::time::timeout;

pub struct AgentManager {
    session: Arc<Session>,
    confirmation_required: Arc<AtomicBool>,
    passkey_sender: UnboundedSender<bool>,
    _agent_handle: AgentHandle,
}

impl AgentManager {
    pub async fn new(
        session: Arc<Session>,
        log_sender: UnboundedSender<String>,
        pairing_handler: Arc<dyn PairingConfirmationHandler>,
    ) -> Result<Self> {
        let (passkey_sender, passkey_receiver) = unbounded_channel::<bool>();
        let _passkey_receiver = Arc::new(Mutex::new(passkey_receiver));
        let confirmation_required = Arc::new(AtomicBool::new(false));

        let agent = {
            let confirmation_required_clone = confirmation_required.clone();
            let passkey_sender_clone = passkey_sender.clone();
            let log_sender_clone = log_sender.clone();
            let pairing_handler = pairing_handler.clone();

            Agent {
                request_default: true,
                request_confirmation: Some(Box::new(move |req| {
                    let confirmation_required = confirmation_required_clone.clone();
                    let _passkey_sender = passkey_sender_clone.clone();
                    let log_sender = log_sender_clone.clone();
                    let pairing_handler = pairing_handler.clone();

                    async move {
                        confirmation_required.store(true, Ordering::Relaxed);

                        let device_address = req.device.to_string();
                        let passkey_str = format!("{:06}", req.passkey);

                        try_send_log!(
                            log_sender,
                            format!("Confirm passkey {passkey_str} for device {device_address}? (yes/no)")
                        );

                        let (tx, mut rx) = tokio::sync::mpsc::channel::<bool>(1);

                        let device_address_clone = device_address.clone();

                        let _ = pairing_handler.request_confirmation(
                            &device_address,
                            &passkey_str,
                            Box::new({
                                let tx = tx.clone();
                                let log_sender = log_sender.clone();
                                let device_addr = device_address_clone.clone();
                                move || {
                                    try_send_log!(
                                        log_sender,
                                        format!("User confirmed pairing for device {device_addr}")
                                    );
                                    let _ = tx.blocking_send(true);
                                }
                            }),
                            Box::new({
                                let tx = tx.clone();
                                let log_sender = log_sender.clone();
                                let device_addr = device_address_clone.clone();
                                move || {
                                    try_send_log!(
                                        log_sender,
                                        format!("User rejected pairing for device {device_addr}")
                                    );
                                    let _ = tx.blocking_send(false);
                                }
                            }),
                        );

                        let result = match timeout(Duration::from_secs(30), rx.recv()).await {
                            Ok(Some(true)) => Ok(()),
                            _ => Err(ReqError::Rejected),
                        };

                        confirmation_required.store(false, Ordering::Relaxed);
                        result
                    }
                    .boxed()
                })),
                ..Default::default()
            }
        };

        let agent_handle = session.register_agent(agent).await?;

        try_send_log!(log_sender, "Bluetooth agent registered".to_string());

        Ok(Self {
            session,
            confirmation_required,
            passkey_sender,
            _agent_handle: agent_handle,
        })
    }

    pub fn session(&self) -> Arc<Session> {
        self.session.clone()
    }

    pub fn confirm_passkey(&self) -> Result<()> {
        self.passkey_sender.send(true)?;
        self.confirmation_required.store(false, Ordering::Relaxed);
        Ok(())
    }

    pub fn reject_passkey(&self) -> Result<()> {
        self.passkey_sender.send(false)?;
        self.confirmation_required.store(false, Ordering::Relaxed);
        Ok(())
    }
}

use anyhow::Result;
use bluer::agent::{Agent, AgentHandle, ReqError};
use bluer::Session;
use futures::FutureExt;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedSender},
    Mutex,
};

pub struct AgentManager {
    session: Arc<Session>,
    confirmation_required: Arc<AtomicBool>,
    passkey_sender: UnboundedSender<bool>,
    _agent_handle: AgentHandle,
}

impl AgentManager {
    pub async fn new(session: Arc<Session>, log_sender: UnboundedSender<String>) -> Result<Self> {
        let (passkey_sender, passkey_receiver) = unbounded_channel::<bool>();
        let passkey_receiver = Arc::new(Mutex::new(passkey_receiver));
        let confirmation_required = Arc::new(AtomicBool::new(false));

        let agent = {
            let confirmation_required_clone = confirmation_required.clone();
            let passkey_receiver_clone = passkey_receiver.clone();
            let log_sender_clone = log_sender.clone();

            Agent {
                request_default: true,
                request_confirmation: Some(Box::new(move |req| {
                    let confirmation_required = confirmation_required_clone.clone();
                    let passkey_receiver = passkey_receiver_clone.clone();
                    let log_sender = log_sender_clone.clone();

                    async move {
                        confirmation_required.store(true, Ordering::Relaxed);

                        try_send_log!(
                            log_sender,
                            format!(
                                "Confirm passkey {:06} for device {}? (yes/no)",
                                req.passkey,
                                req.device.to_string()
                            )
                        );

                        let mut rx = passkey_receiver.lock().await;
                        let result = match rx.recv().await {
                            Some(true) => Ok(()),
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

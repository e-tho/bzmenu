use crate::{
    bz::{
        agent::AgentManager,
        controller::Controller,
        pairing::{PairingConfirmationHandler, PairingManager},
        scanner::Scanner,
    },
    icons::Icons,
    menu::{AdapterMenuOptions, DeviceMenuOptions, MainMenuOptions, Menu, SettingsMenuOptions},
    notification::NotificationManager,
};
use anyhow::Result;
use bluer::Session;
use log::{debug, error, info};
use rust_i18n::t;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::runtime::Builder;

pub struct App {
    pub running: bool,
    pub reset_mode: bool,
    session: Arc<Session>,
    controller: Controller,
    agent_manager: AgentManager,
    scanner: Scanner,
    pairing_manager: PairingManager,
    notification_manager: Arc<NotificationManager>,
    scan_duration: u64,
}

impl App {
    pub fn get_session(&self) -> Arc<Session> {
        self.session.clone()
    }

    pub fn get_agent_manager(&self) -> &AgentManager {
        &self.agent_manager
    }

    pub async fn new(icons: Arc<Icons>, scan_duration: u64) -> Result<Self> {
        let session = Arc::new(Session::new().await?);
        let notification_manager = Arc::new(NotificationManager::new(icons.clone()));

        let agent_manager = AgentManager::new(
            session.clone(),
            notification_manager.clone() as Arc<dyn PairingConfirmationHandler>,
        )
        .await?;

        let controller = Controller::new(session.clone()).await?;

        let scanner = Scanner::new(controller.adapter.clone(), controller.is_scanning.clone());

        let pairing_manager = PairingManager::new(controller.adapter.clone());

        if !controller.is_powered {
            info!("{}", t!("notifications.bt.adapter_powered_off"));
        }

        Ok(Self {
            running: true,
            reset_mode: false,
            session,
            controller,
            agent_manager,
            scanner,
            pairing_manager,
            notification_manager,
            scan_duration,
        })
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub async fn run(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
    ) -> Result<Option<String>> {
        if !self.controller.is_powered {
            self.handle_adapter_options(menu, menu_command, icon_type, spaces)
                .await?;
            if self.running {
                self.controller.refresh().await?;
            } else {
                return Ok(None);
            }
        }

        while self.running {
            self.controller.refresh().await?;

            match menu
                .show_main_menu(menu_command, &self.controller, icon_type, spaces)
                .await?
            {
                Some(main_menu_option) => {
                    self.handle_main_options(
                        menu,
                        menu_command,
                        icon_type,
                        spaces,
                        main_menu_option,
                    )
                    .await?;
                }
                None => {
                    debug!("{}", t!("notifications.bt.main_menu_exited"));
                    self.running = false;
                }
            }
        }

        Ok(None)
    }

    async fn handle_main_options(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
        main_menu_option: MainMenuOptions,
    ) -> Result<Option<String>> {
        match main_menu_option {
            MainMenuOptions::Scan => {
                self.perform_device_scan().await?;
            }
            MainMenuOptions::Settings => {
                self.handle_settings_menu(menu, menu_command, icon_type, spaces)
                    .await?;
            }
            MainMenuOptions::Device(output) => {
                if let Some(device) = self
                    .handle_device_selection(menu, menu_command, &output, icon_type, spaces)
                    .await?
                {
                    return Ok(Some(device.addr.to_string()));
                }
            }
        }
        Ok(None)
    }

    async fn handle_settings_menu(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
    ) -> Result<()> {
        let mut stay_in_settings = true;

        while stay_in_settings {
            self.controller.refresh().await?;

            if let Some(option) = menu
                .show_settings_menu(menu_command, &self.controller, icon_type, spaces)
                .await?
            {
                if matches!(option, SettingsMenuOptions::DisableAdapter) {
                    self.handle_settings_options(menu, menu_command, icon_type, spaces, option)
                        .await?;
                    stay_in_settings = false;
                } else {
                    self.handle_settings_options(menu, menu_command, icon_type, spaces, option)
                        .await?;
                }
            } else {
                stay_in_settings = false;
                debug!("Exited settings menu");
            }
        }

        Ok(())
    }

    async fn handle_settings_options(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
        option: SettingsMenuOptions,
    ) -> Result<()> {
        match option {
            SettingsMenuOptions::ToggleDiscoverable => {
                let new_state = !self.controller.is_discoverable;
                self.controller.set_discoverable(new_state).await?;

                let msg = if new_state {
                    t!("notifications.bt.discoverable_enabled")
                } else {
                    t!("notifications.bt.discoverable_disabled")
                };

                info!("{msg}");
                try_send_notification!(
                    self.notification_manager,
                    None,
                    Some(msg.to_string()),
                    Some("bluetooth"),
                    None,
                    None
                );
            }
            SettingsMenuOptions::TogglePairable => {
                let new_state = !self.controller.is_pairable;
                self.controller.set_pairable(new_state).await?;

                let msg = if new_state {
                    t!("notifications.bt.pairable_enabled")
                } else {
                    t!("notifications.bt.pairable_disabled")
                };

                info!("{msg}");
                try_send_notification!(
                    self.notification_manager,
                    None,
                    Some(msg.to_string()),
                    Some("bluetooth"),
                    None,
                    None
                );
            }
            SettingsMenuOptions::DisableAdapter => {
                self.perform_adapter_disable(menu, menu_command, icon_type, spaces)
                    .await?;
            }
        }
        Ok(())
    }

    async fn handle_adapter_options(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
    ) -> Result<()> {
        if let Some(option) = menu.prompt_enable_adapter(menu_command, icon_type, spaces) {
            match option {
                AdapterMenuOptions::PowerOnDevice => {
                    self.controller.power_on().await?;
                    self.controller.refresh().await?;

                    info!("{}", t!("notifications.bt.adapter_enabled"));
                    try_send_notification!(
                        self.notification_manager,
                        None,
                        Some(t!("notifications.bt.adapter_enabled").to_string()),
                        Some("bluetooth"),
                        None,
                        None
                    );
                }
            }
        } else {
            info!("{}", t!("notifications.bt.adapter_menu_exited"));
            self.running = false;
        }

        Ok(())
    }

    async fn handle_device_menu(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        device: &crate::bz::device::Device,
        icon_type: &str,
        spaces: usize,
    ) -> Result<()> {
        let mut device_clone = device.clone();
        let mut stay_in_device_menu = true;

        while stay_in_device_menu {
            if let Ok(refreshed_device) =
                crate::bz::device::Device::new(&self.controller.adapter, &device_clone.addr).await
            {
                device_clone = refreshed_device;
            } else {
                error!("Device {} is no longer available", device_clone.alias);
                break;
            }

            let available_options = if device_clone.is_paired {
                menu.get_paired_device_options(&device_clone)
            } else {
                vec![DeviceMenuOptions::Connect]
            };

            match menu
                .show_device_options(
                    menu_command,
                    icon_type,
                    spaces,
                    available_options,
                    &device_clone.alias,
                )
                .await?
            {
                Some(option) => {
                    match option {
                        DeviceMenuOptions::Connect => {
                            if !device_clone.is_connected {
                                self.perform_device_connection(&device_clone).await?;
                            }
                        }
                        DeviceMenuOptions::Disconnect => {
                            if device_clone.is_connected {
                                self.perform_device_disconnection(&device_clone).await?;
                            }
                        }
                        DeviceMenuOptions::Trust => {
                            if !device_clone.is_trusted {
                                self.perform_trust_device(&device_clone, true).await?;
                            }
                        }
                        DeviceMenuOptions::RevokeTrust => {
                            if device_clone.is_trusted {
                                self.perform_trust_device(&device_clone, false).await?;
                            }
                        }
                        DeviceMenuOptions::Forget => {
                            self.perform_forget_device(&device_clone).await?;
                            stay_in_device_menu = false;
                        }
                    }

                    self.controller.refresh().await?;
                }
                None => {
                    stay_in_device_menu = false;
                    debug!("Exited device menu for {}", device_clone.alias);
                }
            }
        }

        Ok(())
    }

    async fn handle_device_selection(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        output: &str,
        icon_type: &str,
        spaces: usize,
    ) -> Result<Option<crate::bz::device::Device>> {
        let cleaned_output = menu.clean_menu_output(output, icon_type);

        let paired_device_clone = self
            .controller
            .paired_devices
            .iter()
            .find(|device| {
                let formatted = menu.format_device_display(device, icon_type, spaces);
                menu.clean_menu_output(&formatted, icon_type) == cleaned_output
            })
            .cloned();

        let new_device_clone = self
            .controller
            .new_devices
            .iter()
            .find(|device| {
                let formatted = menu.format_device_display(device, icon_type, spaces);
                menu.clean_menu_output(&formatted, icon_type) == cleaned_output
            })
            .cloned();

        if let Some(device) = paired_device_clone.or(new_device_clone) {
            self.handle_device_menu(menu, menu_command, &device, icon_type, spaces)
                .await?;
            return Ok(Some(device));
        }

        Ok(None)
    }

    async fn perform_device_scan(&mut self) -> Result<()> {
        if self.controller.is_scanning.load(Ordering::Relaxed) {
            let msg = t!("notifications.bt.scan_already_in_progress");
            info!("{msg}");
            try_send_notification!(
                self.notification_manager,
                None,
                Some(msg.to_string()),
                Some("bluetooth"),
                None,
                None
            );
            return Ok(());
        }

        let scan_duration = self.scan_duration;

        self.scanner.start_discovery(scan_duration).await?;

        let scanner_clone = self.scanner.clone();

        let progress_msg = t!("notifications.bt.scan_in_progress");
        let completed_msg = t!("notifications.bt.scan_completed");

        let id = self.notification_manager.send_progress_notification(
            scan_duration,
            move || {
                debug!("User cancelled Bluetooth scan");

                let rt = Builder::new_current_thread().enable_all().build().unwrap();

                rt.block_on(async {
                    let _ = scanner_clone.stop_discovery().await;
                });
            },
            progress_msg.to_string(),
            Some("scan_in_progress"),
        )?;

        self.scanner.wait_for_discovery_completion().await?;

        self.controller.refresh().await?;

        let _ = self.notification_manager.send_notification(
            None,
            Some(completed_msg.to_string()),
            Some("ok"),
            None,
            Some(id),
        );

        Ok(())
    }

    async fn perform_device_connection(&self, device: &crate::bz::device::Device) -> Result<()> {
        debug!("Connecting to device: {}", device.alias);

        let result = if !device.is_paired {
            self.pairing_manager.pair_device(device).await
        } else {
            Ok(())
        };

        if let Err(err) = result {
            let msg = t!(
                "notifications.bt.pairing_failed",
                device_name = device.alias,
                error = err.to_string()
            );

            info!("{msg}");
            try_send_notification!(
                self.notification_manager,
                None,
                Some(msg.to_string()),
                Some("bluetooth"),
                None,
                None
            );
            return Ok(());
        }

        let connection_result = self.pairing_manager.connect_device(device).await;

        match connection_result {
            Ok(_) => {
                let msg = t!(
                    "notifications.bt.device_connected",
                    device_name = device.alias
                );

                info!("{msg}");
                try_send_notification!(
                    self.notification_manager,
                    None,
                    Some(msg.to_string()),
                    Some("bluetooth"),
                    None,
                    None
                );
                Ok(())
            }
            Err(err) => {
                let msg = if err.to_string().contains("Page Timeout") {
                    t!(
                        "notifications.bt.device_out_of_range",
                        device_name = device.alias
                    )
                } else {
                    t!(
                        "notifications.bt.connection_failed",
                        device_name = device.alias,
                        error = err.to_string()
                    )
                };

                info!("{msg}");

                try_send_notification!(
                    self.notification_manager,
                    None,
                    Some(msg.to_string()),
                    Some("bluetooth"),
                    None,
                    None
                );

                Ok(())
            }
        }
    }

    async fn perform_device_disconnection(&self, device: &crate::bz::device::Device) -> Result<()> {
        debug!("Disconnecting from device: {}", device.alias);

        self.pairing_manager.disconnect_device(device).await?;

        let msg = t!(
            "notifications.bt.device_disconnected",
            device_name = device.alias
        );

        info!("{msg}");
        try_send_notification!(
            self.notification_manager,
            None,
            Some(msg.to_string()),
            Some("bluetooth"),
            None,
            None
        );

        Ok(())
    }

    async fn perform_trust_device(
        &self,
        device: &crate::bz::device::Device,
        trust: bool,
    ) -> Result<()> {
        info!(
            "{} trust for device: {}",
            if trust { "Enabling" } else { "Revoking" },
            device.alias
        );

        device.set_trusted(trust).await?;

        let msg = if trust {
            t!(
                "notifications.bt.device_trusted",
                device_name = device.alias
            )
        } else {
            t!(
                "notifications.bt.device_untrusted",
                device_name = device.alias
            )
        };

        info!("{msg}");
        try_send_notification!(
            self.notification_manager,
            None,
            Some(msg.to_string()),
            Some("bluetooth"),
            None,
            None
        );

        Ok(())
    }

    async fn perform_forget_device(&self, device: &crate::bz::device::Device) -> Result<()> {
        info!("Forgetting device: {}", device.alias);

        self.pairing_manager.forget_device(device).await?;

        let msg = t!(
            "notifications.bt.device_forgotten",
            device_name = device.alias
        );

        info!("{msg}");
        try_send_notification!(
            self.notification_manager,
            None,
            Some(msg.to_string()),
            Some("bluetooth"),
            None,
            None
        );

        Ok(())
    }

    async fn perform_adapter_disable(
        &mut self,
        menu: &Menu,
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
    ) -> Result<()> {
        self.controller.power_off().await?;

        let msg = t!("notifications.bt.adapter_disabled").to_string();
        info!("{msg}");
        try_send_notification!(
            self.notification_manager,
            None,
            Some(msg),
            Some("bluetooth"),
            None,
            None
        );

        self.handle_adapter_options(menu, menu_command, icon_type, spaces)
            .await?;

        Ok(())
    }
}

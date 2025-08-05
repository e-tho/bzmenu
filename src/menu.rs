use crate::bz::{controller::Controller, device::Device};
use crate::icons::Icons;
use crate::launcher::{Launcher, LauncherType};
use anyhow::Result;
use rust_i18n::t;
use std::borrow::Cow;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum MainMenuOptions {
    Scan,
    Settings,
    Device(String),
}

impl MainMenuOptions {
    pub fn from_string(option: &str) -> Option<Self> {
        match option {
            s if s == t!("menus.main.options.scan.name") => Some(MainMenuOptions::Scan),
            s if s == t!("menus.main.options.settings.name") => Some(MainMenuOptions::Settings),
            other => Some(MainMenuOptions::Device(other.to_string())),
        }
    }

    pub fn to_str(&self) -> Cow<'static, str> {
        match self {
            MainMenuOptions::Scan => t!("menus.main.options.scan.name"),
            MainMenuOptions::Settings => t!("menus.main.options.settings.name"),
            MainMenuOptions::Device(_) => t!("menus.main.options.device.name"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceMenuOptions {
    Connect,
    Disconnect,
    Trust,
    RevokeTrust,
    Forget,
}

impl DeviceMenuOptions {
    pub fn from_string(option: &str) -> Option<Self> {
        match option {
            s if s == t!("menus.device.options.connect.name") => Some(DeviceMenuOptions::Connect),
            s if s == t!("menus.device.options.disconnect.name") => {
                Some(DeviceMenuOptions::Disconnect)
            }
            s if s == t!("menus.device.options.trust.name") => Some(DeviceMenuOptions::Trust),
            s if s == t!("menus.device.options.revoke_trust.name") => {
                Some(DeviceMenuOptions::RevokeTrust)
            }
            s if s == t!("menus.device.options.forget.name") => Some(DeviceMenuOptions::Forget),
            _ => None,
        }
    }

    pub fn to_str(&self) -> Cow<'static, str> {
        match self {
            DeviceMenuOptions::Connect => t!("menus.device.options.connect.name"),
            DeviceMenuOptions::Disconnect => t!("menus.device.options.disconnect.name"),
            DeviceMenuOptions::Trust => t!("menus.device.options.trust.name"),
            DeviceMenuOptions::RevokeTrust => t!("menus.device.options.revoke_trust.name"),
            DeviceMenuOptions::Forget => t!("menus.device.options.forget.name"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SettingsMenuOptions {
    ToggleDiscoverable,
    TogglePairable,
    DisableAdapter,
}

impl SettingsMenuOptions {
    pub fn from_string(option: &str) -> Option<Self> {
        match option {
            s if s == t!("menus.settings.options.toggle_discoverable.name") => {
                Some(SettingsMenuOptions::ToggleDiscoverable)
            }
            s if s == t!("menus.settings.options.toggle_pairable.name") => {
                Some(SettingsMenuOptions::TogglePairable)
            }
            s if s == t!("menus.settings.options.disable_adapter.name") => {
                Some(SettingsMenuOptions::DisableAdapter)
            }
            _ => None,
        }
    }

    pub fn to_str(&self) -> Cow<'static, str> {
        match self {
            SettingsMenuOptions::ToggleDiscoverable => {
                t!("menus.settings.options.toggle_discoverable.name")
            }
            SettingsMenuOptions::TogglePairable => {
                t!("menus.settings.options.toggle_pairable.name")
            }
            SettingsMenuOptions::DisableAdapter => {
                t!("menus.settings.options.disable_adapter.name")
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AdapterMenuOptions {
    PowerOnDevice,
}

impl AdapterMenuOptions {
    pub fn from_string(option: &str) -> Option<Self> {
        if option == t!("menus.adapter.options.power_on_device.name") {
            Some(AdapterMenuOptions::PowerOnDevice)
        } else {
            None
        }
    }

    pub fn to_str(&self) -> Cow<'static, str> {
        match self {
            AdapterMenuOptions::PowerOnDevice => t!("menus.adapter.options.power_on_device.name"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BluetoothMenuOptions {
    EnableDiscoverable,
    DisableDiscoverable,
    EnablePairable,
    DisablePairable,
}

impl BluetoothMenuOptions {
    pub fn from_string(option: &str) -> Option<Self> {
        match option {
            s if s == t!("menus.bluetooth.options.enable_discoverable.name") => {
                Some(BluetoothMenuOptions::EnableDiscoverable)
            }
            s if s == t!("menus.bluetooth.options.disable_discoverable.name") => {
                Some(BluetoothMenuOptions::DisableDiscoverable)
            }
            s if s == t!("menus.bluetooth.options.enable_pairable.name") => {
                Some(BluetoothMenuOptions::EnablePairable)
            }
            s if s == t!("menus.bluetooth.options.disable_pairable.name") => {
                Some(BluetoothMenuOptions::DisablePairable)
            }
            _ => None,
        }
    }

    pub fn to_str(&self) -> Cow<'static, str> {
        match self {
            BluetoothMenuOptions::EnableDiscoverable => {
                t!("menus.bluetooth.options.enable_discoverable.name")
            }
            BluetoothMenuOptions::DisableDiscoverable => {
                t!("menus.bluetooth.options.disable_discoverable.name")
            }
            BluetoothMenuOptions::EnablePairable => {
                t!("menus.bluetooth.options.enable_pairable.name")
            }
            BluetoothMenuOptions::DisablePairable => {
                t!("menus.bluetooth.options.disable_pairable.name")
            }
        }
    }
}

#[derive(Clone)]
pub struct Menu {
    pub launcher_type: LauncherType,
    pub icons: Arc<Icons>,
}

impl Menu {
    pub fn new(launcher_type: LauncherType, icons: Arc<Icons>) -> Self {
        Self {
            launcher_type,
            icons,
        }
    }

    pub fn run_launcher(
        &self,
        launcher_command: &Option<String>,
        input: Option<&str>,
        icon_type: &str,
        hint: Option<&str>,
    ) -> Result<Option<String>> {
        let cmd = Launcher::create_command(&self.launcher_type, launcher_command, icon_type, hint)?;

        Launcher::run(cmd, input)
    }

    pub fn clean_menu_output(&self, output: &str, icon_type: &str) -> String {
        let output_trimmed = output.trim();

        if icon_type == "font" {
            output_trimmed
                .chars()
                .skip_while(|c| !c.is_ascii_alphanumeric())
                .collect::<String>()
                .trim()
                .to_string()
        } else if icon_type == "xdg" {
            output_trimmed
                .split('\0')
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        } else {
            output_trimmed.to_string()
        }
    }

    pub fn get_icon_text<T>(&self, items: Vec<(&str, T)>, icon_type: &str, spaces: usize) -> String
    where
        T: AsRef<str>,
    {
        items
            .into_iter()
            .map(|(icon_key, text)| {
                let icon = self.icons.get_icon(icon_key, icon_type);
                let text = text.as_ref();
                match icon_type {
                    "font" => format!("{}{}{}", icon, " ".repeat(spaces), text),
                    "xdg" => format!("{text}\0icon\x1f{icon}"),
                    _ => text.to_string(),
                }
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn format_device_display(&self, device: &Device, icon_type: &str, spaces: usize) -> String {
        let mut display_name = device.alias.to_string();

        let mut status_indicators = String::new();

        if let Some(battery) = device.battery_percentage {
            if icon_type == "font" {
                if let Some(battery_icon) = self.icons.get_battery_icon(battery, icon_type) {
                    status_indicators.push_str(&format!(" [{battery_icon}]"));
                }
            } else if icon_type == "xdg" {
                status_indicators.push_str(&format!(" [{battery}%]"));
            }
        }

        if device.is_connected {
            status_indicators
                .push_str(&format!(" {}", self.icons.get_icon("connected", "generic")));
        }

        if device.is_trusted {
            status_indicators.push_str(&format!(" {}", self.icons.get_icon("trusted", "generic")));
        }

        display_name.push_str(&status_indicators);

        let icon = self.icons.get_device_icon(&device.device_type, icon_type);

        self.icons
            .format_display_with_icon(&display_name, &icon, icon_type, spaces)
    }

    pub async fn show_main_menu(
        &self,
        launcher_command: &Option<String>,
        controller: &Controller,
        icon_type: &str,
        spaces: usize,
    ) -> Result<Option<MainMenuOptions>> {
        let scan_text = MainMenuOptions::Scan.to_str();
        let settings_text = MainMenuOptions::Settings.to_str();

        let options_start = vec![("scan", scan_text.as_ref())];
        let mut input = self.get_icon_text(options_start, icon_type, spaces);

        for device in &controller.paired_devices {
            let device_display = self.format_device_display(device, icon_type, spaces);
            input.push_str(&format!("\n{device_display}"));
        }

        for device in &controller.new_devices {
            let device_display = self.format_device_display(device, icon_type, spaces);
            input.push_str(&format!("\n{device_display}"));
        }

        let options_end = vec![("settings", settings_text.as_ref())];
        let settings_input = self.get_icon_text(options_end, icon_type, spaces);
        input.push_str(&format!("\n{settings_input}"));

        let menu_output = self.run_launcher(launcher_command, Some(&input), icon_type, None)?;

        if let Some(output) = menu_output {
            let cleaned_output = self.clean_menu_output(&output, icon_type);

            if cleaned_output == scan_text.as_ref() {
                return Ok(Some(MainMenuOptions::Scan));
            } else if cleaned_output == settings_text.as_ref() {
                return Ok(Some(MainMenuOptions::Settings));
            } else {
                return Ok(Some(MainMenuOptions::Device(cleaned_output)));
            }
        }

        Ok(None)
    }

    pub async fn show_device_options(
        &self,
        launcher_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
        available_options: Vec<DeviceMenuOptions>,
        device_name: &str,
    ) -> Result<Option<DeviceMenuOptions>> {
        let mut input = String::new();

        for option in &available_options {
            let icon_key = match option {
                DeviceMenuOptions::Connect => "connect",
                DeviceMenuOptions::Disconnect => "disconnect",
                DeviceMenuOptions::Trust => "trust",
                DeviceMenuOptions::RevokeTrust => "revoke_trust",
                DeviceMenuOptions::Forget => "forget",
            };

            let option_text =
                self.get_icon_text(vec![(icon_key, option.to_str())], icon_type, spaces);
            input.push_str(&format!("{option_text}\n"));
        }

        let hint = t!("menus.device.hint", device_name = device_name);

        let menu_output =
            self.run_launcher(launcher_command, Some(&input), icon_type, Some(&hint))?;

        if let Some(output) = menu_output {
            let cleaned_output = self.clean_menu_output(&output, icon_type);
            return Ok(DeviceMenuOptions::from_string(&cleaned_output));
        }

        Ok(None)
    }

    pub fn get_paired_device_options(&self, device: &Device) -> Vec<DeviceMenuOptions> {
        let mut options = Vec::new();

        if device.is_connected {
            options.push(DeviceMenuOptions::Disconnect);
        } else {
            options.push(DeviceMenuOptions::Connect);
        }

        if device.is_trusted {
            options.push(DeviceMenuOptions::RevokeTrust);
        } else {
            options.push(DeviceMenuOptions::Trust);
        }

        options.push(DeviceMenuOptions::Forget);

        options
    }

    pub async fn show_settings_menu(
        &self,
        launcher_command: &Option<String>,
        controller: &Controller,
        icon_type: &str,
        spaces: usize,
    ) -> Result<Option<SettingsMenuOptions>> {
        let (discoverable_text, discoverable_icon) = if controller.is_discoverable {
            (
                t!("menus.settings.options.disable_discoverable.name"),
                "disable_discoverable",
            )
        } else {
            (
                t!("menus.settings.options.enable_discoverable.name"),
                "enable_discoverable",
            )
        };

        let (pairable_text, pairable_icon) = if controller.is_pairable {
            (
                t!("menus.settings.options.disable_pairable.name"),
                "disable_pairable",
            )
        } else {
            (
                t!("menus.settings.options.enable_pairable.name"),
                "enable_pairable",
            )
        };

        let disable_adapter_text = t!("menus.settings.options.disable_adapter.name");

        let options = vec![
            (discoverable_icon, discoverable_text.as_ref()),
            (pairable_icon, pairable_text.as_ref()),
            ("disable_adapter", disable_adapter_text.as_ref()),
        ];

        let input = self.get_icon_text(options, icon_type, spaces);

        let menu_output = self.run_launcher(launcher_command, Some(&input), icon_type, None)?;

        if let Some(output) = menu_output {
            let cleaned_output = self.clean_menu_output(&output, icon_type);

            if cleaned_output == discoverable_text.as_ref() {
                return Ok(Some(SettingsMenuOptions::ToggleDiscoverable));
            } else if cleaned_output == pairable_text.as_ref() {
                return Ok(Some(SettingsMenuOptions::TogglePairable));
            } else if cleaned_output == disable_adapter_text.as_ref() {
                return Ok(Some(SettingsMenuOptions::DisableAdapter));
            }
        }

        Ok(None)
    }

    pub fn prompt_enable_adapter(
        &self,
        launcher_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
    ) -> Option<AdapterMenuOptions> {
        let options = vec![(
            "power_on_device",
            AdapterMenuOptions::PowerOnDevice.to_str(),
        )];

        let input = self.get_icon_text(options, icon_type, spaces);

        if let Ok(Some(output)) = self.run_launcher(launcher_command, Some(&input), icon_type, None)
        {
            let cleaned_output = self.clean_menu_output(&output, icon_type);
            return AdapterMenuOptions::from_string(&cleaned_output);
        }

        None
    }

    pub fn prompt_passkey_confirmation(
        &self,
        launcher_command: &Option<String>,
        device_name: &str,
        passkey: &str,
        icon_type: &str,
    ) -> Result<bool> {
        let hint = t!(
            "menus.bluetooth.confirm_passkey",
            device_name = device_name,
            passkey = passkey
        );

        let options = vec![
            ("confirm", t!("menus.bluetooth.confirm")),
            ("cancel", t!("menus.bluetooth.cancel")),
        ];

        let input = self.get_icon_text(options, icon_type, 1);

        let menu_output =
            self.run_launcher(launcher_command, Some(&input), icon_type, Some(&hint))?;

        if let Some(output) = menu_output {
            let cleaned_output = self.clean_menu_output(&output, icon_type);
            return Ok(cleaned_output == t!("menus.bluetooth.confirm"));
        }

        Ok(false)
    }
}

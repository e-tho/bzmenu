use crate::bz::{controller::Controller, device::Device};
use crate::icons::Icons;
use anyhow::{anyhow, Result};
use clap::ArgEnum;
use nix::sys::signal::{killpg, Signal};
use nix::unistd::Pid;
use process_wrap::std::{ProcessGroup, StdCommandWrap};
use rust_i18n::t;
use shlex::Shlex;
use signal_hook::iterator::Signals;
use std::borrow::Cow;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;

#[derive(Debug, Clone, ArgEnum)]
pub enum MenuType {
    Fuzzel,
    Rofi,
    Dmenu,
    Walker,
    Custom,
}

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
            s if s == t!("menus.device.options.trust.name") => {
                Some(DeviceMenuOptions::Trust)
            }
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
    pub menu_type: MenuType,
    pub icons: Arc<Icons>,
}

impl Menu {
    pub fn new(menu_type: MenuType, icons: Arc<Icons>) -> Self {
        Self { menu_type, icons }
    }

    pub fn run_menu_command(
        &self,
        menu_command: &Option<String>,
        input: Option<&str>,
        icon_type: &str,
        prompt: Option<&str>,
    ) -> Result<Option<String>> {
        let (prompt_text, placeholder_text) = if let Some(p) = prompt {
            (format!("{}: ", p), p.to_string())
        } else {
            (String::new(), String::new())
        };

        let mut command = match self.menu_type {
            MenuType::Fuzzel => {
                let mut cmd = Command::new("fuzzel");
                cmd.arg("-d");
                if icon_type == "font" {
                    cmd.arg("-I");
                }
                if !placeholder_text.is_empty() {
                    cmd.arg("--placeholder").arg(&placeholder_text);
                }
                cmd
            }
            MenuType::Rofi => {
                let mut cmd = Command::new("rofi");
                cmd.arg("-m").arg("-1").arg("-dmenu");
                if icon_type == "xdg" {
                    cmd.arg("-show-icons");
                }
                if !placeholder_text.is_empty() {
                    cmd.arg("-theme-str").arg(format!(
                        "entry {{ placeholder: \"{}\"; }}",
                        placeholder_text
                    ));
                }
                cmd
            }
            MenuType::Dmenu => {
                let mut cmd = Command::new("dmenu");
                if !prompt_text.is_empty() {
                    cmd.arg("-p").arg(&prompt_text);
                }
                cmd
            }
            MenuType::Walker => {
                let mut cmd = Command::new("walker");
                cmd.arg("-d").arg("-k");
                if !placeholder_text.is_empty() {
                    cmd.arg("-p").arg(&placeholder_text);
                }
                cmd
            }
            MenuType::Custom => {
                if let Some(cmd_str) = menu_command {
                    let mut cmd_processed = cmd_str.clone();
                    cmd_processed = cmd_processed.replace("{prompt}", &prompt_text);
                    cmd_processed = cmd_processed.replace("{placeholder}", &placeholder_text);

                    let parts: Vec<String> = Shlex::new(&cmd_processed).collect();
                    let (cmd_program, args) = parts
                        .split_first()
                        .ok_or_else(|| anyhow!("Failed to parse custom menu command"))?;

                    let mut cmd = Command::new(cmd_program);
                    cmd.args(args);
                    cmd
                } else {
                    return Err(anyhow!("No custom menu command provided"));
                }
            }
        };

        command.stdin(Stdio::piped()).stdout(Stdio::piped());

        let mut command_wrap = StdCommandWrap::from(command);
        command_wrap.wrap(ProcessGroup::leader());

        let mut child = command_wrap.spawn()?;

        let pid = child.id() as i32;
        thread::spawn(move || {
            let mut signals = Signals::new([libc::SIGTERM, libc::SIGINT]).unwrap();
            for _signal in signals.forever() {
                let _ = killpg(Pid::from_raw(pid), Signal::SIGTERM);
            }
        });

        if let Some(input_data) = input {
            if let Some(stdin) = child.stdin().as_mut() {
                stdin.write_all(input_data.as_bytes())?;
            }
        }

        let output = child.wait_with_output()?;
        let trimmed_output = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if trimmed_output.is_empty() {
            Ok(None)
        } else {
            Ok(Some(trimmed_output))
        }
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
                    "xdg" => format!("{}\0icon\x1f{}", text, icon),
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
                    status_indicators.push_str(&format!(" [{}]", battery_icon));
                }
            } else if icon_type == "xdg" {
                status_indicators.push_str(&format!(" [{}%]", battery));
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
        menu_command: &Option<String>,
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
            input.push_str(&format!("\n{}", device_display));
        }

        for device in &controller.new_devices {
            let device_display = self.format_device_display(device, icon_type, spaces);
            input.push_str(&format!("\n{}", device_display));
        }

        let options_end = vec![("settings", settings_text.as_ref())];
        let settings_input = self.get_icon_text(options_end, icon_type, spaces);
        input.push_str(&format!("\n{}", settings_input));

        let menu_output = self.run_menu_command(menu_command, Some(&input), icon_type, None)?;

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
        menu_command: &Option<String>,
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
            input.push_str(&format!("{}\n", option_text));
        }

        let prompt = t!("menus.device.prompt", device_name = device_name);

        let menu_output =
            self.run_menu_command(menu_command, Some(&input), icon_type, Some(&prompt))?;

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
        menu_command: &Option<String>,
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

        let menu_output = self.run_menu_command(menu_command, Some(&input), icon_type, None)?;

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
        menu_command: &Option<String>,
        icon_type: &str,
        spaces: usize,
    ) -> Option<AdapterMenuOptions> {
        let options = vec![("power_on", AdapterMenuOptions::PowerOnDevice.to_str())];

        let input = self.get_icon_text(options, icon_type, spaces);

        if let Ok(Some(output)) = self.run_menu_command(menu_command, Some(&input), icon_type, None)
        {
            let cleaned_output = self.clean_menu_output(&output, icon_type);
            return AdapterMenuOptions::from_string(&cleaned_output);
        }

        None
    }

    pub fn prompt_passkey_confirmation(
        &self,
        menu_command: &Option<String>,
        device_name: &str,
        passkey: &str,
        icon_type: &str,
    ) -> Result<bool> {
        let prompt = t!(
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
            self.run_menu_command(menu_command, Some(&input), icon_type, Some(&prompt))?;

        if let Some(output) = menu_output {
            let cleaned_output = self.clean_menu_output(&output, icon_type);
            return Ok(cleaned_output == t!("menus.bluetooth.confirm"));
        }

        Ok(false)
    }
}

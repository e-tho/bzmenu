use std::collections::HashMap;

#[derive(Clone)]
pub struct Icons {
    generic_icons: HashMap<&'static str, char>,
    font_icons: HashMap<&'static str, char>,
    xdg_icons: HashMap<&'static str, &'static str>,
}

impl Icons {
    pub fn new() -> Self {
        let mut generic_icons = HashMap::new();
        let mut font_icons = HashMap::new();
        let mut xdg_icons = HashMap::new();

        generic_icons.insert("connected", '\u{23FA}');

        font_icons.insert("bluetooth", '\u{f293}');
        font_icons.insert("bluetooth_connected", '\u{f294}');
        font_icons.insert("connected", '\u{f294}');
        font_icons.insert("disconnected", '\u{f295}');
        font_icons.insert("connect", '\u{f0337}');
        font_icons.insert("disconnect", '\u{f0338}');
        font_icons.insert("scan", '\u{f46a}');
        font_icons.insert("settings", '\u{f08bb}');
        font_icons.insert("disable_adapter", '\u{f00b2}');
        font_icons.insert("power_on_device", '\u{f0425}');
        font_icons.insert("trust", '\u{f05e0}');
        font_icons.insert("forget", '\u{f0377}');

        font_icons.insert("enable_pairable", '\u{f0339}');
        font_icons.insert("disable_pairable", '\u{f033a}');

        font_icons.insert("enable_discoverable", '\u{f06d0}');
        font_icons.insert("disable_discoverable", '\u{f06d1}');

        font_icons.insert("device", '\u{f0fb0}');
        font_icons.insert("phone", '\u{f011c}');
        font_icons.insert("headphones", '\u{f02cb}');
        font_icons.insert("keyboard", '\u{f030c}');
        font_icons.insert("mouse", '\u{f037d}');
        font_icons.insert("speaker", '\u{f04c3}');
        font_icons.insert("gamepad", '\u{f0eb5}');
        font_icons.insert("computer", '\u{f0aab}');
        font_icons.insert("laptop", '\u{f0322}');
        font_icons.insert("tablet", '\u{f04f7}');
        font_icons.insert("watch", '\u{f0897}');
        font_icons.insert("tv", '\u{f0379}');
        font_icons.insert("display", '\u{f0379}');

        font_icons.insert("ok", '\u{f05e1}');
        font_icons.insert("error", '\u{f05d6}');
        font_icons.insert("paired", '\u{f119f}');
        font_icons.insert("trusted", '\u{f0cc8}');

        xdg_icons.insert("bluetooth", "bluetooth-symbolic");
        xdg_icons.insert("bluetooth_connected", "bluetooth-active-symbolic");
        xdg_icons.insert("connected", "bluetooth-active-symbolic");
        xdg_icons.insert("disconnected", "bluetooth-disabled-symbolic");
        xdg_icons.insert("connect", "entries-linked-symbolic");
        xdg_icons.insert("disconnect", "entries-unlinked-symbolic");
        xdg_icons.insert("scan", "emblem-synchronizing-symbolic");
        xdg_icons.insert("settings", "preferences-system-symbolic");
        xdg_icons.insert("disable_adapter", "bluetooth-disabled-symbolic");
        xdg_icons.insert("power_on_device", "bluetooth-symbolic");
        xdg_icons.insert("trust", "security-high-symbolic");
        xdg_icons.insert("forget", "list-remove-symbolic");

        xdg_icons.insert("enable_pairable", "network-transmit-receive-symbolic");
        xdg_icons.insert("disable_pairable", "network-offline-symbolic");

        xdg_icons.insert("enable_discoverable", "object-visible-symbolic");
        xdg_icons.insert("disable_discoverable", "object-hidden-symbolic");

        xdg_icons.insert("device", "drive-harddisk-symbolic");
        xdg_icons.insert("phone", "phone-symbolic");
        xdg_icons.insert("headphones", "audio-headphones-symbolic");
        xdg_icons.insert("keyboard", "input-keyboard-symbolic");
        xdg_icons.insert("mouse", "input-mouse-symbolic");
        xdg_icons.insert("speaker", "audio-speakers-symbolic");
        xdg_icons.insert("gamepad", "input-gaming-symbolic");
        xdg_icons.insert("computer", "computer-symbolic");
        xdg_icons.insert("laptop", "laptop-symbolic");
        xdg_icons.insert("tablet", "tablet-symbolic");
        xdg_icons.insert("watch", "smartwatch-symbolic");
        xdg_icons.insert("tv", "video-display-symbolic");
        xdg_icons.insert("display", "video-display-symbolic");

        xdg_icons.insert("ok", "emblem-default-symbolic");
        xdg_icons.insert("error", "dialog-error-symbolic");
        xdg_icons.insert("paired", "emblem-checked-symbolic");
        xdg_icons.insert("trusted", "security-high-symbolic");

        Icons {
            font_icons,
            xdg_icons,
            generic_icons,
        }
    }

    pub fn get_icon(&self, key: &str, icon_type: &str) -> String {
        match icon_type {
            "font" => self
                .font_icons
                .get(key)
                .map(|&icon| icon.to_string())
                .unwrap_or_default(),
            "xdg" => self
                .xdg_icons
                .get(key)
                .map(|&icon| icon.to_string())
                .unwrap_or_default(),
            "generic" => self
                .generic_icons
                .get(key)
                .map(|&icon| icon.to_string())
                .unwrap_or_default(),
            _ => String::new(),
        }
    }

    pub fn get_xdg_icon(&self, key: &str) -> String {
        self.xdg_icons
            .get(key)
            .map(|&icon| icon.to_string())
            .unwrap_or_default()
    }

    pub fn get_icon_text<T>(&self, items: Vec<(&str, T)>, icon_type: &str, spaces: usize) -> String
    where
        T: AsRef<str>,
    {
        items
            .into_iter()
            .map(|(icon_key, text)| {
                let icon = self.get_icon(icon_key, icon_type);
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

    pub fn format_with_spacing(icon: char, spaces: usize, before: bool) -> String {
        if before {
            format!("{}{}", " ".repeat(spaces), icon)
        } else {
            format!("{}{}", icon, " ".repeat(spaces))
        }
    }

    pub fn format_display_with_icon(
        &self,
        name: &str,
        icon: &str,
        icon_type: &str,
        spaces: usize,
    ) -> String {
        match icon_type {
            "xdg" => format!("{}\0icon\x1f{}", name, icon),
            "font" | "generic" => format!("{}{}{}", icon, " ".repeat(spaces), name),
            _ => name.to_string(),
        }
    }

    pub fn get_device_icon(&self, device_type: &str, icon_type: &str) -> String {
        let icon_key = match device_type {
            "phone" | "smartphone" => "phone",
            "audio" | "headset" | "headphones" => "headphones",
            "keyboard" => "keyboard",
            "mouse" | "pointing" => "mouse",
            "speaker" => "speaker",
            "gamepad" | "joystick" => "gamepad",
            "computer" | "desktop" => "computer",
            "laptop" => "laptop",
            "tablet" => "tablet",
            "watch" | "wearable" => "watch",
            "tv" | "television" | "display" => "tv",
            _ => "device",
        };

        self.get_icon(icon_key, icon_type)
    }
}

impl Default for Icons {
    fn default() -> Self {
        Self::new()
    }
}

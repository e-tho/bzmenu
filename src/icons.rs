use std::collections::HashMap;

#[derive(Clone)]
pub struct IconDefinition {
    single: String,
    list: String,
}

impl IconDefinition {
    pub fn simple(icon: &str) -> Self {
        Self {
            single: icon.to_string(),
            list: icon.to_string(),
        }
    }

    pub fn with_fallbacks(single: Option<&str>, list: &str) -> Self {
        let single_icon = match single {
            Some(icon) => icon.to_string(),
            None => list.split(',').next().unwrap_or("").trim().to_string(),
        };

        Self {
            single: single_icon,
            list: list.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct Icons {
    generic_icons: HashMap<&'static str, char>,
    font_icons: HashMap<&'static str, char>,
    xdg_icons: HashMap<&'static str, IconDefinition>,
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
        font_icons.insert("trust", '\u{f0cc8}');
        font_icons.insert("revoke_trust", '\u{f099c}');
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

        font_icons.insert("battery_100", '\u{f0079}');
        font_icons.insert("battery_90", '\u{f0082}');
        font_icons.insert("battery_80", '\u{f0081}');
        font_icons.insert("battery_70", '\u{f0080}');
        font_icons.insert("battery_60", '\u{f007f}');
        font_icons.insert("battery_50", '\u{f007e}');
        font_icons.insert("battery_40", '\u{f007d}');
        font_icons.insert("battery_30", '\u{f007c}');
        font_icons.insert("battery_20", '\u{f007b}');
        font_icons.insert("battery_10", '\u{f007a}');

        font_icons.insert("ok", '\u{f05e1}');
        font_icons.insert("error", '\u{f05d6}');
        font_icons.insert("paired", '\u{f119f}');
        font_icons.insert("trusted", '\u{f0cc8}');

        xdg_icons.insert(
            "bluetooth",
            IconDefinition::with_fallbacks(
                None,
                "bluetooth-symbolic,network-bluetooth-symbolic,bluetooth",
            ),
        );
        xdg_icons.insert(
            "connected",
            IconDefinition::with_fallbacks(
                None,
                "bluetooth-active-symbolic,network-bluetooth-activated-symbolic,bluetooth-active",
            ),
        );
        xdg_icons.insert(
            "disconnected",
            IconDefinition::with_fallbacks(None, "bluetooth-disabled-symbolic,network-bluetooth-inactive-symbolic,bluetooth-disabled"),
        );
        xdg_icons.insert(
            "connect",
            IconDefinition::with_fallbacks(
                Some("network-connect-symbolic"),
                "entries-linked-symbolic,network-connect-symbolic,link-symbolic",
            ),
        );
        xdg_icons.insert(
            "disconnect",
            IconDefinition::with_fallbacks(
                Some("network-disconnect-symbolic"),
                "entries-unlinked-symbolic,network-disconnect-symbolic,media-eject-symbolic",
            ),
        );
        xdg_icons.insert(
            "scan",
            IconDefinition::with_fallbacks(
                None,
                "sync-synchronizing-symbolic,emblem-synchronizing-symbolic,view-refresh-symbolic",
            ),
        );
        xdg_icons.insert(
            "scan_in_progress",
            IconDefinition::simple("bluetooth-acquiring-symbolic"),
        );
        xdg_icons.insert(
            "settings",
            IconDefinition::simple("preferences-system-symbolic"),
        );
        xdg_icons.insert(
            "disable_adapter",
            IconDefinition::with_fallbacks(
                None,
                "bluetooth-disabled-symbolic,network-bluetooth-inactive-symbolic",
            ),
        );
        xdg_icons.insert(
            "power_on_device",
            IconDefinition::simple("bluetooth-symbolic"),
        );
        xdg_icons.insert("trust", IconDefinition::simple("emblem-default-symbolic"));
        xdg_icons.insert(
            "revoke_trust",
            IconDefinition::simple("action-unavailable-symbolic"),
        );
        xdg_icons.insert("forget", IconDefinition::simple("list-remove-symbolic"));

        xdg_icons.insert(
            "enable_pairable",
            IconDefinition::simple("changes-allow-symbolic"),
        );
        xdg_icons.insert(
            "disable_pairable",
            IconDefinition::simple("changes-prevent-symbolic"),
        );
        xdg_icons.insert(
            "enable_discoverable",
            IconDefinition::with_fallbacks(
                None,
                "view-reveal-symbolic,view-visible-symbolic,object-visible-symbolic",
            ),
        );
        xdg_icons.insert(
            "disable_discoverable",
            IconDefinition::with_fallbacks(
                None,
                "view-conceal-symbolic,view-hidden-symbolic,object-hidden-symbolic",
            ),
        );

        xdg_icons.insert("device", IconDefinition::simple("drive-harddisk-symbolic"));
        xdg_icons.insert(
            "phone",
            IconDefinition::with_fallbacks(None, "phone-symbolic,drive-harddisk-symbolic"),
        );
        xdg_icons.insert(
            "headphones",
            IconDefinition::with_fallbacks(
                None,
                "audio-headphones-symbolic,drive-harddisk-symbolic",
            ),
        );
        xdg_icons.insert(
            "keyboard",
            IconDefinition::with_fallbacks(None, "input-keyboard-symbolic,drive-harddisk-symbolic"),
        );
        xdg_icons.insert(
            "mouse",
            IconDefinition::with_fallbacks(None, "input-mouse-symbolic,drive-harddisk-symbolic"),
        );
        xdg_icons.insert(
            "speaker",
            IconDefinition::with_fallbacks(None, "audio-speakers-symbolic,drive-harddisk-symbolic"),
        );
        xdg_icons.insert(
            "gamepad",
            IconDefinition::with_fallbacks(
                None,
                "input-gaming-symbolic,input-gamepad-symbolic,drive-harddisk-symbolic",
            ),
        );
        xdg_icons.insert(
            "computer",
            IconDefinition::with_fallbacks(None, "computer-symbolic,drive-harddisk-symbolic"),
        );
        xdg_icons.insert(
            "laptop",
            IconDefinition::with_fallbacks(
                None,
                "laptop-symbolic,computer-laptop-symbolic,computer-symbolic,drive-harddisk-symbolic",
            ),
        );
        xdg_icons.insert(
            "tablet",
            IconDefinition::with_fallbacks(None, "tablet-symbolic,drive-harddisk-symbolic"),
        );
        xdg_icons.insert(
            "watch",
            IconDefinition::with_fallbacks(None, "smartwatch-symbolic,drive-harddisk-symbolic"),
        );
        xdg_icons.insert(
            "tv",
            IconDefinition::with_fallbacks(
                None,
                "video-display-symbolic,preferences-desktop-display-randr-symbolic,drive-harddisk-symbolic",
            ),
        );
        xdg_icons.insert("display", IconDefinition::with_fallbacks(None,"video-display-symbolic,preferences-desktop-display-randr-symbolic,drive-harddisk-symbolic"));

        xdg_icons.insert(
            "battery_100",
            IconDefinition::with_fallbacks(
                Some("battery-full-symbolic"),
                "battery-100-symbolic,battery-full-symbolic",
            ),
        );
        xdg_icons.insert(
            "battery_90",
            IconDefinition::with_fallbacks(
                Some("battery-good-symbolic"),
                "battery-090-symbolic,battery-good-symbolic",
            ),
        );
        xdg_icons.insert(
            "battery_80",
            IconDefinition::with_fallbacks(
                Some("battery-good-symbolic"),
                "battery-080-symbolic,battery-good-symbolic",
            ),
        );
        xdg_icons.insert(
            "battery_70",
            IconDefinition::with_fallbacks(
                Some("battery-good-symbolic"),
                "battery-070-symbolic,battery-good-symbolic",
            ),
        );
        xdg_icons.insert(
            "battery_60",
            IconDefinition::with_fallbacks(
                Some("battery-good-symbolic"),
                "battery-060-symbolic,battery-good-symbolic",
            ),
        );
        xdg_icons.insert(
            "battery_50",
            IconDefinition::with_fallbacks(
                Some("battery-medium-symbolic"),
                "battery-050-symbolic,battery-medium-symbolic",
            ),
        );
        xdg_icons.insert(
            "battery_40",
            IconDefinition::with_fallbacks(
                Some("battery-medium-symbolic"),
                "battery-040-symbolic,battery-medium-symbolic",
            ),
        );
        xdg_icons.insert(
            "battery_30",
            IconDefinition::with_fallbacks(
                Some("battery-low-symbolic"),
                "battery-030-symbolic,battery-low-symbolic",
            ),
        );
        xdg_icons.insert(
            "battery_20",
            IconDefinition::with_fallbacks(
                Some("battery-low-symbolic"),
                "battery-020-symbolic,battery-low-symbolic",
            ),
        );
        xdg_icons.insert(
            "battery_10",
            IconDefinition::with_fallbacks(
                Some("battery-caution-symbolic"),
                "battery-010-symbolic,battery-caution-symbolic",
            ),
        );

        xdg_icons.insert("ok", IconDefinition::simple("emblem-default-symbolic"));
        xdg_icons.insert("error", IconDefinition::simple("dialog-error-symbolic"));
        xdg_icons.insert("paired", IconDefinition::simple("emblem-checked-symbolic"));
        xdg_icons.insert("trusted", IconDefinition::simple("security-high-symbolic"));

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
                .map(|icon_definition| icon_definition.list.clone())
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
            .map(|icon_def| icon_def.single.clone())
            .unwrap_or_default()
    }

    pub fn get_xdg_icon_list(&self, key: &str) -> String {
        self.xdg_icons
            .get(key)
            .map(|icon_def| icon_def.list.clone())
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
                    "xdg" => format!("{text}\0icon\x1f{icon}"),
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
            "xdg" => format!("{name}\0icon\x1f{icon}"),
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

    pub fn get_battery_icon(&self, percentage: u8, icon_type: &str) -> Option<String> {
        let icon_key = match percentage {
            91..=100 => "battery_100",
            81..=90 => "battery_90",
            71..=80 => "battery_80",
            61..=70 => "battery_70",
            51..=60 => "battery_60",
            41..=50 => "battery_50",
            31..=40 => "battery_40",
            21..=30 => "battery_30",
            11..=20 => "battery_20",
            0..=10 => "battery_10",
            _ => return None,
        };

        Some(self.get_icon(icon_key, icon_type))
    }
}

impl Default for Icons {
    fn default() -> Self {
        Self::new()
    }
}

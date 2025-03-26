use std::sync::Arc;

use anyhow::Result;
use bluer::{Adapter, Address, Device as BluerDevice};

#[derive(Debug, Clone)]
pub struct Device {
    device: BluerDevice,
    adapter: Arc<Adapter>,
    pub addr: Address,
    pub icon: Option<String>,
    pub device_type: String,
    pub alias: String,
    pub is_paired: bool,
    pub is_trusted: bool,
    pub is_connected: bool,
    pub battery_percentage: Option<u8>,
}

impl Device {
    pub async fn new(adapter: &Adapter, addr: &Address) -> Result<Self> {
        let device = adapter.device(*addr)?;

        let alias = device.alias().await?;
        let icon_name = device.icon().await?.unwrap_or_default();
        let icon = if !icon_name.is_empty() {
            Some(icon_name.clone())
        } else {
            None
        };

        let device_type = Self::determine_device_type(&device).await?;

        let is_paired = device.is_paired().await?;
        let is_trusted = device.is_trusted().await?;
        let is_connected = device.is_connected().await?;
        let battery_percentage = device.battery_percentage().await.ok().flatten();

        Ok(Self {
            device,
            adapter: Arc::new(adapter.clone()),
            addr: *addr,
            icon,
            device_type,
            alias,
            is_paired,
            is_trusted,
            is_connected,
            battery_percentage,
        })
    }

    async fn determine_device_type(device: &BluerDevice) -> Result<String> {
        if let Ok(Some(class_value)) = device.class().await {
            let major_class = (class_value >> 8) & 0x1F;
            let minor_class = (class_value >> 2) & 0x3F;

            let device_type = match major_class {
                0x01 => "computer", // Computer
                0x02 => match minor_class {
                    0x01 => "phone",    // Cellular
                    0x02 => "modem",    // Cordless
                    0x03 => "phone",    // Smartphone/PDA
                    0x04 => "computer", // Desktop
                    0x05 => "computer", // Server
                    0x06 => "laptop",   // Laptop
                    0x07 => "tablet",   // Tablet
                    _ => "phone",       // Generic phone
                },
                0x03 => "network", // LAN/Network Access Point
                0x04 => match minor_class {
                    0x01 => "headphones", // Headset
                    0x02 => "headphones", // Hands-free
                    0x04 => "microphone", // Microphone
                    0x05 => "speaker",    // Speaker
                    0x06 => "headphones", // Headphones
                    0x08 => "speaker",    // Car audio
                    0x09 => "tv",         // Video display
                    0x0A => "speaker",    // Loudspeaker
                    _ => "audio",         // Generic audio
                },
                0x05 => match minor_class {
                    0x01 => "keyboard",  // Keyboard
                    0x02 => "mouse",     // Mouse
                    0x03 => "trackball", // Trackball
                    0x04 => "joystick",  // Joystick
                    0x05 => "gamepad",   // Gamepad/Controller
                    0x06 => "tablet",    // Digitizer tablet
                    0x07 => "mouse",     // Card reader
                    0x08 => "pen",       // Digital pen
                    _ => "peripheral",   // Generic peripheral
                },
                0x06 => match minor_class {
                    0x01 | 0x02 => "printer", // Printer
                    0x04 | 0x08 => "camera",  // Camera
                    0x10 => "display",        // Display
                    0x20 => "tv",             // Television
                    _ => "imaging",           // Generic imaging
                },
                0x07 => match minor_class {
                    0x01 => "watch",      // Wrist watch
                    0x02 => "glasses",    // Smart glasses
                    0x03 => "wearable",   // Generic wearable
                    0x04 => "headphones", // Sports watch
                    _ => "wearable",      // Generic wearable
                },
                _ => "", // No type identified, continue with other methods
            };

            if !device_type.is_empty() {
                return Ok(device_type.to_string());
            }
        }

        if let Ok(Some(appearance)) = device.appearance().await {
            let device_type = match appearance {
                0 => "",                 // Unknown
                64 => "phone",           // Generic Phone
                128 => "computer",       // Generic Computer
                192 => "watch",          // Generic Watch
                256 => "display",        // Generic Display
                512 => "remote",         // Generic Remote Control
                576 => "scanner",        // Generic Eye-glasses
                640 => "tag",            // Generic Tag
                704 => "keyring",        // Generic Keyring
                768 => "media",          // Generic Media Player
                832 => "barcode",        // Generic Barcode Scanner
                896 => "thermometer",    // Generic Thermometer
                960 => "peripheral",     // Generic HID
                961 => "keyboard",       // Keyboard
                962 => "mouse",          // Mouse
                963 => "joystick",       // Joystick
                964 => "gamepad",        // Gamepad
                976 => "digitizer",      // Digitizer Tablet
                1024 => "reader",        // Generic Card Reader
                1088 => "pen",           // Digital Pen
                1152 => "scanner",       // Generic Barcode Scanner
                1216 => "speaker",       // Generic Audio
                1280 => "headphones",    // Generic Audio: Headset
                1344 => "speaker",       // Generic Audio: Speaker
                1408 => "microphone",    // Generic Audio: Microphone
                1472 => "audio",         // Generic Audio: Hearing aid
                1600..=1663 => "health", // Health devices
                1664..=1727 => "sensor", // Environmental sensors
                _ => "",                 // Unrecognized, continue
            };

            if !device_type.is_empty() {
                return Ok(device_type.to_string());
            }
        }

        if let Ok(Some(uuids)) = device.uuids().await {
            for uuid in uuids {
                let uuid_str = uuid.to_string();

                match &uuid_str[..] {
                    // Audio
                    "0000110b-0000-1000-8000-00805f9b34fb" => return Ok("audio".to_string()), // A/V Remote Control
                    "0000110c-0000-1000-8000-00805f9b34fb" => return Ok("headphones".to_string()), // A/V Remote Control Target
                    "0000110e-0000-1000-8000-00805f9b34fb" => return Ok("headphones".to_string()), // A/V Remote Control Controller
                    "0000110f-0000-1000-8000-00805f9b34fb" => return Ok("speaker".to_string()), // Advanced Audio Distribution
                    "00001112-0000-1000-8000-00805f9b34fb" => return Ok("headphones".to_string()), // Headset
                    "00001117-0000-1000-8000-00805f9b34fb" => return Ok("speaker".to_string()), // AVCTP
                    "00001131-0000-1000-8000-00805f9b34fb" => return Ok("headphones".to_string()), // Phone book server
                    "00001132-0000-1000-8000-00805f9b34fb" => return Ok("phone".to_string()), // Message Access Server

                    // Peripherals
                    "00001124-0000-1000-8000-00805f9b34fb" => return Ok("keyboard".to_string()), // HID
                    "00001812-0000-1000-8000-00805f9b34fb" => return Ok("peripheral".to_string()), // HID over GATT

                    // Sensors
                    "0000180d-0000-1000-8000-00805f9b34fb" => return Ok("health".to_string()), // Heart Rate
                    "0000180f-0000-1000-8000-00805f9b34fb" => return Ok("battery".to_string()), // Battery Service

                    // Generic, continue
                    "00001800-0000-1000-8000-00805f9b34fb" => {} // Generic Access
                    "00001801-0000-1000-8000-00805f9b34fb" => {} // Generic Attribute
                    "0000180a-0000-1000-8000-00805f9b34fb" => {} // Device Information

                    _ => {}
                }
            }
        }

        if let Ok(Some(icon_str)) = device.icon().await {
            let device_type = match icon_str.as_str() {
                "audio-card" | "audio-speakers" => "speaker",
                "audio-headphones" | "audio-headset" => "headphones",
                "input-keyboard" => "keyboard",
                "input-mouse" => "mouse",
                "input-gaming" | "input-joystick" => "gamepad",
                "phone" => "phone",
                "computer" => "computer",
                "computer-laptop" => "laptop",
                "video-display" | "tv" => "tv",
                _ => icon_str.as_str(),
            };

            return Ok(device_type.to_string());
        }

        Ok("device".to_string())
    }

    pub async fn connect(&self) -> Result<()> {
        self.device.connect().await?;
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<()> {
        self.device.disconnect().await?;
        Ok(())
    }

    pub async fn pair(&self) -> Result<()> {
        self.device.pair().await?;
        Ok(())
    }

    pub async fn set_trusted(&self, trusted: bool) -> Result<()> {
        self.device.set_trusted(trusted).await?;
        Ok(())
    }

    pub async fn forget(&self) -> Result<()> {
        self.adapter.remove_device(self.addr).await?;
        Ok(())
    }
}

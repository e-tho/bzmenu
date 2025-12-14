#[macro_use]
extern crate rust_i18n;
#[macro_use]
mod macros;
i18n!("locales", fallback = "en");

pub mod app;
pub mod icons;
pub mod launcher;
pub mod menu;
pub mod notification;
pub mod bz {
    pub mod agent;
    pub mod controller;
    pub mod device;
    pub mod pairing;
    pub mod scanner;
}

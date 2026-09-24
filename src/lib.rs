#[macro_use]
extern crate extension_traits;
#[macro_use]
extern crate rust_i18n;

mod analytics;
mod curseforge;
mod discord;
mod feature_flags;
mod util;
pub mod web;

i18n!("lang", fallback = "en");

pub const USER_AGENT: &str = "mods.cf/Service (+https://github.com/Up-Mods/mods.cf)";

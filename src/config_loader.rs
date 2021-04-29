use config::{Config, ConfigError, File};
use std::net::{Ipv4Addr};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
	pub server:Ipv4Addr,
	pub port:u16,
	pub data_dir:String,
	pub data_url:String,
	pub static_dir:String,
	pub cert_path:String,
	pub key_path:String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name("./config.json"))?;
        s.try_into()
    }
}
use config::{Config, ConfigError, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
	pub server:String,
	pub port:String,
	pub data_dir:String,
	pub data_url:String,
	pub num_threads:usize
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name("./config.json"))?;
        s.try_into()
    }
}
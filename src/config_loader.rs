use serde::Deserialize;
use std::env;
use std::fs;
use std::net::Ipv4Addr;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
	pub NLR_ADDR: Ipv4Addr,
	pub NLR_PORT: u16,
	pub NLR_DATA_FILE: String,
	pub NLR_DATA_SOURCE_URL: String,
	pub NLR_STATIC_HTTP: String,
	pub NLR_CERT_PATH: String,
	pub NLR_PRIVATE_KEY_PATH: String,
}

impl Settings {
	pub fn default() -> Settings {
		Settings {
			NLR_ADDR:Ipv4Addr::new(0,0,0,0),
			NLR_PORT:8080,
			NLR_DATA_FILE:"./data/data.json.lz4".to_string(),
			NLR_DATA_SOURCE_URL:"https://mrgis.mainroads.wa.gov.au/arcgis/rest/services/OpenData/RoadAssets_DataPortal/MapServer/17/query?where=1%3D1&outFields=ROAD,START_SLK,END_SLK,CWY&outSR=4326&f=json".to_string(),
			NLR_STATIC_HTTP:"./__static_http".to_string(),
			NLR_CERT_PATH:"./certs/public.crt".to_string(),
			NLR_PRIVATE_KEY_PATH:"./certs/decrypted_private.key".to_string()
		}
	}

	pub fn get() -> Result<Self, Box<dyn std::error::Error>> {
		let mut settings: Settings = match env::args().skip_while(|item| item != "--config").nth(1){
			Some(path) => {
				println!("Found config file based on --config command line argument: {}", path);
				match fs::File::open(&path) {
					Ok(config_file) => match serde_json::from_reader(config_file) {
						Ok(config) => config,
						Err(e) => return Err(Box::new(e)),
					},
					Err(e) => {
						// The user specifically provided a path to a config .json file,
						//  Therefore this error is not recoverable by reverting to default config options
						//  or by looking for environment variables
						println!(
							"Error reading config from path provided in arguments {}. Fatal.",
							path
						);
						return Err(Box::new(e));
					}
				}
			}
			None => {
				println!("--config command line argument not provided. Using default settings.");
				Settings::default()
			},
		};
		println!("Will try to override any config using any available environment variables:");
		// override whatever we got with environment variables.
		read_env_var_with_fallback("NLR_ADDR", &mut settings.NLR_ADDR);
		read_env_var_with_fallback("NLR_PORT", &mut settings.NLR_PORT);
		read_env_var_with_fallback("NLR_DATA_FILE", &mut settings.NLR_DATA_FILE);
		read_env_var_with_fallback("NLR_DATA_SOURCE_URL", &mut settings.NLR_DATA_SOURCE_URL);
		read_env_var_with_fallback("NLR_STATIC_HTTP", &mut settings.NLR_STATIC_HTTP);
		read_env_var_with_fallback("NLR_CERT_PATH", &mut settings.NLR_CERT_PATH);
		read_env_var_with_fallback("NLR_KEY_PATH", &mut settings.NLR_PRIVATE_KEY_PATH);

		Ok(settings)
	}
}

fn read_env_var_with_fallback<T>(env_var: &str, out: &mut T)
where
	T: std::str::FromStr + Clone+ std::fmt::Display,
	<T as std::str::FromStr>::Err : std::error::Error
{
	*out = match env::var(env_var) {
		Ok(val_str) => match val_str.parse::<T>() {
			Ok(val) => {
				println!("   {} = {}", env_var, val_str);
				val
			},
			Err(e) => {
				// TODO: This should probably be a fatal error.
				println!("   {} failed to parse! Value '{}' could not be parsed because {}. Will continue with {}", env_var, val_str, e, out);
				out.clone()
			},
		},
		_ => {
			println!("   {} not found. Using {}", env_var, out);
			out.clone()
		}
	};
}

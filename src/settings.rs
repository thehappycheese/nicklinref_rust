use serde::Deserialize;
use std::env;
use std::fs;
use std::net::{IpAddr, Ipv4Addr};


#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    /// The IP address to listen on
    pub NLR_ADDR: IpAddr,
    /// The port to listen on
    pub NLR_PORT: u16,
    /// The location of data.json.lz4
    pub NLR_DATA_FILE: String,
    /// the url of the esri rest service hosting the road network data
    pub NLR_DATA_SOURCE_URL: String,
    /// The folder containing static http files for the /show/ route
    pub NLR_STATIC_HTTP: String,
}


impl Default for Settings {
    fn default() -> Settings {
        Settings {
            NLR_ADDR:IpAddr::V4(Ipv4Addr::new(127,0,0,1)),
            NLR_PORT:8080,
            NLR_DATA_FILE:"./data/data.json.lz4".to_string(),
            NLR_DATA_SOURCE_URL:"https://mrgis.mainroads.wa.gov.au/arcgis/rest/services/OpenData/RoadAssets_DataPortal/MapServer/17/query?where=1%3D1&outFields=ROAD,START_SLK,END_SLK,CWY&outSR=4326&f=json".to_string(),
            NLR_STATIC_HTTP:"./__static_http".to_string(),
        }
    }
}

impl Settings {
    pub fn load_settings() -> Result<Self, Box<dyn std::error::Error>> {
        let mut settings: Settings = match env::args().skip_while(|item| item != "--config").nth(1){
            // TODO: check this works with spaces and quotes etc
            Some(path) => {
                println!("Loading settings from '{}'", path);
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
                println!("`--config \"some_config.json\"` command line argument not found.");
                println!("Using default settings.");
                Settings::default()
            },
        };
        println!("Will try to override any config using any available environment variables:");
        // override whatever we got with environment variables.
        overwrite_if_environment_variable_exists("NLR_ADDR", &mut settings.NLR_ADDR);
        overwrite_if_environment_variable_exists("NLR_PORT", &mut settings.NLR_PORT);
        overwrite_if_environment_variable_exists("NLR_DATA_FILE", &mut settings.NLR_DATA_FILE);
        overwrite_if_environment_variable_exists("NLR_DATA_SOURCE_URL", &mut settings.NLR_DATA_SOURCE_URL);
        overwrite_if_environment_variable_exists("NLR_STATIC_HTTP", &mut settings.NLR_STATIC_HTTP);

        Ok(settings)
    }

}


// TODO: To emit the fatal error below, this function must return a result.
fn overwrite_if_environment_variable_exists<T>(environment_variable_name: &str, output: &mut T)
where
    T: std::str::FromStr + Clone+ std::fmt::Display,
    <T as std::str::FromStr>::Err : std::error::Error
{
    *output = match env::var(environment_variable_name) {
        Ok(val_str) => match val_str.parse::<T>() {
            Ok(val) => {
                println!("   {} = {}", environment_variable_name, val_str);
                val
            },
            Err(e) => {
                println!("   {} found but failed to parse! The provided value '{}' could not be parsed because {}.", environment_variable_name, val_str, e);				
                panic!("Please update environment variable `{}`.", environment_variable_name);
            },
        },
        _ => {
            println!("   {} not found. Using: {}", environment_variable_name, output);
            output.clone()
        }
    };
}

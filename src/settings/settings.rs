use serde::Deserialize;
use serde_json;
use std::{net::{IpAddr, Ipv4Addr}, fs::File};
use clap::Parser;

use crate::helpers::{ErrorWithDynamicMessage};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Clone, Parser)]
pub struct Settings {
    /// The IP address to listen on
    #[clap(
        long="addr",
        env="NLR_ADDR",
        default_value_t = IpAddr::V4(Ipv4Addr::new(127,0,0,1)),
    )]
    pub NLR_ADDR: IpAddr,

    /// The port to listen on
    #[clap(
        long="port",
        env="NLR_PORT",
        default_value_t = 8080,
    )]
    pub NLR_PORT: u16,

    /// The desired path to the cache data file, including the file name.
    #[clap(
        long="data-file",
        env="NLR_DATA_FILE",
        default_value = "./data/data.json.lz4",
        value_hint=clap::ValueHint::FilePath
    )]
    pub NLR_DATA_FILE: String,

    /// The folder containing static http files for the /show/ route
    #[clap(
        long="static-http",
        env="NLR_STATIC_HTTP",
        default_value = "./__static_http",
        value_hint=clap::ValueHint::DirPath
    )]
    pub NLR_STATIC_HTTP: String,

    /// The url of the esri rest service hosting the road network data
    #[clap(
        long="data-source-url",
        env="NLR_DATA_SOURCE_URL",
        default_value = "https://mrgis.mainroads.wa.gov.au/arcgis/rest/services/OpenData/RoadAssets_DataPortal/MapServer/17/query?where=1%3D1&outFields=ROAD,START_SLK,END_SLK,CWY&outSR=4326&f=json",
        value_hint=clap::ValueHint::Url
    )]
    pub NLR_DATA_SOURCE_URL: String,
}


// impl Default for Settings {
//     fn default() -> Settings {
//         Settings {
//             NLR_ADDR:IpAddr::V4(Ipv4Addr::new(127,0,0,1)),
//             NLR_PORT:8080,
//             NLR_DATA_FILE:"./data/data.json.lz4".to_string(),
//             NLR_DATA_SOURCE_URL:"https://mrgis.mainroads.wa.gov.au/arcgis/rest/services/OpenData/RoadAssets_DataPortal/MapServer/17/query?where=1%3D1&outFields=ROAD,START_SLK,END_SLK,CWY&outSR=4326&f=json".to_string(),
//             NLR_STATIC_HTTP:"./__static_http".to_string(),
//         }
//     }
// }

#[derive(Parser)]
struct ConfigFileReader{
    /// The path to the JSON settings file
    #[clap(
        long="config",
        value_hint=clap::ValueHint::FilePath
    )]
    path_to_config_file: String
}


impl Settings {
    
    /// `load_settings()` is responsible for loading settings for the application.
    ///
    /// The function uses the following precedence order for loading settings:
    /// 
    /// 1. Command line argument: The function first checks for a `--config`
    ///    command line argument.
    ///    - If this argument is provided and the file is successfully opened
    ///      and read, the settings in the file will be loaded.
    ///    - If the file cannot be opened or read, the function will panic and
    ///      terminate the application.
    /// 
    /// 2. Default settings: If no `--config` argument is provided, the function
    ///    will load the default settings.
    /// 
    /// 3. Environment variables: After the settings are loaded from the JSON
    ///    file or the default settings, the or the default settings are used,
    ///    the function attempts to override these settings with any environment
    ///    variables that are set.
    ///    - If an environment variable is found but cannot be parsed to the
    ///      required type, the function will panic and terminate the
    ///      application.
    ///
    /// # Errors
    /// The function returns an error if it cannot read the settings from the
    /// specified JSON file. It will also panic and terminate the application if an environment variable is found but cannot be parsed to the required type.
    ///
    /// # Panics
    /// The function panics and terminates the application under the following conditions:
    /// - If the JSON settings file specified by the user via `--config` argument cannot be opened or read.
    /// - If an environment variable exists but cannot be parsed to the required type.
    pub fn load_settings() -> Result<Self, ErrorWithDynamicMessage> {

        match ConfigFileReader::try_parse(){
            Ok(ConfigFileReader{path_to_config_file}) => {
                println!("Will try to read from '{:}'", path_to_config_file);
                match File::open(&path_to_config_file) {
                    Ok(config_file) => match serde_json::from_reader(config_file) {
                        Ok(settings) => Ok(settings),
                        Err(e) => Err(ErrorWithDynamicMessage::new(format!("Could not parse the config file that was specified using --config: {:}", e)))
                    },
                    Err(e) => Err(ErrorWithDynamicMessage::new(format!("Could not open the config file that was specified using --config: {:}", e)))
                }
            },
            Err(_)=>Ok(Settings::parse())
        }
    }
}
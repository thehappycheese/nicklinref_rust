use serde::Deserialize;
use std::{
    net::{IpAddr, Ipv4Addr},
};
use clap::Parser;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Clone, Parser, PartialEq)]
#[command(name="nicklinref")]
#[command(version)]
#[command(about, long_about=None)]
pub struct Settings {

    #[clap(
        long="ip-address",
        env="NLR_ADDR",
        default_value_t = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
    )]
    /// The IP address to listen on
    pub NLR_ADDR: IpAddr,

    #[clap(
        long="port",
        env="NLR_PORT",
        default_value_t = 8080,
    )]
    /// The port to listen on
    pub NLR_PORT: u16,

    #[clap(
        long="data-file",
        env="NLR_DATA_FILE",
        default_value = "./data/data.json.lz4",
        value_hint=clap::ValueHint::FilePath
    )]
    /// File path to where the cache data file is/will be stored, including file name
    pub NLR_DATA_FILE: String,

    #[clap(
        long="static-http",
        env="NLR_STATIC_HTTP",
        default_value = "./__static_http",
        value_hint=clap::ValueHint::DirPath
    )]
    /// Folder path containing static http files for the /show/ route
    pub NLR_STATIC_HTTP: String,

    #[clap(
        long="force-update-data",
        env="NLR_FORCE_UPDATE_DATA",
        default_value_t = false,
    )]
    /// Cause the old data cache file to be deleted and re-downloaded
    pub NLR_FORCE_UPDATE_DATA: bool,

    #[clap(
        long="data-source-url",
        env="NLR_DATA_SOURCE_URL",
        default_value = "https://mrgis.mainroads.wa.gov.au/arcgis/rest/services/OpenData/RoadAssets_DataPortal/MapServer/17/query?where=1%3D1&outFields=ROAD,START_SLK,END_SLK,CWY&outSR=4326&f=json",
        value_hint=clap::ValueHint::Url
    )]
    /// Url of the esri rest service hosting the road network data
    pub NLR_DATA_SOURCE_URL: String,

}




#[cfg(test)]
mod tests {

    use super::*;

    impl Default for Settings {
        fn default() -> Self {
            Settings::parse_from([""].into_iter())
        }
    }

    #[test]
    /// a few spot-checks for the key default settings
    /// it is important that these are stable and match the docs
    fn test_settings_defaults(){
        let defaults = Settings::default();
        assert_eq!(defaults.NLR_ADDR, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        assert_eq!(defaults.NLR_PORT, 8080);
        assert_eq!(defaults.NLR_FORCE_UPDATE_DATA, false);
    }


    #[test]
    /// Test that the settings can be parsed from a command line
    fn test_settings_parse_from_args(){
        let simulated_args = [
            "none.exe",
            "--ip-address", "127.0.0.5",
            "--port", "8093",
            "--data-file", "none.lz4",
            "--static-http", "__static_none",
            "--force-update-data",
            "--data-source-url", "https://none.none.none",
        ];
        let settings = Settings::parse_from(simulated_args.into_iter());
        assert_eq!(settings, Settings{
            NLR_ADDR: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 5)),
            NLR_PORT: 8093,
            NLR_DATA_FILE: "none.lz4".to_string(),
            NLR_STATIC_HTTP: "__static_none".to_string(),
            NLR_FORCE_UPDATE_DATA: true,
            NLR_DATA_SOURCE_URL: "https://none.none.none".to_string(),
        });
    }

    #[test]
    /// confirm that clap will not accept unexpected arguments such as `--pux`
    fn test_settings_unexpected_extra_arg(){
        let settings = Settings::try_parse_from([
            "none.exe",
            "--pux",
            "--ip-address", "127.0.0.5"
        ].into_iter());
        assert!(settings.is_err());
    }

    #[tokio::test]
    /// The default data url is hard-coded, we cant know if it will be reliably
    /// available into the future, but at least we can do a quick check during
    /// testing so see if it is up and running.
    async fn test_settings_default_url_has_data() {
        // modify the url to return just the count
        let url = format!("{}&returnCountOnly=true", Settings::default().NLR_DATA_SOURCE_URL);
        #[derive(Deserialize, Debug)]
        /// create a quick struct to capture the result
        struct CountResponse {
            count: u32
        }
        let a:CountResponse = reqwest::get(url).await.unwrap().json().await.unwrap();
        assert!(a.count > 180_000); // was 184011 at last check
    }
}
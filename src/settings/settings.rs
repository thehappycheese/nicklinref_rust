use serde::Deserialize;
use std::{
    net::{IpAddr, Ipv4Addr},
};
use clap::Parser;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Clone, Parser)]
#[command(name="NickLinRef")]
#[command(version)]
#[command(about, long_about=None)]
pub struct Settings {
    /// The IP address to listen on
    #[clap(
        long="ip-address",
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

    /// File path to where the cache data file is/will be stored, including file name
    #[clap(
        long="data-file",
        env="NLR_DATA_FILE",
        default_value = "./data/data.json.lz4",
        value_hint=clap::ValueHint::FilePath
    )]
    pub NLR_DATA_FILE: String,

    /// Folder path containing static http files for the /show/ route
    #[clap(
        long="static-http",
        env="NLR_STATIC_HTTP",
        default_value = "./__static_http",
        value_hint=clap::ValueHint::DirPath
    )]
    pub NLR_STATIC_HTTP: String,

    /// Cause the old data cache file to be deleted and re-downloaded
    #[clap(
        long="force-update-data",
        env="NLR_FORCE_UPDATE_DATA",
        default_value_t = false,
    )]
    #[serde(default="default_NLR_FORCE_UPDATE_DATA")]
    pub NLR_FORCE_UPDATE_DATA: bool,

    /// Url of the esri rest service hosting the road network data
    #[clap(
        long="data-source-url",
        env="NLR_DATA_SOURCE_URL",
        default_value = "https://mrgis.mainroads.wa.gov.au/arcgis/rest/services/OpenData/RoadAssets_DataPortal/MapServer/17/query?where=1%3D1&outFields=ROAD,START_SLK,END_SLK,CWY&outSR=4326&f=json",
        value_hint=clap::ValueHint::Url
    )]
    pub NLR_DATA_SOURCE_URL: String,

}

#[allow(non_snake_case)]
fn default_NLR_FORCE_UPDATE_DATA()->bool{
    false
}
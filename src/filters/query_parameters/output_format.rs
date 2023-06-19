use serde::{Deserialize};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(non_camel_case_types)]
pub enum OutputFormatPoints {
    geojson,
    wkt,
    json,
    latlon,
    latlondir,
}

impl Default for OutputFormatPoints {
    fn default() -> Self {
        OutputFormatPoints::geojson
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[allow(non_camel_case_types)]
pub enum OutputFormatLines {
    geojson,
    wkt,
    json,
}

impl Default for OutputFormatLines {
    fn default() -> Self {
        OutputFormatLines::geojson
    }
}
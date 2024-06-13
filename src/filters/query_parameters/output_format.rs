use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Default, Copy)]
#[allow(non_camel_case_types)]
pub enum OutputFormatPoints {
    #[default]
    geojson,
    wkt,
    json,
    latlon,
    latlondir,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Default)]
#[allow(non_camel_case_types)]
pub enum OutputFormatLines {
    #[default]
    geojson,
    wkt,
    json,
}


#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Default)]
#[allow(non_camel_case_types)]
pub enum OutputFormatUnified {
    #[default]
    geojson,
    wkt,
    json,
}

impl From<OutputFormatUnified> for OutputFormatPoints{
    fn from(value: OutputFormatUnified) -> Self {
        match value {
            OutputFormatUnified::geojson => OutputFormatPoints::geojson,
            OutputFormatUnified::wkt => OutputFormatPoints::wkt,
            OutputFormatUnified::json => OutputFormatPoints::json,
        }
    }
}
impl From<OutputFormatUnified> for OutputFormatLines{
    fn from(value:OutputFormatUnified) -> Self {
        match value {
            OutputFormatUnified::geojson => OutputFormatLines::geojson,
            OutputFormatUnified::wkt => OutputFormatLines::wkt,
            OutputFormatUnified::json => OutputFormatLines::json,
        }
    }
}
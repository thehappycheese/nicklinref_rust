use nickslinetoolsrust::vector2::Vector2;
use serde::{Deserialize};

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum EsriCwy {
    Left,
    Right,
    Single,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct EsriAttributes {
    pub ROAD: String,
    pub CWY: EsriCwy,
    pub START_SLK: f32,
    pub END_SLK: f32,
}

#[derive(Deserialize, Debug)]
pub struct EsriPolylineGeometry {
    pub paths: [Vec<Vector2>; 1],
    // #[serde(default)]
    // pub hasZ:bool;
    // #[serde(default)]
    // pub hasZ:bool;
    // /// https://developers.arcgis.com/documentation/common-data-types/geometry-objects.htm#GUID-DFF0E738-5A42-40BC-A811-ACCB5814BABC
    // pub spatialReference: ??? 
}

#[derive(Debug, Deserialize)]
pub struct EsriFeature {
    pub geometry: EsriPolylineGeometry,
    pub attributes: EsriAttributes,
}

#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug)]
/// used to restrict parsing to succeed only when receiving the expected geometry type
pub enum EsriGeometryType {
    //esriGeometryPoint,
    //esriGeometryMultipoint,
    esriGeometryPolyline,
    // esriGeometryPolygon,
    // esriGeometryEnvelope,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
/// Supports a narrow subset of EsriJSON;
/// when `"geometryType":"esriGeometryPolyline"`
/// and when `"exceededTransferLimit":` is present.
/// 
/// See reference
/// <https://developers.arcgis.com/documentation/common-data-types/featureset-object.htm>
pub struct EsriFeatureSet {
    pub geometryType: EsriGeometryType,
    pub features: Vec<EsriFeature>,
    pub exceededTransferLimit: Option<bool>,

    // #[serde(default)]
    // pub hasZ:bool;

    // #[serde(default)]
    // pub hasZ:bool;

    // pub fields: ???

    // /// if not present, must assume the spatialReference of first feature.
    // /// If not set on first feature then it is UnknownCoordinateSystem.
    // /// https://developers.arcgis.com/documentation/common-data-types/geometry-objects.htm#GUID-DFF0E738-5A42-40BC-A811-ACCB5814BABC
    // pub spatialReference: ??? 
}
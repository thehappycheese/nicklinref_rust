use serde::{Serialize, Deserialize};
use nickslinetoolsrust::vector2::Vector2;
use super::{
    super::esri_json,
    Attributes
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Feature {
    pub attributes: Attributes,
    pub geometry: Vec<Vector2>,
}


impl From<esri_json::EsriFeature> for Feature {
    fn from(item: esri_json::EsriFeature) -> Feature {
        Feature {
            attributes: item.attributes.into(),
            geometry: item.geometry.paths[0].clone(),
        }
    }
}

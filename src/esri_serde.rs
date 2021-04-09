//use serde_json::Value as JsonValue;
use nickslinetoolsrust::vector2::Vector2;
use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Cwy {
	Left,
	Right,
	Single,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct LayerFeatureAttr {
	ROAD: String,
	START_SLK: f32,
	END_SLK: f32,
	CWY: Cwy,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FeatureGeom {
	paths: [Vec<Vector2>; 1],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LayerFeature {
	attributes: LayerFeatureAttr,
	geometry: FeatureGeom,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
/// This is the struct/document that will be deserialised from what is recieved over the interwebs.
pub struct LayerDownloadChunk {
	#[serde(default)]
	pub exceededTransferLimit: bool,
	pub geometryType: String,
	pub features: Vec<LayerFeature>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LayerSavedFeature {
	attributes: LayerFeatureAttr,
	geometry: Vec<Vector2>,
}

#[derive(Serialize, Deserialize, Debug)]
/// This is the struct/document that will be saved to the harddrive in some format or another.
pub struct LayerSaved {
	pub features: Vec<LayerSavedFeature>,
}

impl From<&LayerFeature> for LayerSavedFeature{
	fn from(item:&LayerFeature) -> LayerSavedFeature{
		LayerSavedFeature{
			attributes:item.attributes.clone(),
			geometry:item.geometry.paths[0].clone()
		}
	}
}
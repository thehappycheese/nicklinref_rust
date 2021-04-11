//use serde_json::Value as JsonValue;
use nickslinetoolsrust::vector2::Vector2;
use serde::{Deserialize, Serialize};


#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum Cwy {
	Left,
	Right,
	Single,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, PartialOrd)]
#[allow(non_snake_case)]
pub struct LayerFeatureAttr {
	pub ROAD: String,
	pub CWY: Cwy,
	pub START_SLK: f32,
	pub END_SLK: f32,
}

impl Eq for LayerFeatureAttr{}

impl Ord for LayerFeatureAttr {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering{
		match self.ROAD.cmp(&other.ROAD) {
			std::cmp::Ordering::Less=>std::cmp::Ordering::Less,
			std::cmp::Ordering::Equal=>{
				match self.CWY.cmp(&other.CWY){
					std::cmp::Ordering::Less=>std::cmp::Ordering::Less,
					std::cmp::Ordering::Equal=>{
						if self.START_SLK.is_nan() || other.START_SLK.is_nan(){
							std::cmp::Ordering::Equal
						}else if self.START_SLK == other.START_SLK{
							std::cmp::Ordering::Equal
						}else if self.START_SLK < other.START_SLK{
							std::cmp::Ordering::Less
						}else{
							std::cmp::Ordering::Greater
						}
					},
					std::cmp::Ordering::Greater=>std::cmp::Ordering::Greater
				}
			},
			std::cmp::Ordering::Greater=>std::cmp::Ordering::Greater
		}
	}
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FeatureGeom {
	paths: [Vec<Vector2>; 1],
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LayerFeature {
	pub attributes: LayerFeatureAttr,
	pub geometry: FeatureGeom,
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
	pub attributes: LayerFeatureAttr,
	pub geometry: Vec<Vector2>,
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
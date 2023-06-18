use serde::{Deserialize, Serialize};
use super::{
    Cwy,
    super::esri_json
};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, PartialOrd)]
#[allow(non_snake_case)]
pub struct Attributes {
    pub ROAD: String,
    pub CWY: Cwy,
    pub START_SLK: f32,
    pub END_SLK: f32,
}

impl Eq for Attributes {}

impl Ord for Attributes {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.ROAD.cmp(&other.ROAD) {
            std::cmp::Ordering::Less => std::cmp::Ordering::Less,
            std::cmp::Ordering::Equal => self.CWY.cmp(&other.CWY),
            std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
        }
    }
}

impl From<esri_json::EsriAttributes> for Attributes {
    fn from(value: esri_json::EsriAttributes) -> Self {
        Self {
            ROAD: value.ROAD.clone(),
            CWY: value.CWY.into(),
            START_SLK: value.START_SLK,
            END_SLK: value.END_SLK,
        }
    }
}

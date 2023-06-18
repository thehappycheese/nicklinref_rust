use super::super::esri_json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum Cwy {
    Left,
    Right,
    Single,
}

impl From<esri_json::EsriCwy> for Cwy {
    fn from(value: esri_json::EsriCwy) -> Self {
        match value {
            esri_json::EsriCwy::Left => Self::Left,
            esri_json::EsriCwy::Right => Self::Right,
            esri_json::EsriCwy::Single => Self::Single,
        }
    }
}

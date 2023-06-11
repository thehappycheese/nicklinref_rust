use super::RequestedCwy;
use super::OutputFormat;

use serde;

#[derive(serde::Deserialize, Debug)]
pub struct QueryParametersPoint {
	pub road: String,

	#[serde(default = "default_cwy")]
	pub cwy: RequestedCwy,

	pub slk: f32,

	#[serde(default = "default_offset")]
	pub offset:f32,

	#[serde(default = "default_output_format")]
	pub f: OutputFormat,

}

fn default_offset() -> f32 {
	0.0f32
}

fn default_cwy() -> RequestedCwy {
	RequestedCwy::LRS
}

fn default_output_format() -> OutputFormat {
	OutputFormat::GEOJSON
}
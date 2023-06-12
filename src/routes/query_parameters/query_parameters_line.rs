use super::RequestedCwy;
use super::OutputFormat;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct QueryParametersLine {
    /// Road number
	pub road: String,

    
    /// the starting SLK offset to slice the road network
	pub slk_from: f32,
    
    /// the ending SLK offset to slice the road network
	pub slk_to: f32,

	#[serde(default)] // use Default trait: LRS
    /// The carriageway filter; all carriageways are included in the result by
    /// default
	pub cwy: RequestedCwy,

	#[serde(default)] // default 0
    /// The number of metres to offset the point or linestring from the road
    /// centreline. If facing the direction of increasing SLK, negative values
    /// will offset to the left, and positive values to the right.
	pub offset:f32,

	#[serde(default)] // use Default trait: GEOJSON
    /// The output data format to be returned by the server
	pub f: OutputFormat,

	#[serde(default)] // default false
    /// request that the linear referencing M coordinate should be included if
    /// possible
	pub m:bool,

}
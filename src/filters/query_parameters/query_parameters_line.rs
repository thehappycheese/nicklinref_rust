use serde::Deserialize;

use crate::helpers::serde_helpers::{f32_finite_or_zero, f32_not_nan_or_fail};

use super::RequestedCwy;
use super::output_format::OutputFormatLines;

#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct QueryParametersLine {
    /// Road number
	pub road: String,

    #[serde(default="default_slk_from", deserialize_with = "f32_not_nan_or_fail")]
    /// the starting SLK offset to slice the road network
	pub slk_from: f32,
    
    #[serde(default="default_slk_to", deserialize_with = "f32_not_nan_or_fail")]
    /// the ending SLK offset to slice the road network
	pub slk_to: f32,

	#[serde(default)] // default LRS
    /// The carriageway filter; all carriageways are included in the result by
    /// default
	pub cwy: RequestedCwy,

	#[serde(default, deserialize_with = "f32_finite_or_zero")] // default 0
    /// The number of metres to offset the point or linestring from the road
    /// centreline. If facing the direction of increasing SLK, negative values
    /// will offset to the left, and positive values to the right.
	pub offset:f32,

	#[serde(default)] // default GEOJSON
    /// The output data format to be returned by the server
	pub f: OutputFormatLines,

	#[serde(default)] // default false
    /// request that the linear referencing M coordinate should be included if
    /// possible
	pub m:bool,

}

impl QueryParametersLine {
    pub fn with_format(&self, format:&OutputFormatLines) -> Self{
        QueryParametersLine{
            f:format.clone(),
            ..self.clone()
        }
    }
}

fn default_slk_from() -> f32 {
    f32::NEG_INFINITY
}

fn default_slk_to() -> f32 {
    f32::INFINITY
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIRMED_WORKING_MINIMUM: &str = "road=H001&slk_from=5&slk_to=6";

    #[test]
    /// Test when minimum parameters are provided
    fn test_query_parameters_line_defaults() {
        let query = CONFIRMED_WORKING_MINIMUM;
        let query: QueryParametersLine = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(query, QueryParametersLine{
            road: "H001".to_string(),
            slk_from: 5.0,
            slk_to: 6.0,
            offset: 0.0,
            cwy: RequestedCwy::LRS,
            f: OutputFormatLines::geojson,
            m: false,
        });
    }

    #[test]
    /// Test new behaviour where slk_from or slk_to can be omitted to get the
    /// entire road
    fn test_query_parameters_line_defaults_road_only() {
        let query = "road=H001";
        let query: QueryParametersLine = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(query, QueryParametersLine{
            road: "H001".to_string(),
            slk_from: f32::NEG_INFINITY,
            slk_to: f32::INFINITY,
            offset: 0.0,
            cwy: RequestedCwy::LRS,
            f: OutputFormatLines::geojson,
            m: false,
        });
    }

    #[test]
    /// Test when all parameters are provided.
    fn test_query_parameters_line_all() {
        let query = format!("{}&cwy=LS&offset=10&f=wkt&m=true", CONFIRMED_WORKING_MINIMUM);
        let query: QueryParametersLine = serde_urlencoded::from_str(&query).unwrap();
        assert_eq!(query, QueryParametersLine{
            road: "H001".to_string(),
            slk_from: 5.0,
            slk_to: 6.0,
            cwy: RequestedCwy::LS,
            offset: 10.0,
            f: OutputFormatLines::wkt,
            m:true
        });
    }

    #[test]
    /// Test nan offset should replace with default of zero
    fn test_query_parameters_line_offset_nan() {
        let query = format!("{}&offset=nan", CONFIRMED_WORKING_MINIMUM);
        let query: QueryParametersLine = serde_urlencoded::from_str(&query).unwrap();
        assert_eq!(query, QueryParametersLine{
            road: "H001".to_string(),
            slk_from: 5.0,
            slk_to: 6.0,
            cwy: RequestedCwy::LRS,
            offset: 0.0,
            f: OutputFormatLines::geojson,
            m: false,
        });
    }

    #[test]
    /// Test infinite offset should fail to parse and reject the query
    fn test_query_parameters_line_offset_infinity_fails() {
        let query = format!("{}&offset=Infinity", CONFIRMED_WORKING_MINIMUM);
        let query: Result<QueryParametersLine,_> = serde_urlencoded::from_str(&query);
        assert!(query.is_err());
    }

    #[test]
    /// Negative SLK values are allowed to parse but obviously will not produce
    /// valid output. This is to support the very remote possibility that
    /// negative SLK values may one day be used. (I'd have preferred this to the
    /// reset they recently did with Tonkin Highway's SLKs for example.)
    fn test_query_parameters_line_prevent_slk_negative() {
        let query = "road=H001&slk_from=-5&slk_to=-1";
        let query:QueryParametersLine = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(query.slk_from, -5.0);
        assert_eq!(query.slk_to, -1.0);
    }

    #[test]
    /// SLK infinite slk bounds are allowed because they are potentially useful
    /// to get the entire road without knowing the start and end slks
    /// TODO: Add a test to see if they actually function as expected
    fn test_query_parameters_line_prevent_slk_infinity() {
        let query = "road=H001&slk_from=-Infinity&slk_to=Infinity";
        let query:QueryParametersLine = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(query.slk_from, f32::NEG_INFINITY);
        assert_eq!(query.slk_to, f32::INFINITY);
    }

    #[test]
    /// SLK NaN should fail to parse and reject the query
    fn test_query_parameters_line_slk_nan() {
        let query = "road=H001&slk_from=nan&slk_to=nan";
        let query:Result<QueryParametersLine, _> = serde_urlencoded::from_str(query);
        assert!(query.is_err());

        let query = "road=H001&slk_from=1&slk_to=nan";
        let query:Result<QueryParametersLine, _> = serde_urlencoded::from_str(query);
        assert!(query.is_err());

        let query = "road=H001&slk_from=nan&slk_to=1";
        let query:Result<QueryParametersLine, _> = serde_urlencoded::from_str(query);
        assert!(query.is_err());
    }

    #[test]
    /// don't allow unknown fields (eg misspelling `cwy=` as `cway=`)
    fn test_query_parameters_line_disallow_unknown_fields() {
        let query = "road=H001&cway=L";
        let query:Result<QueryParametersLine, _> = serde_urlencoded::from_str(query);
        assert!(query.is_err());
    }

}
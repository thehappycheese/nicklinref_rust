use serde::Deserialize;

use crate::helpers::serde_helpers::{f32_finite_or_fail, f32_finite_or_zero};

use super::RequestedCwy;
use super::output_format::OutputFormatPoints;


#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct QueryParametersPoint {
    /// road number (eg "H001")
	pub road: String,

	#[serde(default)]
    /// Carriageway
	pub cwy: RequestedCwy,

    #[serde(deserialize_with = "f32_finite_or_fail")]
    /// Straight Line Kilometres (SLK) location on the road
	pub slk: f32,

	#[serde(default, deserialize_with = "f32_finite_or_zero")]
    /// offset in metres from the road centreline. See readme regarding
    /// offset direction
	pub offset:f32,

	#[serde(default)]
    /// format of the response
	pub f: OutputFormatPoints,

}

impl QueryParametersPoint {
    pub fn with_format(&self, format:&OutputFormatPoints) -> Self{
        QueryParametersPoint{
            f:format.clone(), // TODO: SHould not clone inside function i think???
            cwy:self.cwy,
            offset:self.offset,
            road:self.road.clone(),
            slk:self.slk
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    /// Test when minimum parameters are provided
    fn test_query_parameters_point_defaults() {
        let query = "road=H001&slk=5";
        let query: QueryParametersPoint = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(query, QueryParametersPoint{
            road: "H001".to_string(),
            cwy: RequestedCwy::LRS,
            slk: 5.0,
            offset: 0.0,
            f: OutputFormatPoints::geojson,
        });
    }

    #[test]
    /// Test when all parameters are provided
    fn test_query_parameters_point_all() {
        let query = "road=H001&slk=5&cwy=LS&offset=10&f=wkt";
        let query: QueryParametersPoint = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(query, QueryParametersPoint{
            road: "H001".to_string(),
            cwy: RequestedCwy::LS,
            slk: 5.0,
            offset: 10.0,
            f: OutputFormatPoints::wkt,
        });
    }

    #[test]
    /// Test nan offset should replace with default of zero
    fn test_query_parameters_point_offset_nan() {
        let query = "road=H001&slk=5&offset=nan";
        let query: QueryParametersPoint = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(query, QueryParametersPoint{
            road: "H001".to_string(),
            cwy: RequestedCwy::LRS,
            slk: 5.0,
            offset: 0.0,
            f: OutputFormatPoints::geojson,
        });
    }

    #[test]
    /// Test infinite offset should fail to parse and reject the query
    fn test_query_parameters_point_offset_infinity_fails() {
        let query = "road=H001&slk=5&offset=Infinity";
        let query: Result<QueryParametersPoint,_> = serde_urlencoded::from_str(query);
        assert!(query.is_err());
    }

    #[test]
    /// Negative SLK values are allowed to parse but obviously will not produce
    /// valid output. This is to support the very remote possibility that
    /// negative SLK values may one day be used. (I'd have preferred this to the
    /// reset they recently did with Tonkin Highway's SLKs for example.)
    fn test_query_parameters_point_prevent_slk_negative() {
        let query = "road=H001&slk=-5&offset=10";
        let query:QueryParametersPoint = serde_urlencoded::from_str(query).unwrap();
        assert_eq!(query.slk, -5.0);
    }

    #[test]
    /// SLK Infinity should fail to parse and reject the query
    fn test_query_parameters_point_prevent_slk_infinity() {
        let query = "road=H001&slk=Infinity&offset=10";
        let query:Result<QueryParametersPoint, _> = serde_urlencoded::from_str(query);
        assert!(query.is_err());
    }

    #[test]
    /// SLK NaN should fail to parse and reject the query
    fn test_query_parameters_point_slk_nan() {
        let query = "road=H001&slk=nan&offset=10";
        let query:Result<QueryParametersPoint, _> = serde_urlencoded::from_str(query);
        assert!(query.is_err());
    }

    #[test]
    /// m=true fails
    /// TODO: for json, geojson, and wkt, there is no reason this needs to fail
    ///       m=true is experimental and not yet implemented. Probably it is a
    ///       junk feature, i have never used it in practice. I should get rid of it.
    fn test_query_parameters_point_m_not_permitted() {
        let query = "road=H001&slk=5&m=true";
        let query:Result<QueryParametersPoint, _> = serde_urlencoded::from_str(query);
        assert!(query.is_err());
    }


    #[test]
    /// don't allow unknown fields (eg misspelling `cwy=` as `cway=`)
    fn test_query_parameters_point_deny_unknown_fields() {
        let query = "road=H001&slk=5&cway=L";
        let query:Result<QueryParametersPoint, _> = serde_urlencoded::from_str(query);
        assert!(query.is_err());
    }

}
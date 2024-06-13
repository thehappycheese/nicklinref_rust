use serde::Deserialize;

use super::{
    output_format::OutputFormatUnified,
    QueryParametersLine,
    QueryParametersPoint
};

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum QueryParametersPointLine{
    Point(QueryParametersPoint),
    Line(QueryParametersLine)
}



#[derive(Deserialize, Debug, PartialEq)]
pub struct QueryParametersUnifiedPost{
    pub format:OutputFormatUnified,
    pub items:Vec<QueryParametersPointLine>
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct QueryParametersUnifiedGet{
    pub format:OutputFormatUnified,
    pub items:String
}


#[cfg(test)]
mod tests {
    use crate::filters::query_parameters::{output_format::{OutputFormatLines, OutputFormatPoints}, RequestedCwy};

    use super::*;

    #[test]
    fn test_point_line_deserialization() {
        let example_json = r#"{
            "format":"geojson",
            "items":[
                {
                    "road":"H001",
                    "slk_from":10,
                    "slk_to":20,
                    "offset":10
                },
                {
                    "road":"H016",
                    "slk":10
                },
                {
                    "road":"H015",
                    "slk":10
                }
            ]
        }
        "#;

        let result: QueryParametersUnifiedPost = serde_json::from_str(example_json).unwrap();

        let expected = QueryParametersUnifiedPost{
            format:OutputFormatUnified::geojson,
            items:vec![
                QueryParametersPointLine::Line(QueryParametersLine {
                    road: String::from("H001"),
                    slk_from: 10.0,
                    slk_to: 20.0,
                    offset: 10.0,
                    f: OutputFormatLines::geojson,
                    cwy: RequestedCwy::LRS,
                    m: false
                }),
                QueryParametersPointLine::Point(QueryParametersPoint {
                    road: String::from("H016"),
                    slk: 10.0,
                    cwy: RequestedCwy::LRS,
                    offset:0.0,
                    f:OutputFormatPoints::geojson
                }),
                QueryParametersPointLine::Point(QueryParametersPoint {
                    road: String::from("H015"),
                    slk: 10.0,
                    cwy: RequestedCwy::LRS,
                    offset:0.0,
                    f:OutputFormatPoints::geojson
                }),
            ],
        };

        assert_eq!(result, expected);
    }
}
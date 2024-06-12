use serde::Deserialize;

use super::{
    QueryParametersLine,
    QueryParametersPoint
};

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum QueryParametersUnified{
    Point(QueryParametersPoint),
    Line(QueryParametersLine)
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn test_unified_batch(){
        let example_json = r#"[
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
        ]"#;
        let result:Vec<QueryParametersUnified> = serde_json::from_str(example_json).unwrap();
        println!("{:?}", result);
    }
}
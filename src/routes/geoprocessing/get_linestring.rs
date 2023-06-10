
use crate::data::IndexedData;
use crate::routes::query_parameters::{QueryParametersLine,OutputFormat};
use std::str;
use std::sync::Arc;
use crate::data::esri_serde::LayerSaved;
use crate::data::index::{LookupMap, RoadDataByCwy};
use nickslinetoolsrust::line_string_measured::{LineStringMeasured};
use crate::helpers::{convert_metres_to_degrees, ErrorWithStaticMessage};


pub fn get_linestring(query:&QueryParametersLine, indexed_data:&IndexedData)->Result<String, ErrorWithStaticMessage>{
	
    let road_data =  indexed_data.query(&query.road, &query.cwy)?;

    let features = road_data.filter_map(|item|{
			if item.attributes.END_SLK>query.slk_from && item.attributes.START_SLK<query.slk_to{

				let lsm:LineStringMeasured = LineStringMeasured::from(&item.geometry);
				
				let item_len_km = item.attributes.END_SLK - item.attributes.START_SLK;
				let frac_start = (query.slk_from-item.attributes.START_SLK) / item_len_km;
				let frac_end = (query.slk_to-item.attributes.START_SLK) / item_len_km;

				match lsm.cut_twice(frac_start.into(), frac_end.into()){
					(_, Some(b), _) => if query.offset == 0.0 {
								Some(b.into_tuples())
							}else{
								let degree_offset:f64 = -convert_metres_to_degrees(query.offset.into());
								match b.offset_basic(degree_offset){
									Some(item)=>{
										Some(item.iter().map(|ii|ii.into()).collect())
									},
									None=>None
								}
							},
					_=>None
				}
			}else{
				None
			}
		});

		match query.f{
			OutputFormat::JSON => {
				let line_string_string = features
					.map(|linestring|{
							"[".to_string() + &linestring.iter().filter_map(|vertex| serde_json::to_string(vertex).ok()).collect::<Vec<String>>().join(",") + "]"
					})
					.collect::<Vec<String>>()
					.join(",");
				Ok("[".to_string() + &line_string_string + "]")
			},
			OutputFormat::GEOJSON => {
				let line_string_string = features
					.map(|linestring|{
							"[".to_string() + &linestring.iter().filter_map(|vertex| serde_json::to_string(vertex).ok()).collect::<Vec<String>>().join(",") + "]"
					})
					.collect::<Vec<String>>()
					.join(",");
				Ok( r#"{"type":"Feature", "geometry":{"type":"MultiLineString", "coordinates":["#.to_string() + &line_string_string + "]}}")
			},
			OutputFormat::WKT => {
				let line_string_string = features
					.map(|linestring|{
							"(".to_string() + &linestring.iter().map(|vertex| format!("{} {}", vertex.0, vertex.1)).collect::<Vec<String>>().join(",") + ")"
					})
					.collect::<Vec<String>>()
					.join(",");
				Ok("MULTILINESTRING (".to_string() + &line_string_string + ")")
			},
			OutputFormat::LATLON=> Err(ErrorWithStaticMessage::new(
                "Invalid query type: f=LATLON can only be used with the point query type. Please use f=JSON, or specify slk instead of slk_from and slk_to."
            )),
			OutputFormat::LATLONDIR=>Err(ErrorWithStaticMessage::new(
                "Invalid query type: f=LATLONDIR can only be used with the point query type. Please use f=JSON, or specify slk instead of slk_from and slk_to."
            ))
		}
}
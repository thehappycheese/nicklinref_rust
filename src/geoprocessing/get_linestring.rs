
use crate::query_parameters::{QueryParametersLine,OutputFormat};
use std::str;
use std::sync::Arc;
use crate::esri_serde::LayerSaved;
use crate::update_data::{LookupMap, RoadDataByCwy};
use nickslinetoolsrust::line_string_measured::{LineStringMeasured};
use crate::unit_conversion::convert_metres_to_degrees;


pub fn get_linestring(query:&QueryParametersLine, data:&Arc<LayerSaved>, data_index:&Arc<LookupMap>)->Result<String, & 'static str>{
	let road_data:&RoadDataByCwy = match match query.road.chars().next(){
		Some(first_letter)=>{
			match data_index.get(&first_letter) {
				Some(mp1) => mp1.get(&query.road),
				None=>{return Err("Road Lookup Failed. First letter of road did not match any in lookup table.")}
			}
		},
		None=>{return Err("Road Lookup Failed. Could not get first letter of road.")}
	}{
		Some(data_lookup_sub_table)=>data_lookup_sub_table,
		None=>{return Err("Road Lookup Failed. Name not found in second level lookup table.")}
	};

	let features = query.cwy
		.into_iter()
		.filter_map(|cwy|{
			if let Some(indexes) = road_data[&cwy]{
				Some(&data.features[indexes.0..indexes.1])
			}else{
				None
			}
		})
		.flatten()
		.filter_map(|item|{
			if item.attributes.END_SLK>query.slk_from && item.attributes.START_SLK<query.slk_to{

				let lsm:LineStringMeasured = LineStringMeasured::from(&item.geometry);
				
				let item_len_km = item.attributes.END_SLK - item.attributes.START_SLK;
				let frac_start = (query.slk_from-item.attributes.START_SLK) / item_len_km;
				let frac_end = (query.slk_to-item.attributes.START_SLK) / item_len_km;

				match lsm.cut_twice(frac_start.into(), frac_end.into()){
					(_, Some(b), _) => if query.offset == 0.0 {
								Some(b.into_vector2())
							}else{
								let degree_offset:f64 = convert_metres_to_degrees(query.offset.into());
								b.offset_basic(-degree_offset)
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
							"(".to_string() + &linestring.iter().map(|vertex| format!("{} {}", vertex.x, vertex.y)).collect::<Vec<String>>().join(",") + ")"
					})
					.collect::<Vec<String>>()
					.join(",");
				Ok("MULTILINESTRING (".to_string() + &line_string_string + ")")
			},
			OutputFormat::LATLON=>{
				return Err("Invalid query type LATLON used with /lines/ query. Please use JSON instead.")
			}
		}
}
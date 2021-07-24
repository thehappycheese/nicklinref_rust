use crate::esri_serde::LayerSaved;
use crate::query_parameters::{OutputFormat, QueryParametersPoint};
use crate::update_data::{LookupMap, RoadDataByCwy};
use nickslinetoolsrust::line_string_measured::LineStringMeasured;
use nickslinetoolsrust::vector2::Vector2;
use std::str;
use std::sync::Arc;

pub fn get_points(
	query: &QueryParametersPoint,
	data: &Arc<LayerSaved>,
	data_index: &Arc<LookupMap>,
) -> Result<String, &'static str> {
	let road_data: &RoadDataByCwy = match match query.road.chars().next() {
		Some(first_letter) => match data_index.get(&first_letter) {
			Some(mp1) => mp1.get(&query.road),
			None => {
				return Err("road lookup failed, first letter did not match any lookup tables.")
			}
		},
		None => return Err("could not get first letter of road"),
	} {
		Some(data_lookup_sub_table) => data_lookup_sub_table,
		None => return Err("full road name not found. lookup failed"),
	};

	let features = query
		.cwy
		.into_iter()
		.filter_map(|cwy| {
			if let Some(indexes) = road_data[&cwy] {
				Some(&data.features[indexes.0..indexes.1])
			} else {
				None
			}
		})
		.flatten()
		.filter_map(|item| {
			if item.attributes.END_SLK > query.slk && item.attributes.START_SLK < query.slk {
				let lsm: LineStringMeasured = LineStringMeasured::from(&item.geometry);
				let item_len_km = item.attributes.END_SLK - item.attributes.START_SLK;
				let frac = (query.slk - item.attributes.START_SLK) / item_len_km;
				lsm.interpolate(frac as f64)
			} else {
				None
			}
		});

	match query.f {
		OutputFormat::JSON => {
			let points = features
				.filter_map(|vertex| serde_json::to_string(&vertex).ok())
				.collect::<Vec<String>>()
				.join(",");
			Ok("[".to_string() + &points + "]")
		}
		OutputFormat::GEOJSON => {
			let points = features
				.filter_map(|vertex| serde_json::to_string(&vertex).ok())
				.collect::<Vec<String>>()
				.join(",");
			Ok(
				r#"{"type":"Feature", "geometry":{"type":"MultiPoint", "coordinates":["#
					.to_string()
					+ &points
					+ "]}}",
			)
		}
		OutputFormat::WKT => {
			let points = features
				.map(|vertex| format!("({} {})", vertex.x, vertex.y))
				.collect::<Vec<String>>()
				.join(",");
			Ok("MULTIPOINT (".to_string() + &points + ")")
		}
		OutputFormat::LATLON => {
			let vertexes: Vec<Vector2> = features.collect();
			let point = vertexes
				.iter()
				.fold(Vector2::new(0f64, 0f64), |acc, el| acc + *el)
				/ (vertexes.len() as f64);
			Ok(format!("{},{}", point.y, point.x))
		}
	}
}

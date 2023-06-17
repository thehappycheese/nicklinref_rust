use crate::data::IndexedData;
use crate::filters::query_parameters::{QueryParametersPoint, output_format::OutputFormatPoints};
use nickslinetoolsrust::line_string_measured::LineStringMeasured;
use nickslinetoolsrust::vector2::Vector2;
use crate::helpers::{convert_metres_to_degrees, ErrorWithStaticMessage, mean_angle};

pub fn get_points(
	query: &QueryParametersPoint,
	indexed_data: &IndexedData,
) -> Result<String, ErrorWithStaticMessage> {
	let features = indexed_data.query(&query.road, &query.cwy)?
		.filter_map(|item| {
			if item.attributes.END_SLK >= query.slk && item.attributes.START_SLK <= query.slk {
				let lsm: LineStringMeasured = LineStringMeasured::from(&item.geometry);
				let item_len_km = item.attributes.END_SLK - item.attributes.START_SLK;
				let frac = (query.slk - item.attributes.START_SLK) / item_len_km;

				// support offset
				let lsmo:Option<LineStringMeasured> = if query.offset == 0.0 {
					Some(lsm)
				}else{
					let degree_offset:f64 = -convert_metres_to_degrees(query.offset.into());
					match lsm.offset_basic(degree_offset) {
						Some(vec_of_vector2)=> Some(LineStringMeasured::from(vec_of_vector2)),
						None=>None
					}
				};
				
				match lsmo {
					Some(lsm)=> match lsm.interpolate(frac as f64) {
						Some(vec)=>Some((vec, lsm.direction(frac as f64))),
						None=>None
					},
					_=>None
				}

			} else {
				None
			}
		});

	match query.f {
		OutputFormatPoints::json => {
			let points = features
				.filter_map(|(vertex, _dir)| serde_json::to_string(&vertex).ok())
				.collect::<Vec<String>>()
				.join(",");
			if points.len()>0{
				Ok("[".to_string() + &points + "]")
			}else{
				Err(ErrorWithStaticMessage::new("Found no points"))
			}
		}
		OutputFormatPoints::geojson => {
			let points = features
				.filter_map(|(vertex, _dir)| serde_json::to_string(&vertex).ok())
				.collect::<Vec<String>>()
				.join(",");
			if points.len()>0{
				Ok(
					r#"{"type":"Feature", "geometry":{"type":"MultiPoint", "coordinates":["#
						.to_string()
						+ &points
						+ "]}}",
				)
			}else{
				Err(ErrorWithStaticMessage::new("Found no points"))
			}
		}
		OutputFormatPoints::wkt => {
			let points = features
				.map(|(vertex, _dir)| format!("({} {})", vertex.x, vertex.y))
				.collect::<Vec<String>>()
				.join(",");
			if points.len()>0{
				Ok("MULTIPOINT (".to_string() + &points + ")")
			}else{
				Err(ErrorWithStaticMessage::new("Found no points"))
			}
		}
		OutputFormatPoints::latlon => {
			let vertexes: Vec<(Vector2,f64)> = features.collect();
			if vertexes.len()>0{
				let point = vertexes
					.iter()
					.fold(Vector2::new(0f64, 0f64), |acc, (el, _dir)| acc + *el)
					/ (vertexes.len() as f64);
				Ok(format!("{},{}", point.y, point.x))
			}else{
				Err(ErrorWithStaticMessage::new("Found no points"))
			}
		},
		OutputFormatPoints::latlondir => {
			let vertexes: Vec<(Vector2,f64)> = features.collect();
			if vertexes.len()>0{
				let point = vertexes
					.iter()
					.fold(Vector2::new(0f64, 0f64), |acc, (el, _dir)| acc + *el)
					/ (vertexes.len() as f64);
				let angle = mean_angle(vertexes.iter().map(|item|item.1).collect());
				Ok(format!("{},{},{}", point.y, point.x, angle.to_degrees()))
			}else{
				Err(ErrorWithStaticMessage::new("Found no points"))
			}
		}
	}
}

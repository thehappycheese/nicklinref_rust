use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::Index;
use std::sync::Arc;
use crate::helpers::ErrorWithStaticMessage;
use super::esri_serde::{Cwy, LayerSaved};

#[allow(non_snake_case)]
pub struct RoadDataByCwy {
	pub Left: Option<(usize, usize)>,
	pub Right: Option<(usize, usize)>,
	pub Single: Option<(usize, usize)>,
}
impl RoadDataByCwy {
	fn new(
		l: Option<(usize, usize)>,
		r: Option<(usize, usize)>,
		s: Option<(usize, usize)>,
	) -> Self {
		Self {
			Left: l,
			Right: r,
			Single: s,
		}
	}
	fn new_from_cwy(cwy: &Cwy, range: (usize, usize)) -> Self {
		match cwy {
			Cwy::Left => Self::new(Some(range), None, None),
			Cwy::Right => Self::new(None, Some(range), None),
			Cwy::Single => Self::new(None, None, Some(range)),
		}
	}
	fn with_updated_cwy(&self, cwy: &Cwy, range: (usize, usize)) -> Self {
		match cwy {
			Cwy::Left => Self::new(Some(range), self.Right, self.Single),
			Cwy::Right => Self::new(self.Left, Some(range), self.Single),
			Cwy::Single => Self::new(self.Left, self.Right, Some(range)),
		}
	}
}
impl Index<&Cwy> for RoadDataByCwy {
	type Output = Option<(usize, usize)>;
	fn index(&self, index: &Cwy) -> &Self::Output {
		match index {
			Cwy::Left => &self.Left,
			Cwy::Right => &self.Right,
			Cwy::Single => &self.Single,
		}
	}
}

// 'X' has 2
// '0' has 1136
// '1' has 35330
// '2' has 10328
// '3' has 4285
// '4' has 5017
// '5' has 4489
// '6' has 2511
// '7' has 527
// '8' has 2280
// 'P' has 317
// 'H' has 567
// 'M' has 75

pub type LookupMap = HashMap<char, HashMap<String, RoadDataByCwy>>;

/// TODO: this function builds a LookupMap, but it is based on some
///       potentially incorrect assumptions about hash table performance.
///       In practice it is working just fine, but there is probably a
///       simpler way to do this. Nested Hash tables probably don't perform any 
///       better than a flat one at lookup-time.

pub fn index_data(layer: Arc<LayerSaved>) -> Result<LookupMap, Box<dyn std::error::Error>> {
	let mut map_from_first_letter: LookupMap = HashMap::new(); // map_from_first_letter_to_roads

	let (mut previous_road, mut previous_cwy, mut first_letter) = match layer.features.iter().next()
	{
		Some(first_feature) => (
			&first_feature.attributes.ROAD,
			&first_feature.attributes.CWY,
			match layer.features[0].attributes.ROAD.chars().next() {
				Some(fl) => fl,
				None => ' ',
			},
		),
		None => {
			return Err(Box::new(ErrorWithStaticMessage::new(
				"Zero features received by perform_analysis()",
			)))
		}
	};

	let mut current_slice_start = 0; // inclusive of that index
	let mut map_from_road_number = map_from_first_letter.entry(first_letter).or_default();
	let mut i: usize = 1;

	while i < layer.features.len() {
		let feature = &layer.features[i];

		let current_feature_is_new_road = previous_road != &feature.attributes.ROAD;
		let current_feature_is_different_cwy = previous_cwy != &feature.attributes.CWY;
		if current_feature_is_new_road || current_feature_is_different_cwy {
			// the into() function on the next line is doing magic that I don't quite understand. Maybe its better than my previous solution which was .clone() ?
			match map_from_road_number.entry(previous_road.into()) {
				Entry::Vacant(e) => {
					e.insert(RoadDataByCwy::new_from_cwy(
						previous_cwy,
						(current_slice_start, i),
					));
				}
				Entry::Occupied(mut e) => {
					e.insert(
						e.get().with_updated_cwy(previous_cwy, (current_slice_start, i)),
					);
				}
			}
			current_slice_start = i;
		}

		if current_feature_is_new_road {
			let new_first_letter = match feature.attributes.ROAD.chars().next() {
				Some(fl) => fl,
				None => ' ',
			};
			if new_first_letter != first_letter {
				first_letter = new_first_letter;
				map_from_road_number = map_from_first_letter.entry(first_letter).or_default();
			}
		}
		previous_road = &feature.attributes.ROAD;
		previous_cwy = &feature.attributes.CWY;
		i += 1;
	}

	// println!(">>>Analysis Completed");
	// println!(
	// 	"Result top level has {} keys",
	// 	map_from_first_letter.keys().len()
	// );
	// for key in map_from_first_letter.keys() {
	// 	println!(
	// 		"Key: '{}' has {} sub-keys",
	// 		key,
	// 		map_from_first_letter[key].keys().len()
	// 	);
	// }
	// for key in map_from_first_letter[&'H'].keys(){
	// 	let item = &map_from_first_letter[&'H'][key];
	// 	println!("Key exists on 'H': \"{}\" having the following number of features in L:{} R:{} S:{}", key,
	// 	match item.Left {
	// 		Some(v)=>v.1-v.0,
	// 		None=>0
	// 	},
	// 	match item.Right {
	// 		Some(v)=>v.1-v.0,
	// 		None=>0
	// 	},
	// 	match item.Single {
	// 		Some(v)=>v.1-v.0,
	// 		None=>0
	// 	});
	// }

	Ok(map_from_first_letter)
}
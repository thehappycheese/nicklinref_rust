use std::error::Error;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::{
    helpers::ErrorWithStaticMessage,
    filters::query_parameters::RequestedCwy
};

use super::{
    RoadDataByCwy,
    super::cached::{
        Layer,
        Feature,
    }
};


pub type LookupMap = HashMap<char, HashMap<String, RoadDataByCwy>>;

pub struct IndexedData {
    pub(super) data:Layer,
    pub(super) index:LookupMap
}

impl IndexedData {

    /// Load existing data from the cache file path, or try to download data 
    /// and save it to the cache file path
    pub async fn load(
        path_to_data_cache_file:&String,
        url_to_download_new_data:&String,
        force_update:&bool,
    ) -> Result<Self, Box<dyn Error>>{
        let data = Layer::read_or_update_cache_data(path_to_data_cache_file,url_to_download_new_data,force_update).await?;
        let index = Self::index_data(&data)?;
        return Ok(Self{
            data,
            index
        })
    }


    pub fn query(&self, road_name:&String, cwy:&RequestedCwy) -> Result<impl Iterator<Item = &Feature>, ErrorWithStaticMessage> {
        let road_data_by_cwy = self.get_road_by_cwy(road_name)?;
        let feature_iterator = cwy
            .into_iter()
            .filter_map(|cwy|{
                if let Some(indexes) = road_data_by_cwy[&cwy]{
                    Some(&self.data.features[indexes.0..indexes.1])
                }else{
                    None
                }
            })
            .flatten();
        Ok(feature_iterator)
    }

    fn get_road_by_cwy(&self, road_name:&String) -> Result<&RoadDataByCwy, ErrorWithStaticMessage> {
        
        // try to get the first letter of the road name
        let first_letter = road_name.chars().next().ok_or(
            ErrorWithStaticMessage::new("Road Lookup Failed. First letter of 'road' did not match any in lookup table.")
        )?;
        
        // lookup the list of roads that start with that letter
        let roads_with_first_letter = self.index.get(&first_letter).ok_or(
            ErrorWithStaticMessage::new("Road Lookup Failed. Could not get first letter of road.")
        )?;

        // find the matching road
        roads_with_first_letter.get(road_name).ok_or(
            ErrorWithStaticMessage::new("Road Lookup Failed. 'road' not found in second level lookup table.")
        )
    }

    /// TODO: this function builds a LookupMap, but it is based on some
    ///       potentially incorrect assumptions about hash table performance.
    ///       In practice it is working just fine, but there is probably a
    ///       simpler way to do this. Nested Hash tables probably don't perform any 
    ///       better than a flat one.
    pub(super) fn index_data(layer: &Layer) -> Result<LookupMap, Box<dyn std::error::Error>> {
        println!("INFO: Indexing data");
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

        Ok(map_from_first_letter)
    }
}

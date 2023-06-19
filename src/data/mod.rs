
/// This module contains a deserializer for a subset of the EsriJSON format.
mod esri_json;

/// This is the data we keep in memory and save/load from disk
pub mod cached;

/// This is a wrapper around the cached data which lets us query it really fast.
mod indexed;
pub use indexed::{
    IndexedData
};
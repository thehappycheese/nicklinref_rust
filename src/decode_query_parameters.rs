use crate::esri_serde::Cwy;
use serde;
use serde::de::{Deserialize, Deserializer, Visitor};

use std::str::{from_utf8};
use std::fmt;
use std::convert::TryFrom;
use std::iter::IntoIterator;

#[derive(Debug)]
pub enum OutputFormat {
	GEOJSON,
	WKT,
	JSON,
}

impl<'de> Deserialize<'de> for OutputFormat {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct VariantVisitor;
		impl<'de> Visitor<'de> for VariantVisitor {
			type Value = OutputFormat;
			// Format a message stating what data this Visitor expects to receive.
			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("expects to recive any of the following values l, r, s, lr, ls, rs, lrs (or any capitalisation thereof)")
			}
			fn visit_borrowed_str<E>(self, s: &'de str) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				let chars = s.to_uppercase().chars().collect::<String>();
				Ok(match chars.as_str() {
					"GEOJSON" => OutputFormat::GEOJSON,
					"WKT" => OutputFormat::WKT,
					"JSON" => OutputFormat::JSON,
					_ => OutputFormat::GEOJSON,
				})
			}
		}
		deserializer.deserialize_string(VariantVisitor)
	}
}

#[derive(Debug, PartialEq)]
pub enum RequestedCwy {
	L,
	R,
	S,
	LR,
	LS,
	RS,
	LRS,
}
impl IntoIterator for &RequestedCwy{
	type Item = Cwy;
	type IntoIter = std::vec::IntoIter<Self::Item>;
	fn into_iter(self)->Self::IntoIter{
		match self {
			RequestedCwy::L => vec![Cwy::Left].into_iter(),
			RequestedCwy::R => vec![Cwy::Right].into_iter(),
			RequestedCwy::S => vec![Cwy::Single].into_iter(),
			RequestedCwy::LR => vec![Cwy::Left, Cwy::Right].into_iter(),
			RequestedCwy::LS => vec![Cwy::Left, Cwy::Single].into_iter(),
			RequestedCwy::RS => vec![Cwy::Right, Cwy::Single].into_iter(),
			RequestedCwy::LRS => vec![Cwy::Left, Cwy::Right, Cwy::Single].into_iter(),
		}
	}
}
// impl Iterator for &RequestedCwy{
// 	type Item = Cwy;
// 	fn next(self)->Option<Item>{
// 		&self.into_iter()
// 	}
// }

impl PartialEq<Cwy> for RequestedCwy {
	fn eq(&self, other: &Cwy) -> bool {
		match self {
			RequestedCwy::L => other == &Cwy::Left,
			RequestedCwy::R => other == &Cwy::Right,
			RequestedCwy::S => other == &Cwy::Single,
			RequestedCwy::LR => other == &Cwy::Left || other == &Cwy::Right,
			RequestedCwy::LS => other == &Cwy::Left || other == &Cwy::Single,
			RequestedCwy::RS => other == &Cwy::Right || other == &Cwy::Single,
			RequestedCwy::LRS => true,
		}
	}
}

impl<'de> Deserialize<'de> for RequestedCwy {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct VariantVisitor;
		impl<'de> Visitor<'de> for VariantVisitor {
			type Value = RequestedCwy;
			// Format a message stating what data this Visitor expects to receive.
			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("expects to recive any of the following values l, r, s, lr, ls, rs, lrs (or any capitalisation thereof)")
			}
			fn visit_borrowed_str<E>(self, s: &'de str) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				let mut chars: Vec<char> = s.to_uppercase().chars().collect::<Vec<char>>();
				chars.sort();
				Ok(match &chars.into_iter().collect::<String>()[..] {
					"L" => RequestedCwy::L,
					"R" => RequestedCwy::R,
					"S" => RequestedCwy::S,
					"LR" => RequestedCwy::LR,
					"LS" => RequestedCwy::LS,
					"RS" => RequestedCwy::RS,
					"LRS" => RequestedCwy::LRS,
					_ => RequestedCwy::LRS,
				})
			}
		}
		//const VARIANTS: &'static [&'static str] = &["L", "R", "S", "LR", "LS", "RS", "LRS"];
		//deserializer.deserialize_enum("RequestedCwy", VARIANTS, VariantVisitor)
		deserializer.deserialize_string(VariantVisitor)
	}
}

#[derive(serde::Deserialize, Debug)]
pub struct QueryParameters {
	pub road: String,

	#[serde(default = "default_cwy")]
	pub cwy: RequestedCwy,

	pub slk_from: f32,
	pub slk_to: f32,

	#[serde(default = "default_offset")]
	pub offset:f64,

	#[serde(default = "default_output_format")]
	pub f: OutputFormat,
}

fn default_offset() -> f64 {
	0.0f64
}

fn default_cwy() -> RequestedCwy {
	RequestedCwy::LRS
}

fn default_output_format() -> OutputFormat {
	OutputFormat::GEOJSON
}

struct QueryParameterBatch(Vec<QueryParameters>);

#[derive(Debug)]
struct BatchQueryParametersDecodeError;
impl std::error::Error for BatchQueryParametersDecodeError{}
impl std::fmt::Display for BatchQueryParametersDecodeError{
	fn fmt(&self, f:&mut fmt::Formatter) -> std::fmt::Result{
		write!(f, "{:?}", self)
	}
}

impl TryFrom<bytes::Bytes> for QueryParameterBatch{
	type Error = Box<dyn std::error::Error>;
	fn try_from(body:bytes::Bytes) -> Result<QueryParameterBatch, Self::Error> {
		let params:Vec<QueryParameters> = vec![];

		let mut offset:usize = 0;
		while offset < body.len(){
			
			let road_name_length:usize = *body.get(offset).ok_or(BatchQueryParametersDecodeError)? as usize;

			let current_query_length = 1+road_name_length+4+4+4+1;
			offset += 1;
			let road_name_bytes = body.get(offset..offset + road_name_length).ok_or(BatchQueryParametersDecodeError)?;
			offset += road_name_length;
			let road_name = from_utf8(road_name_bytes).ok_or(BatchQueryParametersDecodeError)?;
			let road_name = body.get(offset..offset+road_name_length).ok_or(BatchQueryParametersDecodeError)?;
		}
		QueryParameterBatch(params)
	}
}
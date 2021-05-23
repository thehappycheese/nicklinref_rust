use crate::esri_serde::Cwy;
use serde;
use serde::de::{Deserialize, Deserializer, Visitor};

//use std::str::Str
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

impl From<u8> for RequestedCwy{
	fn from(item:u8)->Self{
		match item{
			0b0000_0100=>RequestedCwy::L,
			0b0000_0001=>RequestedCwy::R,
			0b0000_0010=>RequestedCwy::S,
			0b0000_0101=>RequestedCwy::LR,
			0b0000_0110=>RequestedCwy::LS,
			0b0000_0011=>RequestedCwy::RS,
			0b0000_0111=>RequestedCwy::LRS,
			_=>RequestedCwy::LRS
		}
	}
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
	pub offset:f32,

	#[serde(default = "default_output_format")]
	pub f: OutputFormat,
}

fn default_offset() -> f32 {
	0.0f32
}

fn default_cwy() -> RequestedCwy {
	RequestedCwy::LRS
}

fn default_output_format() -> OutputFormat {
	OutputFormat::GEOJSON
}

pub struct QueryParameterBatch(pub Vec<QueryParameters>);

#[derive(Debug)]
pub struct BatchQueryParametersDecodeError;
impl std::error::Error for BatchQueryParametersDecodeError{}
impl std::fmt::Display for BatchQueryParametersDecodeError{
	fn fmt(&self, f:&mut fmt::Formatter) -> std::fmt::Result{
		write!(f, "{:?}", self)
	}
}

impl TryFrom<bytes::Bytes> for QueryParameterBatch{
	type Error = BatchQueryParametersDecodeError; //Box<dyn std::error::Error>;
	fn try_from(buffer:bytes::Bytes) -> Result<QueryParameterBatch, Self::Error> {
		
		let mut params:Vec<QueryParameters> = vec![];
		

		let mut buffer_iter = buffer.iter();

		loop {
			let road_name_byte_length = match buffer_iter.next(){
				Some(&num) => num as usize,
				None => break
			};
			let road_name:Vec<u8> = buffer_iter.by_ref().take(road_name_byte_length).map(|&x|x).collect();
			
			let road_name = std::str::from_utf8(&road_name[..]).or(Err(BatchQueryParametersDecodeError))?;

			let other_bytes = buffer_iter.by_ref().take(13).map(|&x|x).collect::<Vec<u8>>();
			if other_bytes.len()!=13{
				return Err(BatchQueryParametersDecodeError)
			}
			let slk_from = f32::from_le_bytes([other_bytes[0],other_bytes[1],other_bytes[2],other_bytes[3]]); // floats come packed in french bytes apparently ;)
			let slk_to = f32::from_le_bytes([other_bytes[4],other_bytes[5],other_bytes[6],other_bytes[7]]);
			let offset = f32::from_le_bytes([other_bytes[8],other_bytes[9],other_bytes[10],other_bytes[11]]);
			let cwy:RequestedCwy = other_bytes[12].into();
			params.push(QueryParameters{
				road:road_name.to_string(),
				slk_from,
				slk_to,
				offset,
				cwy,
				f:OutputFormat::JSON
			})
		}
		Ok(QueryParameterBatch(params))
	}
}
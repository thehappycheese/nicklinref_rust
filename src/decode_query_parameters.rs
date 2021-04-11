use serde;
use std::fmt;
use serde::de::{Deserialize, Deserializer, Visitor};
use crate::esri_serde::Cwy;

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

impl PartialEq<Cwy> for RequestedCwy{
	fn eq(&self, other:&Cwy)->bool{
		match self{
			RequestedCwy::L		=>other == &Cwy::Left,
			RequestedCwy::R		=>other == &Cwy::Right,
			RequestedCwy::S		=>other == &Cwy::Single,
			RequestedCwy::LR	=>other == &Cwy::Left	|| other == &Cwy::Right,
			RequestedCwy::LS	=>other == &Cwy::Left	|| other == &Cwy::Single,
			RequestedCwy::RS	=>other == &Cwy::Right	|| other == &Cwy::Single,
			RequestedCwy::LRS	=>true,
		}
	}
}

impl<'de> Deserialize<'de> for RequestedCwy {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct VariantVisitor;
		impl<'de> Visitor<'de> for VariantVisitor{
			type Value = RequestedCwy;
			// Format a message stating what data this Visitor expects to receive.
			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("expects to recive any of the following values l, r, s, lr, ls, rs, lrs (or any capitalisation thereof)")
			}
			fn visit_borrowed_str<E>(self, s:&'de str)->Result<Self::Value, E> where E:serde::de::Error {
				let mut chars:Vec<char> = s.to_uppercase().chars().collect::<Vec<char>>();
				chars.sort();
				Ok(match &chars.into_iter().collect::<String>()[..]{
					"L"=>RequestedCwy::L,
					"R"=>RequestedCwy::R,
					"S"=>RequestedCwy::R,
					"LR"=>RequestedCwy::LR,
					"LS"=>RequestedCwy::LS,
					"RS"=>RequestedCwy::RS,
					"LRS"=>RequestedCwy::LRS,
					_=>RequestedCwy::LRS
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
	pub offset: f32,
}

fn default_offset() -> f32 {
	0f32
}

fn default_cwy() -> RequestedCwy {
	RequestedCwy::LRS
}

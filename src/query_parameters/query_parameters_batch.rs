use super::{QueryParametersLine, OutputFormat, RequestedCwy};


use std::fmt;
use std::convert::TryFrom;


pub struct QueryParameterBatch(pub Vec<QueryParametersLine>);

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
		
		let mut params:Vec<QueryParametersLine> = vec![];
		

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
			params.push(QueryParametersLine{
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
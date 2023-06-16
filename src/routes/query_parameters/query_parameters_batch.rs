use super::{OutputFormat, QueryParametersLine, RequestedCwy};

use std::convert::TryFrom;
use std::fmt;

pub struct QueryParameterBatch(pub Vec<QueryParametersLine>);

#[derive(Debug)]
pub struct BatchQueryParametersDecodeError;
impl std::error::Error for BatchQueryParametersDecodeError {}
impl std::fmt::Display for BatchQueryParametersDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TryFrom<bytes::Bytes> for QueryParameterBatch {
    type Error = BatchQueryParametersDecodeError; //Box<dyn std::error::Error>;
    fn try_from(buffer: bytes::Bytes) -> Result<QueryParameterBatch, Self::Error> {
        let mut params: Vec<QueryParametersLine> = vec![];

        let mut buffer_iter = buffer.iter();

        loop {
            let road_name_byte_length = match buffer_iter.next() {
                Some(&num) => num as usize,
                None => break,
            };
            let road_name: Vec<u8> = buffer_iter
                .by_ref()
                .take(road_name_byte_length)
                .map(|&x| x)
                .collect();

            let road_name =
                std::str::from_utf8(&road_name[..]).or(Err(BatchQueryParametersDecodeError))?;

            let other_bytes = buffer_iter
                .by_ref()
                .take(13)
                .map(|&x| x)
                .collect::<Vec<u8>>();
            if other_bytes.len() != 13 {
                return Err(BatchQueryParametersDecodeError);
            }
            let slk_from = f32::from_le_bytes([
                other_bytes[0],
                other_bytes[1],
                other_bytes[2],
                other_bytes[3],
            ]); // floats come packed in french bytes apparently ;)
            let slk_to = f32::from_le_bytes([
                other_bytes[4],
                other_bytes[5],
                other_bytes[6],
                other_bytes[7],
            ]);
            let offset = f32::from_le_bytes([
                other_bytes[8],
                other_bytes[9],
                other_bytes[10],
                other_bytes[11],
            ]);
            let cwy: RequestedCwy = other_bytes[12].into();
            params.push(QueryParametersLine {
                road: road_name.to_string(),
                slk_from,
                slk_to,
                cwy,
                offset,
                m: false,
                f: OutputFormat::JSON,
            })
        }
        Ok(QueryParameterBatch(params))
    }
}

#[cfg(test)]
mod tests {
    use super::super::RequestedCwy;

    use super::*;

    use bytes::{BytesMut, BufMut};

    fn create_sample_binary(query_parameters_line: QueryParametersLine) -> bytes::Bytes {
        let mut buffer = BytesMut::new();
        
        

        // Write road name length and road name
        buffer.put_u8(query_parameters_line.road.len() as u8);
        buffer.put_slice(query_parameters_line.road.as_bytes());

        // Write SLK values, offset, and cwy
        buffer.put_slice(&query_parameters_line.slk_from.to_le_bytes());
        buffer.put_slice(&query_parameters_line.slk_to.to_le_bytes());
        buffer.put_slice(&query_parameters_line.offset.to_le_bytes());
        buffer.put_u8(query_parameters_line.cwy.into());
        // Convert BytesMut into Bytes
        buffer.freeze()
    }


    #[test]
    fn test_query_parameter_batch_try_from() {
        let sample = QueryParametersLine {
            road: "test road".to_string(),
            slk_from: 1.0,
            slk_to: 2.0,
            cwy: RequestedCwy::LRS,
            offset: 0.0,
            m: false,
            f: OutputFormat::JSON,
        };

        let binary = create_sample_binary(sample.clone());

        match QueryParameterBatch::try_from(binary) {
            Ok(batch) => assert_eq!(batch.0[0], sample),
            Err(_) => panic!("Deserialization failed"),
        }
    }
}

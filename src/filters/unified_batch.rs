use std::sync::Arc;

use warp::Filter;

use crate::{
    data::IndexedData,
    filters::{
        geoprocessing::get_linestring,
        query_parameters::output_format::{OutputFormatLines, OutputFormatPoints, OutputFormatUnified}
    }
};


use super::{
    geoprocessing::get_points,
    query_parameters::{QueryParametersPointLine, QueryParametersUnifiedGet, QueryParametersUnifiedPost},
    with_shared_data
};


pub fn unified_batch(
    indexed_data: Arc<IndexedData>
) -> impl Filter<Extract = (String,), Error = warp::Rejection> + Clone {
    use QueryParametersPointLine::*;
    warp::path("batch2").and(warp::path::end())
    .and(
        warp::post()
        .and(with_shared_data(indexed_data.clone()))
        .and(warp::body::json())
        .and_then(|
                indexed_data: Arc<IndexedData>,
                query: QueryParametersUnifiedPost,
            | async move {
                // TODO: must not be used with non JSON return types... or those must be handled differently?
                // todo: could slap rayon in here for some easy parallelization perhaps?
                let QueryParametersUnifiedPost{
                    format,
                    items
                } = query;
                let results:Vec<String> = items.iter().map(|request| match request {
                    Point(point_request) => {
                        let format:OutputFormatPoints = format.clone().into();
                        get_points(&point_request.with_format(&format), &indexed_data).unwrap_or("null".to_owned())
                    },
                    Line (line_request)  =>{
                        let format:OutputFormatLines = format.clone().into();
                        get_linestring(&line_request.with_format(&format), &indexed_data).unwrap_or("null".to_owned())
                    },
                }).collect();
                Ok::<std::string::String, warp::Rejection>(format!("[{}]",results.join(",")))
            })
    ).or(
            warp::get()
            .and(with_shared_data(indexed_data.clone()))
            .and(warp::query())
            .and_then(|
                indexed_data: Arc<IndexedData>,
                query: QueryParametersUnifiedGet,
            | async move {
                let QueryParametersUnifiedGet{
                    format,
                    items
                } = query;
                let query:Result<Vec<QueryParametersPointLine>, _> = serde_json::from_str(items.as_str());
                match query {
                    Ok(items)=>{
                        let results = items.iter().map(|request| match request {
                            Point(point_request) => {
                                let format:OutputFormatPoints = format.clone().into();
                                get_points(&point_request.with_format(&format), &indexed_data).unwrap_or("null".to_owned())
                            },
                            Line (line_request)  =>{
                                let format:OutputFormatLines = format.clone().into();
                                get_linestring(&line_request.with_format(&format), &indexed_data).unwrap_or("null".to_owned())
                            },
                        });
                        let results:Vec<String> = if format==OutputFormatUnified::wkt {
                            results.map(|item| match item.as_str(){
                                "null" => item,
                                item   => format!(r#""{}""#, item)
                            }).collect()
                        }else{
                            results.collect()
                        };
                        Ok::<std::string::String, _>(format!("[{}]",results.join(",")))
                    },
                    Err(_)=>Err(warp::reject()) // TODO: Add custom rejection
                }
            })
        )
        .unify()
   
}



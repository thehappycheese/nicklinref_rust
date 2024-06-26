use std::{error::Error, sync::Arc};

use warp::{Filter, wrap_fn, filters::BoxedFilter, reply::Response, fs::File, Reply};

use crate::{data::IndexedData, settings::Settings};

/// get the combined routes / filters to handle each feature of the server
pub async fn get_combined_filters(settings:&Settings, indexed_data:Arc<IndexedData>) -> Result<BoxedFilter<(Response,)>, Box<dyn Error>> {

    let filter_show = 
        warp::path("show")
        .and(
            warp::fs::dir(settings.NLR_STATIC_HTTP.clone())
            .map(|r:File| r.into_response())
        );
    let filter_lines          = super::lines(indexed_data.clone());
    let filter_points         = super::points(indexed_data.clone());
    let filter_lines_batch    = super::lines_batch(indexed_data.clone());
    let filter_unified_batch  = super::unified_batch(indexed_data.clone());

    // Chain filters together into a single filter
    Ok(
        filter_show
        .or(
            filter_lines
            .or(filter_points)
            .or(filter_unified_batch)
            .or(
                filter_lines_batch
                .with(warp::compression::gzip())
            )
            .recover(super::custom_rejection_handler)
            .with(wrap_fn(super::echo_x_request_id))
        ).unify()
        .boxed()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filters::query_parameters::RequestedCwy;
    use byteorder::{WriteBytesExt, LittleEndian};
    use std::io::Read;
    use flate2::read::GzDecoder;

    /// every test is compiled and executed in a sandbox
    /// rust does not natively support fixtures for testing (yet?)
    /// So, although we can make a macro to avoid code repetition,
    /// we cannot avoid running this initialisation once for every
    /// individual test.
    macro_rules! setup_filter_for_testing {
        () => {{
            // modified settings for testing
            //  - prevent saving the data file by providing an empty filepath
            //  - set data source url to download a small subset of data
            //    (The first 5-ish kilometres of H015)

            let settings = Settings {
                NLR_DATA_FILE: "".to_owned(), 
                NLR_DATA_SOURCE_URL: "https://mrgis.mainroads.wa.gov.au/arcgis/rest/services/OpenData/RoadAssets_DataPortal/MapServer/17/query?where=ROAD%3D%27H015%27%20and%20END_SLK%3C5&outFields=ROAD,START_SLK,END_SLK,CWY&outSR=4326&f=json".to_owned(),
                ..Settings::default()
            };

            let indexed_data:Arc<_> = IndexedData::load(
                &settings.NLR_DATA_FILE,
                &settings.NLR_DATA_SOURCE_URL,
                &settings.NLR_FORCE_UPDATE_DATA
            ).await.unwrap().into();

            get_combined_filters(&settings, indexed_data).await.unwrap()
        }};
    }


    #[tokio::test]
    async fn basic_tests() {
        // download data and build routes
        let filter = setup_filter_for_testing!();

        // run a bunch of tests in this one function

        println!("test: Empty GET should return invalid query");
        let result = warp::test::request().filter(&filter).await.unwrap();
        assert!(result.status().is_client_error());

        println!("test: Minimal query should succeed");
        let result = warp::test::request().method("GET").path("/?road=H015").filter(&filter).await.unwrap();
        assert!(result.status().is_success());

        println!("test: Invalid query parameter value should reject");
        let result = warp::test::request().method("GET").path("/?road=H015&cwy=2").filter(&filter).await.unwrap();
        assert!(result.status().is_client_error());

        println!("test: Invalid query parameter name should reject");
        let result = warp::test::request().method("GET").path("/?road=H015&cway=L").filter(&filter).await.unwrap();
        assert!(result.status().is_client_error());

        println!("test: Invalid HTTP method should reject");
        let result = warp::test::request().method("POST").path("/?road=H015").filter(&filter).await.unwrap();
        assert!(result.status().is_client_error());

        println!("test: Valid request should echo x-request-id");
        let result = warp::test::request().header("x-request-id", "10").path("/?road=H015").filter(&filter).await.unwrap();
        assert!(result.headers().get("x-request-id").map_or(false, |header| header=="10"));

        println!("test: Rejected request should still echo x-request-id");
        let result = warp::test::request().header("x-request-id", "11").path("/?road=H000").filter(&filter).await.unwrap();
        assert!(result.headers().get("x-request-id").map_or(false, |header| header=="11"));

        println!("test: static http");
        let result = warp::test::request().path("/show/index.html").filter(&filter).await.unwrap();
        let body_bytes = warp::hyper::body::to_bytes(result.into_body()).await.unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        let first_line = body_str.lines().next().unwrap_or("");
        assert_eq!(first_line.trim(), "<!doctype html>");
    }

    #[tokio::test]
    /// TODO: so far very primitive tests for /batch/
    async fn batch_request_test(){
        
        let filter = setup_filter_for_testing!();

        /// A quick function to serialise a single request
        fn binary_encode_request(road: &str, slk_from: f32, slk_to: f32, offset: f32, cwy: RequestedCwy) -> Vec<u8> {
            let road_bytes = road.as_bytes();
            let road_name_length = road_bytes.len() as u8;
    
            let mut buffer = Vec::with_capacity(1 + road_bytes.len() + 4 + 4 + 4 + 1);
    
            buffer.push(road_name_length);
            buffer.extend_from_slice(road_bytes);
    
            let mut wtr = vec![];
            wtr.write_f32::<LittleEndian>(slk_from).unwrap();
            wtr.write_f32::<LittleEndian>(slk_to).unwrap();
            wtr.write_f32::<LittleEndian>(offset).unwrap();
    
            buffer.append(&mut wtr);
            buffer.push(cwy.into());
    
            buffer
        }
        
        

        let req:Vec<u8> = binary_encode_request("H015", 0.1, 0.2, 0.0, RequestedCwy::L);
        println!("{:?}", req);

        println!("test: Empty batch request");
        let result = warp::test::request().method("POST").path("/batch/").filter(&filter).await.unwrap();
        let bytes = warp::hyper::body::to_bytes(result.into_body()).await.unwrap();
        let mut gz = GzDecoder::new(&*bytes);
        let mut s = String::new();
        let _size = gz.read_to_string(&mut s).unwrap();
        assert_eq!(s, "[]");
        
        println!("test: Test normal batch request");
        let result = warp::test::request().method("POST").path("/batch/").body(req).filter(&filter).await.unwrap();
        let bytes = warp::hyper::body::to_bytes(result.into_body()).await.unwrap();
        let mut gz = GzDecoder::new(&*bytes);
        let mut s = String::new();
        let _size = gz.read_to_string(&mut s).unwrap();
        println!("{:}",s);
        assert_eq!(s[..3].to_string(), "[[[");
    }

}
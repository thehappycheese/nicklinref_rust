use std::{net::SocketAddr, error::Error};
use clap::Parser;

mod helpers;
mod routes;
mod data;
mod settings;

use settings::Settings;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    
    // Read settings from command line (or environment variables)
    // see run with --help to see nice instructions generated by clap
    let settings:Settings = Settings::parse();

    let filter = routes::load_data_and_get_combined_routes(&settings).await?;

    // Serve
    let address:SocketAddr = (settings.NLR_ADDR, settings.NLR_PORT).into();
    println!("Serving at {:?}", address);
    warp::serve(filter).run(address).await;

    Ok(())
}


#[cfg(test)]
mod main_tests {
    use crate::{routes::load_data_and_get_combined_routes, settings::Settings, routes::query_parameters::RequestedCwy};

    /// every test is compiled and executed in a sandbox
    /// rust does not natively support fixtures for testing
    macro_rules! setup_routes_for_testing {
        () => {
            // modified settings for testing
            //  - prevent saving the data file by providing an empty filepath
            //  - set data source url to download a small subset of data
            //    (The first 5-ish kilometres of H015)
            load_data_and_get_combined_routes(&Settings {
                NLR_DATA_FILE: "".to_owned(), 
                NLR_DATA_SOURCE_URL: "https://mrgis.mainroads.wa.gov.au/arcgis/rest/services/OpenData/RoadAssets_DataPortal/MapServer/17/query?where=ROAD%3D%27H015%27%20and%20END_SLK%3C5&outFields=ROAD,START_SLK,END_SLK,CWY&outSR=4326&f=json".to_owned(),
                ..Settings::default()
            }).await.unwrap()
        };
    }


    #[tokio::test]
    async fn basic_tests() {
        // download data and build routes
        let filter = setup_routes_for_testing!();

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
    }

    }
}
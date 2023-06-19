mod helpers;
mod filters;
mod data;
mod settings;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let settings = settings::Settings::get_from_cli_or_env();

    let indexed_data = data::IndexedData::load(
        &settings.NLR_DATA_FILE,
        &settings.NLR_DATA_SOURCE_URL,
        &settings.NLR_FORCE_UPDATE_DATA
    ).await?;

    let filter = filters::get_combined_filters(&settings, indexed_data.into()).await?;

    println!("Serving at {:?}", settings.get_socket_address());
    warp::serve(filter).run(settings.get_socket_address()).await;

    Ok(())
}

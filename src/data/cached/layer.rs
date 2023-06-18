use std::{
    path::Path,
    fs::{
        File,
        create_dir_all,
        remove_file
    },
    error::Error
};

use lz_fear;
use reqwest;
use serde_json;
use serde::{Deserialize, Serialize};

use crate::helpers::ErrorWithStaticMessage;
use super::{
    super::esri_json,
    Feature
};

#[derive(Serialize, Deserialize, Debug)]
/// This is the struct / document that will be saved to local storage
pub struct Layer {
    pub features: Vec<Feature>,
}

impl Layer {
    pub async fn download_data(
        url:&String,
    ) -> Result<Self, Box<dyn Error>> {
        
        let mut document_to_save = Self {
            features: Vec::with_capacity(183_000),
        };

        let mut offset: usize = 0;
        const MAX_LOOPS: usize = 500;
        let mut loop_safety_limit = MAX_LOOPS;
        loop {
            loop_safety_limit -= 1;
            if loop_safety_limit <= 0 {
                return Err(
                    Box::new(ErrorWithStaticMessage::new("Download failed. Loop safety limit exceeded."))
                );
            }

            let url = format!("{}&resultOffset={}", url, offset);
            let json: esri_json::EsriFeatureSet = reqwest::get(url).await?.json().await?;
            if json.features.len()==0 {
                break;
            }
            offset += json.features.len();
            document_to_save.features.extend(
                json.features
                    .into_iter()
                    .map(|item| item.into()),
            );


            match json.exceededTransferLimit {
                Some(true) => (),
                Some(false) | None => break,
            }
        }
        if document_to_save.features.len()==0{
            return Err(
                Box::new(ErrorWithStaticMessage::new("Download failed. No features were received."))
            );
        }
        println!("INFO: Download completed. Sorting data.");
        document_to_save
            .features
            .sort_by(|a, b| a.attributes.cmp(&b.attributes));
        Ok(document_to_save)
    }

    pub fn load_data_from_file(file_path:&Path) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let lz_frame_reader = lz_fear::framed::LZ4FrameReader::new(file)?;
        let lz_frame_io_reader = lz_frame_reader.into_read();
        let result = serde_json::from_reader(lz_frame_io_reader)?;
        Ok(result)
    }

    pub fn save_data_to_file(file_path:&Path, data:&Self) -> Result<(), Box<dyn Error>> {
        let file_out = File::create(file_path)?;
        let res = serde_json::to_vec(&data)?;
        let compressor = lz_fear::framed::CompressionSettings::default();
        let result = compressor.compress(&res[..], &file_out)?;
        Ok(result)
    }

    pub async fn read_or_update_cache_data (
        path_to_data_cache_file:&String,
        url_to_download_new_data:&String,
        force_update:&bool,
    ) -> Result<Self, Box<dyn Error>> {
        
        let path_to_data_cache_file = Path::new(path_to_data_cache_file);
    
        if !force_update {
            println!("INFO: Loading data from '{}'.", path_to_data_cache_file.display());
            match Self::load_data_from_file(path_to_data_cache_file) {
                Ok(loaded_data) => return Ok(loaded_data),
                Err(error_message) => println!("WARNING: Could not open the specified data file because '{}'. Will try to download fresh data.", error_message),
            }
        }
    
        if let Some(parent) = path_to_data_cache_file.parent() {
            if !parent.is_dir() {
                println!("WARNING: The directory '{}' does not exist. Creating it now", parent.display());
                match create_dir_all(parent) {
                    Ok(_) => (),
                    Err(error_message) => {
                        println!("WARNING: Could not create the directory '{}' because '{}'", parent.display(), error_message);
                    }
                }
            }
        }
    
        if path_to_data_cache_file.is_file() {
            println!("WARNING: The file '{}' already exists. Attempting to delete it", path_to_data_cache_file.display());
            match remove_file(path_to_data_cache_file) {
                Ok(_) => (),
                Err(error_message) => {
                    println!("WARNING: Could not delete the file '{}' because '{}'", path_to_data_cache_file.display(), error_message);
                }
            };
        }
        
        
        println!("INFO: Downloading fresh data");
        println!("INFO: Using '{}'", url_to_download_new_data);
        println!("INFO: Please wait, this can take some time...");
        let new_data = Self::download_data(url_to_download_new_data).await?;
        
        println!("INFO: Saving data to file.");
        match Self::save_data_to_file(path_to_data_cache_file, &new_data){
            Ok(_) => (),
            Err(error_message) => {
                println!("WARNING: Could not save the specified data file '{}' because '{}'", path_to_data_cache_file.display(), error_message);
                println!("WARNING: Data will be downloaded again next time the program is run");
            }
        }
        
    
        Ok(new_data)
    }
}
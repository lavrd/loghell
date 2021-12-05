use std::error::Error;
use std::fs;

use log::debug;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub storage: Storage,
}

#[derive(Debug, Deserialize)]
pub struct Storage {
    pub tantivy: Tantivy,
}

#[derive(Debug, Deserialize)]
pub struct Tantivy {
    pub fields: TantivyFields,
}

#[derive(Debug, Deserialize)]
pub struct TantivyFields {
    pub text: Box<[String]>,
}

impl Config {
    pub fn new(config_path: &str) -> Result<Self, Box<dyn Error>> {
        let config_as_str = fs::read_to_string(config_path)?;
        let config: Config = serde_yaml::from_str(&config_as_str)?;
        debug!("using following config : {:?} from {}", config, config_path);
        Ok(config)
    }
}

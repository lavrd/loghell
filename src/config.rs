use std::fs;

use log::debug;
use serde::Deserialize;

use crate::FnRes;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub storage: Storage,
}

#[derive(Debug, Deserialize)]
pub struct Storage {
    pub fields: Fields,
}

#[derive(Debug, Deserialize)]
pub struct Fields {
    pub text: Box<[String]>,
}

impl Config {
    pub fn new(config_path: &str) -> FnRes<Self> {
        let config_as_str = fs::read_to_string(config_path)?;
        let config: Config = serde_yaml::from_str(&config_as_str)?;
        debug!("using following config : {:?} from {}", config, config_path);
        Ok(config)
    }
}

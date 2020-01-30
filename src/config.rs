use crate::error::*;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub token: String,
}

impl Config {
    fn get_path() -> PathBuf {
        let home = env::var("HOME").map(PathBuf::from).unwrap();
        home.join(".sharpener-config")
    }

    pub fn create(token: String) -> Result<Self> {
        let filename = Self::get_path();
        let file = File::create(&filename).context(OpenConfigFile { filename })?;
        let config = Self { token };
        serde_json::to_writer_pretty(file, &config).context(ConfigWrite {})?;
        Ok(config)
    }

    pub fn load() -> Result<Self> {
        let filename = Self::get_path();
        let file = File::open(&filename).context(OpenConfigFile { filename })?;
        serde_json::from_reader(file).context(ConfigParsing {})
    }
}

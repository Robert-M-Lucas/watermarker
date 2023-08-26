use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub struct Config {
    pub offset: u32,// = 0;
    pub watermark_interval: u32,// = 400;
    pub scale: u32,// = 2;
}

impl Config {
    pub fn get_config_or_default<P: AsRef<Path>>(path: P) -> Config {
        let data = fs::read_to_string(&path);
        if let Ok(data) = data {
            return serde_json::from_str(&data).expect("JSON deserialisation failed");
        }

        println!("config.json not found - creating default");
        let config = Config::default();
        let json = serde_json::to_string_pretty(&config).expect("JSON serialisation failed");
        let r = fs::write(&path, json);
        if r.is_err() {
            println!("Writing default config.json file failed");
        }
        config
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            offset: 0,
            watermark_interval: 400,
            scale: 2
        }
    }
}
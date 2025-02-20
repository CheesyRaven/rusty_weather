use serde::Deserialize;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug, serde::Serialize, Deserialize)]
struct Config {
    api_key: String,
    latitude: i32,
    longitude: i32,
    units: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = "config.yaml";

    let config: Config = if Path::new(config_path).exists() {
        println!("Config file found, loading...");
        let mut file = File::open(config_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        serde_yaml::from_str(&contents)?
    } else {
        println!("Config file not found, creating default...");
        let default_config = Config {
            api_key: "".to_string(),
            latitude: 0,
            longitude: 0,
            units: "imperial".to_string(),
        };

        let yaml_string = serde_yaml::to_string(&default_config)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(config_path)?;
        file.write_all(yaml_string.as_bytes())?;

        default_config
    };

    println!("Loaded config: {:?}", config);
    Ok(())
}
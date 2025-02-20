use serde_json::Value;
use clap::Parser;
use serde::Deserialize;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

/// Command-line arguments
#[derive(Parser, Debug)]
#[command(name = "config_app", about = "Configuration Manager")]
struct Args {
    /// If set, prompts the user to create a new config file
    #[arg(short, long)]
    setup: bool,
}

#[derive(Debug, serde::Serialize, Deserialize)]
struct Config {
    api_key: String,
    latitude: f64,
    longitude: f64,
    units: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();
    let config_path = "config.yaml";

    let mut config: Config = if Path::new(config_path).exists() {
        println!("Config file found, loading...");
        let mut file = File::open(config_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        serde_yaml::from_str(&contents)?
    } else {
        println!("Config file not found, creating default...");
        Config {
            api_key: "".to_string(),
            latitude: 0.0,
            longitude: 0.0,
            units: "imperial".to_string(),
        }
    };

    if args.setup {
        println!("Updating configuration...");
        update_config(&mut config);
        save_config(&config, config_path)?;
        println!("Configuration updated successfully.");
        return Ok(());
    }

    println!("Loaded config: {:?}", config);
    Ok(())
}

/// Updates the configuration by prompting the user for new values
fn update_config(config: &mut Config) {
    println!("Press Enter to keep existing values.");

    config.api_key = prompt_update("Enter API key", &config.api_key);
    config.units = prompt_update("Enter units (imperial, metric, default(Kelvin))", &config.units.to_string());
    let zip_code = prompt_update("Enter ZIP code (or press Enter to skip)", "");
    if !zip_code.is_empty() {
        match get_lat_long(&zip_code, &config.api_key) {
            Ok((lat, lon)) => {
                println!("Coordinates found: Latitude = {}, Longitude = {}", lat, lon);
                config.latitude = lat;
                config.longitude = lon;
            }
            Err(e) => println!("Failed to retrieve coordinates: {}", e),
        }
    }
}

/// Prompts the user for a new value, keeping the existing value if Enter is pressed
fn prompt_update(prompt: &str, current: &str) -> String {
    println!("{} (current: {}):", prompt, current);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let trimmed = input.trim();
    if trimmed.is_empty() {
        current.to_string()
    } else {
        trimmed.to_string()
    }
}

fn get_lat_long(zip_code: &str, api_key: &str) -> Result<(f64, f64), Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?zip={}&appid={}",
        zip_code, api_key
    );

    let response = ureq::get(&url).call()?.into_string()?;
    let json: Value = serde_json::from_str(&response)?;

    let lat = json["coord"]["lat"].as_f64().ok_or("Latitude not found")?;
    let lon = json["coord"]["lon"].as_f64().ok_or("Longitude not found")?;

    Ok((lat, lon))
}


/// Saves the configuration to a YAML file
fn save_config(config: &Config, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let yaml_string = serde_yaml::to_string(config)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    file.write_all(yaml_string.as_bytes())?;
    Ok(())
}
use std::cmp;
use std::collections::HashMap;
use std::error::Error;
use serde_json::{Value};
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

    #[arg(short, long, value_name = "ZIP")]
    zip: Option<String>,
}

#[derive(Debug, serde::Serialize, Deserialize)]
struct Config {
    api_key: String,
    latitude: f64,
    longitude: f64,
    units: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = Args::parse();
    let config_path = "config.yaml";

    let mut config: Config = if Path::new(config_path).exists() {
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

    if config.api_key.is_empty() {
        println!("No API key configured, please run --setup.");
        return Ok(());
    }

    let api_key = &config.api_key;
    let mut lat = config.latitude;
    let mut lon = config.longitude;
    let units = &config.units;

    if let Some(zip) = args.zip {
        match get_lat_long(&zip, &config.api_key) {
            Ok((lat_from_option, lon_from_option)) => {
                lat = lat_from_option;
                lon = lon_from_option;
            },
            Err(_) => todo!()
        }
    }

    match get_weather(api_key, &lat, &lon, units) {
        Ok(json) => print_weather_info(&json),
        Err(e) => eprintln!("Error fetching weather data: {}", e),
    }

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


/// Saves the configuration to a YAML file
fn save_config(config: &Config, path: &str) -> Result<(), Box<dyn Error>> {
    let yaml_string = serde_yaml::to_string(config)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    file.write_all(yaml_string.as_bytes())?;
    Ok(())
}

fn get_lat_long(zip_code: &str, api_key: &str) -> Result<(f64, f64), Box<dyn Error>> {
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

/// Fetches weather data from OpenWeatherMap API and returns JSON.
fn get_weather(api_key: &str, lat: &f64, lon: &f64, units: &str) -> Result<Value, Box<dyn Error>> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&appid={}&units={}",
        lat, lon, api_key, units
    );

    let response = ureq::get(&url).call()?.into_string()?;
    let json: Value = serde_json::from_str(&response)?;

    Ok(json)
}

/// Print formatted response
fn print_weather_info(json: &Value) {
    let city = json["name"].as_str().unwrap_or("Unknown");
    let temp = json["main"]["temp"].as_f64().unwrap_or(0.0);
    let temp_max = json["main"]["temp_max"].as_f64().unwrap_or(0.0);
    let temp_min = json["weather"][0]["temp_min"].as_f64().unwrap_or(0.0);
    let wind_speed = json["wind"]["speed"].as_f64().unwrap_or(0.0);
    let description = json["weather"][0]["main"].as_str().unwrap_or("Unknown");

    // Define ASCII Art HashMap
    let weather_art: HashMap<&str, Vec<&str>> = HashMap::from([
        ("Clear", vec![" \\ | / ", "- ( ) -", " / | \\ ", "       "]),
        ("Clouds", vec!["    .-.   ", " .-(   ). ", "(________)", "          "]),
        ("Rain", vec!["' '' '", " ' '' ", "''  ' ", "      "]),
        ("Snow", vec!["*  * *", " *  * ", "* *  *", "      "]),
    ]);

    // Get ASCII art for the weather condition, or fallback to default
    let binding = vec!["   ", "   ", "   ", "   "];
    let art = weather_art.get(description).unwrap_or(&binding);

    let width = cmp::max(art[3].len(), city.len());

    let city_centered = format!("{:^width$}", city, width = width);

    println!("{} | Temperature: {}", format!("{:^width$}", art[0], width = width), temp);
    println!("{} | Min: {}", format!("{:^width$}", art[1], width = width), temp_max);
    println!("{} | Max: {}", format!("{:^width$}", art[2], width = width), temp_min);
    println!("{} | Wind Speed: {}", city_centered, wind_speed);
}
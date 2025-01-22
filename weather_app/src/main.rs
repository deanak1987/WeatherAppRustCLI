use chrono::{DateTime, Local};
use clap::Parser;
use colored::Colorize;
use serde::Deserialize;
use std::env;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The city to get the weather for
    city: String,

    /// Display temperature in Fahrenheit instead of Celsius
    #[arg(short, long)]
    fahrenheit: bool,
}

#[derive(Deserialize, Debug)]
struct WeatherResponse {
    main: Main,
    weather: Vec<Weather>,
    name: String,
    wind: Wind,
    sys: Sys,
}

#[derive(Deserialize, Debug)]
struct Main {
    temp: f64,
    temp_max: f64,
    temp_min: f64,
    feels_like: f64,
    humidity: i32,
}

#[derive(Deserialize, Debug)]
struct Weather {
    description: String,
    main: String,
}

#[derive(Deserialize, Debug)]
struct Wind {
    speed: f64,
    deg: Option<f64>,
}

#[derive(Deserialize, Debug)]
struct Sys {
    sunrise: i64,
    sunset: i64,
}

fn kelvin_to_celsius(kelvin: f64) -> f64 {
    kelvin - 273.15
}

fn kelvin_to_fahrenheit(kelvin: f64) -> f64 {
    (kelvin - 273.15) * 9.0 / 5.0 + 32.0
}

fn meters_per_second_to_kmh(mps: f64) -> f64 {
    mps * 3.6
}

fn get_wind_direction(degrees: f64) -> &'static str {
    let directions = [
        "N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE", "S", "SSW", "SW", "WSW", "W", "WNW",
        "NW", "NNW",
    ];
    let index = ((degrees + 11.25) % 360.0 / 22.5) as usize;
    directions[index]
}

fn get_weather_emoji(weather_main: &str) -> &str {
    match weather_main.to_lowercase().as_str() {
        "clear" => "☀️",
        "clouds" => "☁️",
        "rain" => "🌧️",
        "snow" => "❄️",
        "thunderstorm" => "⛈️",
        "drizzle" => "🌦️",
        "mist" | "fog" => "🌫️",
        _ => "🌡️",
    }
}

fn format_timestamp(timestamp: i64) -> String {
    let datetime = DateTime::from_timestamp(timestamp, 0)
        .expect("Invalid timestamp")
        .with_timezone(&Local);
    datetime.format("%H:%M").to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read the API key from the environment
    let api_key = env::var("WEATHER_API_KEY")
        .map_err(|_| "Please set the WEATHER_API_KEY environment variable")?;

    let args = Cli::parse();

    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}",
        args.city, api_key
    );

    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Failed to fetch weather data: {}", e))?
        .json::<WeatherResponse>()
        .await
        .map_err(|e| format!("Failed to parse weather data: {}", e))?;

    let temp = if args.fahrenheit {
        kelvin_to_fahrenheit(response.main.temp)
    } else {
        kelvin_to_celsius(response.main.temp)
    };

    let temp_max = if args.fahrenheit {
        kelvin_to_fahrenheit(response.main.temp_max)
    } else {
        kelvin_to_celsius(response.main.temp_max)
    };

    let temp_min = if args.fahrenheit {
        kelvin_to_fahrenheit(response.main.temp_min)
    } else {
        kelvin_to_celsius(response.main.temp_min)
    };

    let feels_like = if args.fahrenheit {
        kelvin_to_fahrenheit(response.main.feels_like)
    } else {
        kelvin_to_celsius(response.main.feels_like)
    };

    let temp_unit = if args.fahrenheit { "°F" } else { "°C" };
    let wind_speed_kmh = meters_per_second_to_kmh(response.wind.speed);

    // Get wind direction if available
    let wind_direction = response.wind.deg.map(get_wind_direction).unwrap_or("-");

    // Get the first weather description or provide a default
    let weather = response
        .weather
        .first()
        .map(|w| (w.description.clone(), w.main.clone()))
        .unwrap_or_default();

    println!("\n{}", "Current Weather".bold().underline());
    println!("🌍 Location: {}", response.name.bright_blue());
    println!(
        "{}  Weather: {}",
        get_weather_emoji(&weather.1),
        weather.0.bright_yellow()
    );
    println!(
        "🌡️  Temperature: {}{:.1}{}",
        if temp <0.0 {"-"} else { ""},
        temp.abs().to_string().bright_green(),
        temp_unit
    );

    println!(
        "🤔 Feels like: {}{:.1}{}",
        if feels_like < 0.0 { "-" } else { "" },  
        feels_like.abs().to_string().bright_green(), 
        temp_unit
    );

    println!(
        "🌡️  Today's High/Low: {}{:.1}{}/{}{:.1}{}",
        if temp_max < 0.0 { "-" } else { "" },  
        temp_max.abs().to_string().bright_green(),
        temp_unit,
        if temp_min < 0.0 { "-" } else { "" },  
        temp_min.abs().to_string().bright_green(),
        temp_unit
    );

    println!(
        "💧 Humidity: {}%",
        response.main.humidity.to_string().bright_cyan()
    );

    // Wind information
    println!(
        "🌪️  Wind: {:.1} km/h from {}",
        wind_speed_kmh.to_string().bright_magenta(),
        wind_direction.bright_magenta()
    );

    // Sun information
    println!(
        "🌅 Sunrise: {}",
        format_timestamp(response.sys.sunrise).bright_yellow()
    );
    println!(
        "🌇 Sunset: {}\n",
        format_timestamp(response.sys.sunset).bright_yellow()
    );

    

    Ok(())
}

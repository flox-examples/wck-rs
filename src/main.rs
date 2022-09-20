use std::fmt::Display;
use std::str::FromStr;

use anyhow::anyhow;
use anyhow::Result;
use anyhow::bail;
use clap::{Parser};
use reqwest::{Url};
use serde::de;
use serde::Deserialize;
use serde::Deserializer;

#[derive(Parser)]
#[clap(author, version, about)]

struct CLI {
    #[clap(value_parser)]
    location: Option<String>,
}

impl TryInto<Url> for CLI {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Url, Self::Error> {
        let mut url = Url::parse("https://wttr.in")?;
        match self.location {
            Some(loc) => url.set_path(&format!("~{}", loc.replace(" ", "+"))),
            None => {}
        }
        url.query_pairs_mut().append_pair("format", "j1");
        url.query_pairs_mut().append_pair("lang", "en");
        Ok(url)
    }
}

fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}

#[derive(Deserialize)]
struct API {
    current_condition: Vec<Conditions>,
    weather: Vec<Weather>,
    nearest_area: Vec<Area>,
}

#[derive(Deserialize)]
struct Conditions {
    #[serde(rename = "precipMM")]
    #[serde(deserialize_with = "from_str")]
    precip_mm: f32,

    #[serde(rename = "humidity")]
    #[serde(deserialize_with = "from_str")]
    humidity: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Weather {
    #[serde(rename = "mintempC")]
    #[serde(deserialize_with = "from_str")]
    min_temp: i32,
    #[serde(rename = "maxtempC")]
    #[serde(deserialize_with = "from_str")]
    max_temp: i32,
    astronomy: Vec<Astronomy>,
    #[serde(deserialize_with = "from_str")]
    sun_hour: f32,
    #[serde(rename = "totalSnow_cm")]
    #[serde(deserialize_with = "from_str")]
    total_snow_cm: f32,
    #[serde(deserialize_with = "from_str")]
    uv_index: i32,
}


#[derive(Deserialize)]
struct Astronomy {
    moon_phase: String,
}

#[derive(Deserialize, Clone)]
struct Area {
    #[serde(rename = "areaName")]
    area_name: Vec<Value>,
    country: Vec<Value>,
}

#[derive(Deserialize, Clone)]
struct Value {
    value: String,
}

struct Hazards {
    location: Location,
    vampires: Vampires,
    precipitation: Precipitation,
    temperature: Temperature,
    sun: Sun,
}

struct Location {
    country: String,
    area_name: String,
}

enum Precipitation {
    Ok,
    Dry,
    Humid,
    Drown,
}

impl Display for Precipitation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Precipitation::Ok => write!(f, ""),
            Precipitation::Dry => write!(f, "🐪"),
            Precipitation::Humid => write!(f, "😰"),
            Precipitation::Drown => write!(f, "🌊"),
        }
    }
}

enum Temperature {
    Ok,
    Freeze,
    Burn,
}

impl Display for Temperature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Temperature::Ok => write!(f, ""),
            Temperature::Freeze => write!(f, "🥶"),
            Temperature::Burn => write!(f, "🥵"),
        }
    }
}

enum Sun {
    Ok,
    Sunburn,
    Depression,
}

impl Display for Sun {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sun::Ok => write!(f, ""),
            Sun::Sunburn => write!(f, "🕶"),
            Sun::Depression => write!(f, "😔"),
        }
    }
}

enum Vampires {
    No,
    Yes,
}

impl Display for Vampires {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Vampires::No => write!(f, ""),
            Vampires::Yes => write!(f, "🧛🏻"),
        }
    }
}

impl TryFrom<API> for Hazards {
    type Error = anyhow::Error;

    fn try_from(value: API) -> Result<Self, Self::Error> {
        let location = {
            let area = value
                .nearest_area
                .first()
                .ok_or_else(|| anyhow!("No location information found"))?;
            let country = area
                .country
                .first()
                .ok_or_else(|| anyhow!("No country information found"))?
                .value
                .clone();
            let area_name = area
                .area_name
                .first()
                .ok_or_else(|| anyhow!("No area information found"))?
                .value
                .clone();
            Location { area_name, country }
        };
        let conditions = value.current_condition.first().ok_or_else(|| anyhow!("No cuuurent conditions found"))?;

        let weather = value
            .weather
            .first()
            .ok_or_else(|| anyhow!("No weather infomration found"))?;
        let astronomy = weather
            .astronomy
            .first()
            .ok_or_else(|| anyhow!("No astronomy data found"))?;

    

        let precipitation = match (conditions.humidity, conditions.precip_mm, weather.max_temp) {
            (_, rain, max_temp) if rain > 100.0 => Precipitation::Drown,
            (humidity, rain, max_temp) if humidity < 10 && rain < 0.5 && max_temp > 30 => Precipitation::Dry,
            (humidity, rain, max_temp) if humidity > 80 && rain < 0.5 && max_temp > 30 => Precipitation::Humid,
            _ => Precipitation::Ok,
        };

        let temperature = match (weather.min_temp, weather.max_temp) {
            (min, _) if min < -6 => Temperature::Freeze,
            (_, max) if max > 37 => Temperature::Burn,
            _ => Temperature::Ok,
        };

        let sun = match (weather.sun_hour, weather.uv_index) {
            (_, 8..) => Sun::Sunburn,
            (0.0..=3.0, _) => Sun::Depression,
            _ => Sun::Ok,
        };

        Ok(Hazards {
            location,
            vampires: if weather.sun_hour < 2.0 && astronomy.moon_phase == "Full Moon" {
                Vampires::Yes
            } else {
                Vampires::No
            },
            precipitation,
            temperature,
            sun,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = CLI::try_parse()?;
    let hazards = send_request(cli).await?;
    print_hazards(hazards);

    Ok(())
}

async fn send_request(options: CLI) -> Result<Hazards> {
    let url: Url = options.try_into()?;
    let response = reqwest::get(url).await?;
    
    match response.status() {
        code if code == 404 => bail!("Location not found"),
        code if code != 200 => bail!("Request to weather service returned unsuccessful"),
        _ => {}
    }

    let api: API = serde_json::from_slice(&response.bytes().await?)?;
    let hazards = Hazards::try_from(api)?;
    Ok(hazards)
}

fn print_hazards(
    Hazards {
        location,
        vampires,
        precipitation,
        temperature,
        sun,
    }: Hazards,
) {
    println!(
        "Current Metereological Safety Hazards in {}, {}:",
        location.area_name, location.country
    );
    println!("[ {vampires}{precipitation}{temperature}{sun} ]");
}

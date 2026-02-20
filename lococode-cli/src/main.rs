use std::time::Instant;

use anyhow::{Result, anyhow};
use clap::{Args, Parser};
use lococode_core::Coordinates;

#[derive(Debug, Args)]
struct CoordinateSource {
    #[arg(conflicts_with = "lat", conflicts_with = "lon")]
    coordinates: Option<String>,
    #[arg(long, conflicts_with = "coordinates")]
    lat: Option<f64>,
    #[arg(long, conflicts_with = "coordinates")]
    lon: Option<f64>,
}

#[derive(Parser)]
enum Command {
    Encode {
        #[clap(flatten)]
        coords: CoordinateSource,
        #[arg(short, long = "len", default_value_t = 4)]
        length: u8,
    },
    Decode {
        code: String,
    },
    Test {
        #[clap(flatten)]
        coords: CoordinateSource,
        #[arg(short, long = "len", default_value_t = 4)]
        length: u8,
    },
}

fn main() {
    let args = Command::parse();

    match args {
        Command::Encode { coords, length } => {
            let coords: Coordinates<f64> = coords
                .to_coordinates()
                .expect("Failed to parse coordinates");
            let code = coords.encode(length);
            println!("{code}");
        }
        Command::Decode { code } => {
            let dec = Coordinates::decode(&code);
            println!("{dec}");
        }
        Command::Test { coords, length } => {
            let coords: Coordinates<f64> = coords
                .to_coordinates()
                .expect("Failed to parse coordinates");
            let start = Instant::now();
            let code = coords.encode(length);
            let t_enc = start.elapsed();
            let decode = Coordinates::decode(&code);
            let t_dec = start.elapsed();
            println!("In..... {coords}");
            println!("Code... {code}");
            println!("Out\n{decode}");

            let lat_err = (decode.latitude - coords.latitude).abs();
            let lon_err = (decode.longitude - coords.longitude).abs();
            println!("Latitude error.... {lat_err:.9}");
            println!("Longitude error... {lon_err:.9}");
            assert!(lat_err <= decode.lat_half_extent_deg);
            assert!(lon_err <= decode.lon_half_extent_deg);
            println!("Encoding time..... {t_enc:?}");
            println!("Decoding time..... {:?}", t_dec - t_enc);
            println!("Round trip time... {t_dec:?}");
        }
    }
}

impl CoordinateSource {
    fn to_coordinates(&self) -> Result<Coordinates<f64>> {
        let (latitude, longitude) = if let Some(coords) = &self.coordinates {
            let (lat, lon) = coords
                .split_once(',')
                .expect("Invalid coordinates. expected lat,lon");
            let lat = lat.trim().parse()?;
            let lon = lon
                .trim()
                .parse()
                .inspect_err(|e| eprintln!("Error while parsing longitude {lon}: {e}"))?;
            (lat, lon)
        } else if let Some(lat) = self.lat
            && let Some(lon) = self.lon
        {
            (lat, lon)
        } else {
            return Err(anyhow!("Missing coordinates"));
        };
        Ok(Coordinates {
            latitude,
            longitude,
        })
    }
}

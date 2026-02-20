use std::fmt;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
mod wasm;

const K: usize = 56;
const SCALE: f64 = (1u64 << K) as f64;
const ALPHABET: [char; 32] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '2', '3', '4', '5', '6', '7',
];

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Coordinates<T> {
    pub latitude: T,
    pub longitude: T,
}

#[derive(Debug)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Decoded {
    pub latitude: f64,
    pub longitude: f64,
    pub lat_half_extent_deg: f64,
    pub lon_half_extent_deg: f64,
    pub lat_half_extent_m: f64,
    pub lon_half_extent_m: f64,
    pub error_radius_m: f64,
}

impl Coordinates<f64> {
    pub const fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
        }
    }

    fn normalize(&self) -> Coordinates<u64> {
        let lat = ((90. - self.latitude) / 180.).clamp(0.0, 1.0 - 1e-9);
        let lon = ((self.longitude + 180.) / 360.).clamp(0.0, 1.0 - 1e-9);
        Coordinates {
            latitude: (lat * SCALE).floor() as u64,
            longitude: (lon * SCALE).floor() as u64,
        }
    }

    pub fn encode(&self, length: u8) -> String {
        self.normalize().encode(length)
    }

    pub fn decode(code: &str) -> Decoded {
        let l = code.len();
        assert!(l <= (K - 1) / 3, "The code is too long");
        let mut c = Coordinates::<u64>::decode(code);

        let even = l / 2;
        let odd = l - even;
        let lat_bits = odd * 2 + even * 3;
        let lon_bits = odd * 3 + even * 2;
        let lat_half_cell: u64 = 1 << (K - lat_bits - 1);
        let lon_half_cell: u64 = 1 << (K - lon_bits - 1);
        let lat_half_extent_deg = lat_half_cell as f64 / SCALE * 180.0;
        let lon_half_extent_deg = lon_half_cell as f64 / SCALE * 360.0;

        c.latitude += lat_half_cell;
        c.longitude += lon_half_cell;
        let center = c.denormalize();

        let lat_half_extent_m =
            lat_half_extent_deg * meters_per_degree_lat(center.latitude.to_radians());
        let lon_half_extent_m =
            lon_half_extent_deg * meters_per_degree_lon(center.latitude.to_radians());
        let error_radius_m = lat_half_extent_m.hypot(lon_half_extent_m);
        Decoded {
            latitude: center.latitude,
            longitude: center.longitude,
            lat_half_extent_deg,
            lon_half_extent_deg,
            lat_half_extent_m,
            lon_half_extent_m,
            error_radius_m,
        }
    }
}

impl Coordinates<u64> {
    fn denormalize(&self) -> Coordinates<f64> {
        let lat = self.latitude as f64 / SCALE * 180.;
        let lon = self.longitude as f64 / SCALE * 360.0;
        Coordinates {
            latitude: 90. - lat,
            longitude: lon - 180.,
        }
    }

    fn encode(&self, length: u8) -> String {
        let mut code = String::new();
        for i in 0..usize::from(length) {
            let iter = i + 1;
            let even = iter / 2;
            let odd = iter - even;
            let lat_shift = K - (2 * odd + 3 * even);
            let lon_shift = K - (3 * odd + 2 * even);
            let lat_bits = 2 + i % 2;
            let lon_bits = 5 - lat_bits;
            let top_lat = ((self.latitude >> lat_shift) & ((1 << lat_bits) - 1)) as u8;
            let top_lon = ((self.longitude >> lon_shift) & ((1 << lon_bits) - 1)) as u8;
            let s = if i % 2 == 0 {
                interleave(top_lat, top_lon)
            } else {
                interleave(top_lon, top_lat)
            };
            code.push(ALPHABET[s as usize]);
        }
        code
    }

    fn decode(code: &str) -> Self {
        let (latitude, longitude) = code
            .chars()
            .map(|c| ALPHABET.iter().position(|x| x == &c).unwrap() as u8)
            .enumerate()
            .map(|(i, idx)| {
                let iter = i + 1;
                let even = iter / 2;
                let odd = iter - even;
                let lat_shift = K - (2 * odd + 3 * even);
                let lon_shift = K - (3 * odd + 2 * even);
                let split = de_interleave(idx);
                let (lat, lon) = if i % 2 == 0 {
                    split
                } else {
                    (split.1, split.0)
                };
                (u64::from(lat) << lat_shift, u64::from(lon) << lon_shift)
            })
            .fold((0, 0), |(acc_lat, acc_lon), (curr_lat, curr_lon)| {
                (acc_lat | curr_lat, acc_lon | curr_lon)
            });
        Self {
            latitude,
            longitude,
        }
    }
}

/// Bit pattern: LO LA LO LA LO (A: 2b, B: 2b)
const fn interleave(lat: u8, lon: u8) -> u8 {
    (lon & 1)
        | ((lat & 1) << 1)
        | (((lon >> 1) & 1) << 2)
        | (((lat >> 1) & 1) << 3)
        | (((lon >> 2) & 1) << 4)
}

/// Bit pattern: LO LA LO LA LO  (A: 2b, B: 2b)
const fn de_interleave(x: u8) -> (u8, u8) {
    let lat = ((x & 0b1000) >> 2) | ((x & 0b10) >> 1);
    let lon = ((x & 0b10000) >> 2) | ((x & 0b100) >> 1) | (x & 1);
    (lat, lon)
}

fn meters_per_degree_lat(lat_rad: f64) -> f64 {
    1.175f64.mul_add(
        (4.0 * lat_rad).cos(),
        559.82f64.mul_add(-(2.0 * lat_rad).cos(), 111_132.92),
    )
}

fn meters_per_degree_lon(lat_rad: f64) -> f64 {
    meters_per_degree_lat(lat_rad) * lat_rad.cos()
}

impl fmt::Display for Coordinates<f64> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.6}, {:.6}", self.latitude, self.longitude)
    }
}

impl fmt::Display for Decoded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "- Center.......... {:.6}, {:.6}\n- Radial bounds... ±{:.6}, ±{:.6}\n- Metric bounds... ±{:.3}, ±{:.3}\n- Error radius.... {:.3}",
            self.latitude,
            self.longitude,
            self.lat_half_extent_deg,
            self.lon_half_extent_deg,
            self.lat_half_extent_m,
            self.lon_half_extent_m,
            self.error_radius_m
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Coordinates, de_interleave, interleave, meters_per_degree_lat, meters_per_degree_lon,
    };

    #[test]
    fn round_trip() {
        let c = Coordinates::new(1.286785, 103.854503);
        let code = c.encode(10);
        println!("Encoded as {code}");
        let decoded = Coordinates::<f64>::decode(&code);

        let abs_err_lat = (c.latitude - decoded.latitude).abs();
        let abs_err_lon = (c.longitude - decoded.longitude).abs();

        println!("err: {abs_err_lat:.6}, {:.6}", decoded.lat_half_extent_deg);
        println!("err: {abs_err_lon:.6}, {:.6}", decoded.lon_half_extent_deg);

        assert!(abs_err_lat <= decoded.lat_half_extent_deg, "Latitude error");
        assert!(
            abs_err_lon <= decoded.lon_half_extent_deg,
            "Longitude error"
        );
        assert!(decoded.lat_half_extent_deg <= 1e-3);
        assert!(decoded.lon_half_extent_deg <= 1e-3);
    }

    #[test]
    fn degrees_to_meters() {
        let lat: f64 = 45.0;

        let lat_meters = meters_per_degree_lat(lat.to_radians());
        let lon_meters = meters_per_degree_lon(lat.to_radians());

        let expected_lat = 111131.745;
        let expected_lon = 78846.80572069259;

        assert!(lat_meters - expected_lat <= 1e-9);
        assert!(lon_meters - expected_lon <= 1e-9);
    }

    #[test]
    fn morton_interleave_1() {
        let lat = 0b00;
        let lon = 0b111;
        let expected = 0b10101;
        let result = interleave(lat, lon);
        assert_eq!(result, expected, "Interleave error");
        let (delat, delon) = de_interleave(result);
        assert_eq!(delat, lat, "De-interleave error");
        assert_eq!(delon, lon, "De-interleave error");
    }

    #[test]
    fn morton_interleave_2() {
        let lat = 0b11;
        let lon = 0b000;
        let expected = 0b01010;
        let result = interleave(lat, lon);
        assert_eq!(result, expected, "Interleave error");
        let (delat, delon) = de_interleave(result);
        assert_eq!(delat, lat, "De-interleave error");
        assert_eq!(delon, lon, "De-interleave error");
    }

    #[test]
    fn normalization() {
        let c = Coordinates::new(36.719562, -4.450188);
        let normalized = c.normalize();
        let denormalized = normalized.denormalize();

        let err = ((c.latitude - denormalized.latitude).powi(2)
            + (c.longitude - denormalized.longitude).powi(2))
        .sqrt();

        assert!(err < 1e-9);
    }
}

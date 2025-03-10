use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Modes {
    Rgb,
    Ir,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Video {
    pub mode: Modes,
    pub device_rgb: i32,
    pub device_ir: i32,
}

impl Default for Video {
    fn default() -> Self {
        Self {
            mode: Modes::Ir,
            device_rgb: 0,
            device_ir: 2,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Detection {
    pub min_similarity_rgb: f64,
    pub min_similarity_ir: f64,
    pub min_brightness_rgb: f64,
    pub min_brightness_ir: f64,
    pub retries: u32,
}

impl Default for Detection {
    fn default() -> Self {
        Self {
            min_similarity_rgb: 0.7,
            min_similarity_ir: 0.9,
            min_brightness_rgb: 50.0,
            min_brightness_ir: 10.0,
            retries: 10,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub video: Video,
    pub detection: Detection,
}

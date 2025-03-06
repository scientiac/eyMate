use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub enum Modes {
    RGB,
    IR,
}

#[derive(Deserialize, Debug)]
pub struct Video {
    pub mode: Modes,
    pub device_rgb: i32,
    pub device_ir: i32,
}

#[derive(Deserialize, Debug)]
pub struct Detection {
    pub min_similarity_rgb: f64,
    pub min_similarity_ir: f64,
    pub min_brightness_ir: f64,
    pub min_brightness_rgb: f64,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub video: Video,
    pub detection: Detection,
}

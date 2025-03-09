use anyhow::{Result, *};
use figment::Figment;
use figment::providers::{Format, Toml};
use opencv::prelude::*;
use opencv::{core, highgui, imgproc, videoio};
use std::{fs, thread};
use std::{path::Path, time::Duration};
use tch::{CModule, Kind, Tensor};

use crate::config::*;
use crate::paths::*;

fn get_data_file(path: &Path, file: &str) -> Result<String> {
    let path = path.join(file);
    let path = path
        .to_str()
        .ok_or_else(|| anyhow!("Failed to convert path to string!"))?;
    let path = path.to_owned();

    Ok(path)
}

fn cosine_similarity(a: &Tensor, b: &Tensor) -> f64 {
    let a_flat = a.view([-1]); // Flatten to 1D
    let b_flat = b.view([-1]); // Flatten to 1D

    let dot_product = a_flat.dot(&b_flat).double_value(&[]);

    let norm_a = a_flat.norm().double_value(&[]);
    let norm_b = b_flat.norm().double_value(&[]);

    dot_product / (norm_a * norm_b)
}

fn process_image(image: &Mat, model: &CModule) -> Result<Tensor> {
    let size = core::Size::new(160, 160);
    let mut resized = Mat::default();
    imgproc::resize(image, &mut resized, size, 0.0, 0.0, imgproc::INTER_LINEAR)?;

    let data = resized.data_bytes()?;

    let tensor = Tensor::from_data_size(data, &[1, 3, 160, 160], Kind::Uint8).to_dtype(
        Kind::Float,
        false,
        true,
    );

    let embedding = model.forward_ts(&[tensor])?;

    Ok(embedding)
}

fn save_tensor(username: &str, filename: &str, tensor: &Tensor) -> Result<()> {
    let data_dir = get_data_dir().join("users").join(username);
    fs::create_dir_all(&data_dir)?;

    let path = get_data_file(&data_dir, filename)?;

    tensor.save(path)?;

    println!("Saved images for user: {}", username);
    Ok(())
}

fn load_tensor(username: &str, filename: &str) -> Result<Tensor> {
    let data_dir = get_data_dir().join("users").join(username);
    let path = get_data_file(&data_dir, filename)?;

    Ok(tch::Tensor::load(path)?)
}

#[allow(dead_code)]
pub fn cmd_add(config: Config, user: &str) -> Result<()> {
    let mut cam_rgb = videoio::VideoCapture::new(config.video.device_rgb, videoio::CAP_V4L2)?;
    let mut cam_ir = videoio::VideoCapture::new(config.video.device_ir, videoio::CAP_V4L2)?;

    let mut frame_rgb = Mat::default();
    let mut frame_ir = Mat::default();

    println!("Adding new user: {}", user);

    let data_dir = get_data_dir();
    let model = CModule::load(get_data_file(&data_dir, "vggface2.pt")?)?;

    let mut found = false;
    let mut brightness = 0.0;

    for _ in 0..config.detection.retries {
        cam_ir.read(&mut frame_ir)?;

        let brightness_vec = core::mean(&frame_ir, &core::no_array())?;
        brightness = brightness_vec.iter().sum::<f64>() / brightness_vec.len() as f64;

        if brightness < config.detection.min_brightness_ir {
            continue;
        } else {
            let embedding = process_image(&frame_ir, &model)?;
            save_tensor(user, "ir.bin", &embedding)?;
            found = true;
        }
    }
    if !found {
        return Err(anyhow!(
            "Failed ir image brightness too low with: {:.2}/{:.2}",
            brightness,
            config.detection.min_brightness_ir
        ));
    }

    found = false;

    for _ in 0..config.detection.retries {
        cam_rgb.read(&mut frame_rgb)?;

        let brightness_vec = core::mean(&frame_rgb, &core::no_array())?;
        brightness = brightness_vec.iter().sum::<f64>() / brightness_vec.len() as f64;

        if brightness < config.detection.min_brightness_rgb {
            continue;
        } else {
            let embedding = process_image(&frame_rgb, &model)?;
            save_tensor(user, "rgb.bin", &embedding)?;
            found = true;
        }
    }

    if !found {
        return Err(anyhow!(
            "Failed rgb image brightness too low with: {:.2}/{:.2}",
            brightness,
            config.detection.min_brightness_rgb
        ));
    }

    println!("Images saved for user: {}", user);

    Ok(())
}

#[allow(dead_code)]
pub fn cmd_test(config: Config, username: &str) -> Result<()> {
    let path = get_data_dir().join("users").join(username);

    if !Path::new(&path).exists() {
        return Err(anyhow!("User not found. Please register first."));
    }

    let path = match &config.video.mode {
        Modes::IR => "ir.bin",
        Modes::RGB => "rgb.bin",
    };

    let device = match &config.video.mode {
        Modes::IR => config.video.device_ir,
        Modes::RGB => config.video.device_rgb,
    };

    let min_similarity = match &config.video.mode {
        Modes::IR => config.detection.min_similarity_ir,
        Modes::RGB => config.detection.min_similarity_rgb,
    };

    let min_brightness = match &config.video.mode {
        Modes::IR => config.detection.min_brightness_ir,
        Modes::RGB => config.detection.min_brightness_rgb,
    };

    let data_dir = get_data_dir();
    let model = CModule::load(get_data_file(&data_dir, "vggface2.pt")?)?;

    let mut cam = videoio::VideoCapture::new(device, videoio::CAP_V4L2)?;
    let mut frame = Mat::default();

    let reference_embedding = load_tensor(username, path)?;

    while highgui::wait_key(1)? != 27 {
        cam.read(&mut frame)?;

        let brightness_vec = core::mean(&frame, &core::no_array())?;
        let brightness = brightness_vec.iter().sum::<f64>() / brightness_vec.len() as f64;

        let input_embedding = process_image(&frame, &model)?;

        let similarity = cosine_similarity(&reference_embedding, &input_embedding);
        println!(
            "Similarity: {:.3} {:.3}/{:.3}",
            brightness, similarity, min_similarity
        );

        highgui::imshow("Press ESC to exit!", &frame)?;

        if brightness < min_brightness {
            println!("Frame too dark!");
        } else if similarity > min_similarity {
            println!("Face matches!");
        } else {
            println!("Face does not match.");
        }
    }
    Ok(())
}

#[allow(dead_code)]
pub fn cmd_auth(username: &str) -> Result<bool> {
    let config_file = get_config_file()?;
    let config: Config = Figment::new().merge(Toml::file(config_file)).extract()?;

    let path = match &config.video.mode {
        Modes::IR => "ir.bin",
        Modes::RGB => "rgb.bin",
    };

    let device = match &config.video.mode {
        Modes::IR => config.video.device_ir,
        Modes::RGB => config.video.device_rgb,
    };

    let min_similarity = match &config.video.mode {
        Modes::IR => config.detection.min_similarity_ir,
        Modes::RGB => config.detection.min_similarity_rgb,
    };

    let min_brightness = match &config.video.mode {
        Modes::IR => config.detection.min_brightness_ir,
        Modes::RGB => config.detection.min_brightness_rgb,
    };

    let data_dir = get_data_dir();
    let model = CModule::load(get_data_file(&data_dir, "vggface2.pt")?)?;

    let mut cam = videoio::VideoCapture::new(device, videoio::CAP_V4L2)?;
    let mut frame = Mat::default();

    let reference_embedding = load_tensor(username, path)?;

    for _ in 0..config.detection.retries {
        cam.read(&mut frame)?;

        let brightness_vec = core::mean(&frame, &core::no_array())?;
        let brightness = brightness_vec.iter().sum::<f64>() / brightness_vec.len() as f64;

        let input_embedding = process_image(&frame, &model)?;

        let similarity = cosine_similarity(&reference_embedding, &input_embedding);

        // println!("Checking: {}/{} {}/{}", similarity, min_similarity, brightness, min_brightness);

        if similarity < min_similarity {
            // println!("Face does not match!");
            thread::sleep(Duration::from_millis(200));
            continue;
        } else if brightness < min_brightness {
            // println!("Frame too dark!");
            thread::sleep(Duration::from_millis(200));
            continue;
        } else {
            return Ok(true);
        }
    }

    // println!("Frame too dark or face not matching!");
    Ok(false)
}

use anyhow::{Result, *};
use opencv::prelude::*;
use opencv::{core, highgui, imgcodecs, imgproc, videoio};
use std::fs;
use std::path::PathBuf;
use std::{path::Path, thread::sleep, time::Duration};
use tch::{CModule, Kind, Tensor};

use crate::config::*;
use crate::paths::get_data_dir;

fn get_data_file(path: &PathBuf, file: &str) -> Result<String> {
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

fn save_images(username: &str, rgb: &Mat, ir: &Mat) -> Result<()> {
    let data_dir = get_data_dir().join(username);
    fs::create_dir_all(&data_dir)?;

    let rgb_path = get_data_file(&data_dir, "rgb.jpg")?;
    let ir_path = get_data_file(&data_dir, "ir.jpg")?;

    imgcodecs::imwrite(&rgb_path, rgb, &core::Vector::new())?;
    imgcodecs::imwrite(&ir_path, ir, &core::Vector::new())?;

    println!("Saved images for user: {}", username);
    Ok(())
}

pub fn cmd_add(config: Config, user: &str) -> Result<()> {
    let mut cam_rgb = videoio::VideoCapture::new(config.video.device_rgb, videoio::CAP_ANY)?;
    let mut cam_ir = videoio::VideoCapture::new(config.video.device_ir, videoio::CAP_ANY)?;

    let mut frame_rgb = Mat::default();
    let mut frame_ir = Mat::default();

    println!("Adding new user: {}", user);

    cam_ir.grab()?;
    cam_rgb.grab()?;

    sleep(Duration::from_secs(2));

    cam_ir.read(&mut frame_ir)?;
    cam_rgb.read(&mut frame_rgb)?;

    let brightness_vec = core::mean(&frame_ir, &core::no_array())?;
    let brightness = brightness_vec.iter().sum::<f64>() / brightness_vec.len() as f64;

    save_images(user, &frame_rgb, &frame_ir)?;

    if brightness < config.detection.min_brightness_ir {
        return Err(anyhow!(
            "Failed ir image brightness too low with: {:.2}/{:.2}",
            brightness,
            config.detection.min_brightness_ir
        ));
    }

    let brightness_vec = core::mean(&frame_rgb, &core::no_array())?;
    let brightness = brightness_vec.iter().sum::<f64>() / brightness_vec.len() as f64;

    if brightness < config.detection.min_brightness_rgb {
        return Err(anyhow!(
            "Failed rgb image brightness too low with: {:.2}/{:.2}",
            brightness,
            config.detection.min_brightness_rgb
        ));
    }
    println!("Images saved for user: {}", user);

    Ok(())
}

pub fn cmd_test(config: Config, username: &str) -> Result<()> {
    let data_dir = get_data_dir().join(username);

    let path = match &config.video.mode {
        Modes::IR => get_data_file(&data_dir, "ir.jpg")?,
        Modes::RGB => get_data_file(&data_dir, "rgb.jpg")?,
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

    if !Path::new(&path).exists() {
        return Err(anyhow!("User not found. Please register first."));
    }

    let model = CModule::load("./vggface2.pt")?;

    let mut cam = videoio::VideoCapture::new(device, videoio::CAP_ANY)?;
    let mut frame = Mat::default();

    let reference = imgcodecs::imread(&path, imgcodecs::IMREAD_COLOR)?;
    let reference_embedding = process_image(&reference, &model)?;

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

        highgui::imshow("Press <ESC> to exit!", &frame)?;

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

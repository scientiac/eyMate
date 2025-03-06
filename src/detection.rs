use anyhow::{Result, *};
use opencv::prelude::*;
use opencv::{core, highgui, imgcodecs, imgproc, videoio};
use std::{fs, path::Path, thread::sleep, time::Duration};
use tch::{CModule, Device, Kind, Tensor};

use crate::config::*;

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
    let user_dir = format!("users/{}", username);
    fs::create_dir_all(&user_dir)?;

    let rgb_path = format!("{}/rgb.jpg", user_dir);
    let ir_path = format!("{}/ir.jpg", user_dir);

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

    cam_ir.read(&mut frame_ir)?;

    cam_rgb.grab()?;

    sleep(Duration::from_secs(2));

    cam_rgb.read(&mut frame_rgb)?;

    save_images(user, &frame_rgb, &frame_ir)?;
    println!("Images saved for user: {}", user);

    Ok(())
}

pub fn cmd_test(config: Config, user: &str) -> Result<()> {
    let path = match &config.video.mode {
        Modes::IR => format!("users/{}/ir.jpg", user),
        Modes::RGB => format!("users/{}/rgb.jpg", user),
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

        let mut img = Mat::default();
        imgproc::cvt_color_def(&frame, &mut img, imgproc::COLOR_BGR2GRAY)?;

        let brightness_array = core::mean(&img, &core::no_array())?;
        let brightness = (brightness_array[0] + brightness_array[1] + brightness_array[2]) / 3.0;

        let input_embedding = process_image(&frame, &model)?;

        let similarity = cosine_similarity(&reference_embedding, &input_embedding);

        // println!(
        //     "Similarity: {:.3} {:.3}/{:.3}",
        //     brightness, similarity, min_similarity
        // );

        if brightness < min_brightness {
            println!("Frame too dark!");
        } else if similarity > min_similarity {
            println!("Face matches!");
        } else {
            println!("Face does not match.");
        }

        highgui::imshow("Camera", &frame)?;
    }
    Ok(())
}

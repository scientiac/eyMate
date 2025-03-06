use anyhow::{Result, *};
use clap::Parser;
use opencv::prelude::*;
use opencv::{core, highgui, imgcodecs, imgproc, videoio};
use std::{fs, path::Path, thread::sleep, time::Duration};
use tch::{CModule, Device, Kind, Tensor};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// User to be looged in
    #[arg(short, long, default_value_t = String::from(""))]
    user: String,
    /// Add a new user
    #[arg(short, long, default_value_t = String::from(""))]
    add: String,
}

fn cosine_similarity(a: &Tensor, b: &Tensor) -> f64 {
    let a_flat = a.view([-1]); // Flatten to 1D
    let b_flat = b.view([-1]); // Flatten to 1D

    let dot_product = a_flat.dot(&b_flat).double_value(&[]);

    let norm_a = a_flat.norm().double_value(&[]);
    let norm_b = b_flat.norm().double_value(&[]);

    dot_product / (norm_a * norm_b)
}

fn preprocess_image(image: &Mat) -> Result<Tensor> {
    let size = core::Size::new(160, 160);
    let mut resized = Mat::default();
    imgproc::resize(image, &mut resized, size, 0.0, 0.0, imgproc::INTER_LINEAR)?;

    let data = resized.data_bytes()?;

    Ok(Tensor::from_data_size(data, &[1, 3, 160, 160], Kind::Uint8)
        .to_dtype(Kind::Float, false, true)
        .to_device(Device::Cpu))
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

fn main() -> Result<()> {
    let model = CModule::load("./vggface2.pt").expect("Failed to load model");

    let mut cam_rgb = videoio::VideoCapture::new(0, videoio::CAP_ANY)?;
    let mut cam_ir = videoio::VideoCapture::new(2, videoio::CAP_ANY)?;

    let mut frame_rgb = Mat::default();
    let mut frame_ir = Mat::default();

    let args = Args::parse();

    let username = args.user.as_str();

    if !args.add.is_empty() {
        let new_user = args.add.as_str();

        println!("Adding new user: {}", args.add);

        cam_ir.read(&mut frame_ir)?;

        cam_rgb.grab()?;

        sleep(Duration::from_secs(2));

        cam_rgb.read(&mut frame_rgb)?;

        save_images(new_user, &frame_rgb, &frame_ir)?;
        if !Path::new(&format!("users/{}/rgb.jpg", new_user)).exists()
            || !Path::new(&format!("users/{}/ir.jpg", new_user)).exists()
        {
            return Err(anyhow!("Failed to save images. Please try again."));
        }
        println!("Images saved for user: {}", new_user);
        return Ok(());
    }

    let user_dir = format!("users/{}/", username);
    let rgb_path = format!("{}rgb.jpg", user_dir);
    let ir_path = format!("{}ir.jpg", user_dir);

    if !Path::new(&rgb_path).exists() || !Path::new(&ir_path).exists() {
        return Err(anyhow!("User not found. Please register first."));
    }

    let reference_rgb = imgcodecs::imread(&rgb_path, imgcodecs::IMREAD_COLOR)?;
    let reference_ir = imgcodecs::imread(&ir_path, imgcodecs::IMREAD_COLOR)?;

    let reference_rgb_tensor = preprocess_image(&reference_rgb)?;
    let reference_ir_tensor = preprocess_image(&reference_ir)?;

    let reference_rgb_embedding = model.forward_ts(&[reference_rgb_tensor])?;
    let reference_ir_embedding = model.forward_ts(&[reference_ir_tensor])?;

    while highgui::wait_key(1)? != 27 {
        cam_rgb.read(&mut frame_rgb)?;
        cam_ir.read(&mut frame_ir)?;

        let input_rgb_tensor = preprocess_image(&frame_rgb)?;
        let input_ir_tensor = preprocess_image(&frame_ir)?;

        let input_rgb_embedding = model.forward_ts(&[input_rgb_tensor])?;

        println!("TENSOR: {:?}", input_rgb_embedding);

        let input_ir_embedding = model.forward_ts(&[input_ir_tensor])?;

        let similarity = cosine_similarity(&reference_ir_embedding, &input_ir_embedding);
        let similarity_rgb = cosine_similarity(&reference_rgb_embedding, &input_rgb_embedding);

        println!("Similarity: {:.3} {:.3}", similarity, similarity_rgb);

        if similarity > 0.8 {
            println!("Face matches!");
        } else {
            println!("Face does not match.");
        }

        highgui::imshow("RGB Camera", &frame_rgb)?;
        highgui::imshow("IR Camera", &frame_ir)?;
    }

    Ok(())
}

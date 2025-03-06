use clap::Parser;
use opencv::{core, highgui, imgcodecs, imgproc, prelude::*, videoio};
use std::fs;
use std::path::Path;
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

fn preprocess_image(image: &Mat) -> Tensor {
    let size = core::Size::new(160, 160);
    let mut resized = Mat::default();
    imgproc::resize(image, &mut resized, size, 0.0, 0.0, imgproc::INTER_LINEAR).unwrap();

    let data = resized.data_bytes().unwrap();
    Tensor::from_data_size(data, &[1, 3, 160, 160], Kind::Uint8)
        .to_dtype(Kind::Float, false, true)
        .to_device(Device::Cpu)
}

fn preprocess_ir_image(image: &Mat) -> Tensor {
    let mut gray = Mat::default();
    imgproc::cvt_color(
        image,
        &mut gray,
        imgproc::COLOR_BGR2GRAY,
        0,
        opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT,
    )
    .unwrap();

    let size = core::Size::new(160, 160);
    let mut resized = Mat::default();
    imgproc::resize(&gray, &mut resized, size, 0.0, 0.0, imgproc::INTER_LINEAR).unwrap();

    let data = resized.data_bytes().unwrap();
    Tensor::from_data_size(data, &[1, 1, 160, 160], Kind::Uint8)
        .to_dtype(Kind::Float, false, true)
        .to_device(Device::Cpu)
}

fn combine_embeddings(rgb_embedding: &Tensor, ir_embedding: &Tensor) -> Tensor {
    Tensor::cat(&[rgb_embedding, ir_embedding], 1)
}

fn save_images(username: &str, rgb: &Mat, ir: &Mat) {
    let user_dir = format!("users/{}", username);
    fs::create_dir_all(&user_dir).unwrap();

    let rgb_path = format!("{}/rgb.jpg", user_dir);
    let ir_path = format!("{}/ir.jpg", user_dir);

    imgcodecs::imwrite(&rgb_path, rgb, &core::Vector::new()).unwrap();
    imgcodecs::imwrite(&ir_path, ir, &core::Vector::new()).unwrap();

    println!("Saved images for user: {}", username);
}

fn main() {
    let model = CModule::load("./vggface2.pt").expect("Failed to load model");

    let mut cam_rgb = videoio::VideoCapture::new(0, videoio::CAP_ANY).unwrap();
    let mut cam_ir = videoio::VideoCapture::new(2, videoio::CAP_ANY).unwrap();

    let mut frame_rgb = Mat::default();
    let mut frame_ir = Mat::default();

    let args = Args::parse();

    let username = args.user.as_str();

    if args.add != "" {
        println!("Adding new user: {}", args.add);
        let new_user = args.add.as_str();
        cam_rgb.read(&mut frame_rgb).unwrap();
        cam_ir.read(&mut frame_ir).unwrap();
        save_images(new_user, &frame_rgb, &frame_ir);
        if !Path::new(&format!("users/{}/rgb.jpg", new_user)).exists()
            || !Path::new(&format!("users/{}/ir.jpg", new_user)).exists()
        {
            println!("Failed to save images. Please try again.");
            return;
        }
        println!("Images saved for user: {}", new_user);
        return;
    }

    let user_dir = format!("users/{}/", username);
    let rgb_path = format!("{}rgb.jpg", user_dir);
    let ir_path = format!("{}ir.jpg", user_dir);

    if !Path::new(&rgb_path).exists() || !Path::new(&ir_path).exists() {
        println!("User not found. Please register first.");
        return;
    }

    let reference_rgb = imgcodecs::imread(&rgb_path, imgcodecs::IMREAD_COLOR).unwrap();
    let reference_ir = imgcodecs::imread(&ir_path, imgcodecs::IMREAD_GRAYSCALE).unwrap();

    let reference_rgb_tensor = preprocess_image(&reference_rgb);
    let reference_ir_tensor = preprocess_ir_image(&reference_ir);

    let reference_rgb_embedding = model.forward_ts(&[reference_rgb_tensor]).unwrap();
    let reference_ir_embedding = model.forward_ts(&[reference_ir_tensor]).unwrap();

    let reference_embedding = combine_embeddings(&reference_rgb_embedding, &reference_ir_embedding);

    while highgui::wait_key(1).unwrap() != 27 {
        cam_rgb.read(&mut frame_rgb).unwrap();
        cam_ir.read(&mut frame_ir).unwrap();

        let input_rgb_tensor = preprocess_image(&frame_rgb);
        let input_ir_tensor = preprocess_ir_image(&frame_ir);

        let input_rgb_embedding = model.forward_ts(&[input_rgb_tensor]).unwrap();
        let input_ir_embedding = model.forward_ts(&[input_ir_tensor]).unwrap();

        let input_embedding = combine_embeddings(&input_rgb_embedding, &input_ir_embedding);

        let dot_product = reference_embedding.dot(&input_embedding).double_value(&[]);
        let norm_ref = reference_embedding.norm().double_value(&[]);
        let norm_input = input_embedding.norm().double_value(&[]);
        let similarity = dot_product / (norm_ref * norm_input);

        println!("Similarity: {:.3}", similarity);

        if similarity > 0.8 {
            println!("Face matches!");
        } else {
            println!("Face does not match.");
        }

        //highgui::imshow("RGB Camera", &frame_rgb).unwrap();
        highgui::imshow("IR Camera", &frame_ir).unwrap();
    }
}

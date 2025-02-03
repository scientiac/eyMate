use tch::{CModule, Tensor, Kind};
use opencv::{prelude::*, videoio, imgproc, highgui, core};

fn preprocess_image(image: &Mat) -> Tensor {
    let size = core::Size::new(160, 160);
    let mut resized = Mat::default();
    imgproc::resize(image, &mut resized, size, 0.0, 0.0, imgproc::INTER_LINEAR).unwrap();
    
    let data = resized.data_bytes().unwrap();
    let tensor = Tensor::from_data_size(data, &[1, 3, 160, 160], Kind::Uint8)
        .to_dtype(Kind::Float, false, true) // Fixed argument count
        .to_device(tch::Device::Cpu);
    tensor
}

fn cosine_similarity(a: &Tensor, b: &Tensor) -> f64 {
    let dot_product = a.dot(b).double_value(&[]);
    let norm_a = a.norm().double_value(&[]);
    let norm_b = b.norm().double_value(&[]);
    dot_product / (norm_a * norm_b)
}

fn main() {
    let model = CModule::load("./20180402-114759-vggface2.pt").expect("Failed to load model");
    let reference_image = opencv::imgcodecs::imread("reference.jpg", opencv::imgcodecs::IMREAD_COLOR).unwrap();
    let reference_tensor = preprocess_image(&reference_image);
    let reference_embedding = model.forward_ts(&[reference_tensor]).unwrap();
    
    let cam_index = 0;
    let mut cam = videoio::VideoCapture::new(cam_index, videoio::CAP_ANY).unwrap();
    let mut frame = Mat::default();
    
    while highgui::wait_key(1).unwrap() != 27 {
        cam.read(&mut frame).unwrap();
        let input_tensor = preprocess_image(&frame);
        let input_embedding = model.forward_ts(&[input_tensor]).unwrap();
        let similarity = cosine_similarity(&reference_embedding, &input_embedding);
        println!("Similarity: {:.3}", similarity);
        if similarity > 0.8 {
            println!("Face matches!");
        } else {
            println!("Face does not match.");
        }
        highgui::imshow("Camera", &frame).unwrap();
    }
}

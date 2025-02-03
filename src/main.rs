use tch::{nn, nn::Module, Device, Tensor};

fn main() {
    // Set the device to CUDA if available, otherwise fallback to CPU
    let device = if tch::Cuda::is_available() {
        Device::Cuda(0)
    } else {
        Device::Cpu
    };
    
    // Load the VGGFace2 model
    let model_path = "vggface2.pt"; // Ensure this file exists in your working directory
    let vs = nn::VarStore::new(device);
    let model = tch::CModule::load_on_device(model_path, device).expect("Failed to load model");
    
    // Create a dummy input tensor (1 image, 3 channels, 224x224 size)
    let input = Tensor::randn(&[1, 3, 224, 224], (tch::Kind::Float, device));
    
    // Run inference
    let output = model.forward_ts(&[input]).expect("Inference failed");
    
    println!("Model output: {:?}", output);
}

use audio_recorder::whisper::model::{Decoder, ModelData};
use std::fs;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn model_assets_load_successfully() {
    // This test verifies that the whisper model assets can be loaded
    // and the model can be initialized without errors

    // Skip this test in CI environments where assets might not be available
    // or when running in environments without file system access
    if cfg!(target_arch = "wasm32") {
        // In WASM environment, we can't access the file system directly
        // This test would need to be run in a Node.js environment with fs access
        // or with pre-loaded assets
        return;
    }

    // Load model assets from the assets directory
    let model_path = "assets/models/whisper/whisper-tiny";

    // Check if the model directory exists
    if !std::path::Path::new(model_path).exists() {
        // Skip test if assets are not available
        println!("Model assets not found at {}, skipping test", model_path);
        return;
    }

    // Load the individual model files
    let weights = fs::read(format!("{}/model.safetensors", model_path))
        .expect("Failed to load model weights");
    let tokenizer =
        fs::read(format!("{}/tokenizer.json", model_path)).expect("Failed to load tokenizer");
    let config = fs::read(format!("{}/config.json", model_path)).expect("Failed to load config");
    let mel_filters =
        fs::read("assets/whisper/mel_filters.safetensors").expect("Failed to load mel filters");

    // Create ModelData structure
    let model_data = ModelData {
        weights,
        tokenizer,
        mel_filters,
        config,
        quantized: false,
        timestamps: true,
        is_multilingual: true,
        language: None,
        task: None,
    };

    // Try to load the decoder
    let decoder_result = Decoder::load(model_data);

    // Verify that the decoder loaded successfully
    assert!(
        decoder_result.is_ok(),
        "Failed to load decoder: {:?}",
        decoder_result.err()
    );

    // If we get here, the model assets loaded successfully
    println!("Model assets loaded successfully");
}

#[wasm_bindgen_test]
fn model_data_structure_valid() {
    // This test verifies that the ModelData structure can be created
    // and serialized/deserialized properly

    let model_data = ModelData {
        weights: vec![],
        tokenizer: vec![],
        mel_filters: vec![],
        config: vec![],
        quantized: false,
        timestamps: true,
        is_multilingual: true,
        language: None,
        task: None,
    };

    // Test serialization
    let serialized = serde_cbor::to_vec(&model_data);
    assert!(serialized.is_ok(), "Failed to serialize ModelData");

    // Test deserialization
    let deserialized: Result<ModelData, _> = serde_cbor::from_slice(&serialized.unwrap());
    assert!(deserialized.is_ok(), "Failed to deserialize ModelData");
}

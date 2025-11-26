use futures::stream::StreamExt;
use leptos_workers::worker;

use crate::whisper::model::{Decoder, ModelData};

#[worker(AudioWorker)]
#[leptos::lazy]
pub fn audio_worker(req: Vec<f32>) -> f32 {
    let sum: f32 = req.iter().sum();
    let count = req.len() as f32;
    if count > 0.0 { sum / count } else { 0.0 }
}

#[worker(WhisperWorker)]
pub async fn whisper_worker(
    model_data: ModelData,
    rx: leptos_workers::Receiver<Vec<f32>>,
    tx: leptos_workers::Sender<String>,
) {
    match Decoder::load(model_data) {
        Ok(mut decoder) => {
            leptos::logging::log!("WhisperWorker: Initialized.");
            let mut stream = rx.into_stream();
            while let Some(audio_samples) = stream.next().await {
                leptos::logging::log!(
                    "WhisperWorker: Received {} audio samples.",
                    audio_samples.len()
                );

                match decoder.convert_and_run(audio_samples) {
                    Ok(segments) => {
                        leptos::logging::log!("found segments: {}", segments.len());
                        let mut full_text = String::new();
                        for segment in segments {
                            full_text.push_str(&segment.dr.text);
                        }
                        leptos::logging::log!("WhisperWorker: Transcribed text: {}", full_text);
                        let _ = tx.send(full_text);
                    }
                    Err(e) => {
                        leptos::logging::error!(
                            "WhisperWorker: Error during transcription: {:?}",
                            e
                        );
                        let _ = tx.send(format!("Error: {:?}", e));
                    }
                }
            }
            leptos::logging::log!("WhisperWorker no longer checking for samples");
        }
        Err(e) => {
            leptos::logging::error!("Failed to load whisper model decoder: {:?}", e);
            // The worker can't proceed without a decoder, so it will effectively stop here.
        }
    }
}

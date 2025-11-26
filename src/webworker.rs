use futures::stream::StreamExt;
use leptos_workers::worker;
use reqwest::Client;

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
    origin: String,
    rx: leptos_workers::Receiver<Vec<f32>>,
    tx: leptos_workers::Sender<String>,
) {
    let client = Client::new();
    let url = format!("{origin}/api/fetch_model_data");
    leptos::logging::log!("WhisperWorker: Fetching model data...");
    if let Ok(resp) = client.post(&url).send().await
        && let Ok(bytes) = resp.bytes().await
        && let Ok(model_data) = serde_cbor::from_slice::<ModelData>(&bytes)
        && let Ok(mut decoder) = Decoder::load(model_data)
    {
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
                    leptos::logging::log!("WhisperWorker: Transcribed text: {full_text}");
                    let _ = tx.send(full_text);
                }
                Err(e) => {
                    leptos::logging::error!("WhisperWorker: Error during transcription: {e:?}");
                    let _ = tx.send(format!("Error: {e:?}"));
                }
            }
        }
        leptos::logging::log!("WhisperWorker no longer checking for samples");
    } else {
        leptos::logging::error!("Failed to load whisper model decoder");
        // The worker can't proceed without a decoder, so it will effectively stop here.
    }
}

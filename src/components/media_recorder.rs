use std::cell::OnceCell;

use futures::stream::StreamExt;
use leptos::prelude::*;
use serde::Serialize;
use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use web_sys::{
    AudioContext, Blob, BlobEvent, Event, FileReader, MediaRecorder, MediaStream,
    MediaStreamConstraints,
};

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AudioConstraints {
    sample_rate: usize,
    channel_count: u32,
    audio: bool,
}

impl Default for AudioConstraints {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            channel_count: 1,
            audio: true,
        }
    }
}

#[component]
pub fn MediaRecorderComponent() -> impl IntoView {
    use crate::webworker::audio_worker;
    use crate::{webworker::whisper_worker, whisper::util::fetch_model_data};

    let (is_recording, set_is_recording) = signal(false);
    let (samples, set_samples) = signal(vec![]);
    let (transcribed_text, set_transcribed_text) = signal(String::new());
    let (is_model_enabled, set_is_model_enabled) = signal(false);
    let (last_audio_url, set_last_audio_url) = signal(String::new());

    // Resource to fetch model data AND create the worker.
    // Returns Option<(Sender, Receiver)>
    let worker_resource = LocalResource::new(move || async move {
        if is_model_enabled.get()
            && let Ok(md) = fetch_model_data().await
            && let Ok((tx, rx)) = whisper_worker(md)
        {
            leptos::logging::log!("worker resource ready");
            Some((tx, rx))
        } else {
            None
        }
    });

    // effect to spawn worker and process received text
    Effect::new(move || {
        // Check if worker is ready
        if let Some(Some((_tx, rx))) = worker_resource.get() {
            leptos::logging::log!("worker spawned and ready to process");
            leptos::task::spawn_local(async move {
                let mut message_stream = rx.into_stream();
                while let Some(text) = message_stream.next().await {
                    leptos::logging::log!("Received transcription: {text}");
                    set_transcribed_text.set(text);
                }
                leptos::logging::log!("message stream loop ended");
            });
        } else {
            leptos::logging::warn!("worker not ready to process");
        }
    });

    // effect to send audio to worker
    Effect::new(move || {
        let s = samples.get();
        leptos::logging::log!("received samples: {}", s.len());
        if !s.is_empty() {
            // Use the resource directly to get the sender
            if let Some(Some((tx, _rx))) = worker_resource.get() {
                if let Err(e) = tx.send(s) {
                    leptos::logging::warn!("send error: {e:?}");
                }
            } else {
                leptos::logging::warn!("unable to get worker resource");
            }
        }
    });

    // Create a LocalResource that sends samples to the worker when they change
    let average_sample = LocalResource::new(move || {
        let data = samples.get();
        audio_worker(data)
    });

    let transcribed_text_resource = LocalResource::new(move || {
        let text = transcribed_text.get();
        async move { text }
    });

    // Store the MediaRecorder and Closure to keep them alive during recording
    // Use new_local because these JS objects are !Send and !Sync
    let recorder_stored = StoredValue::new_local(None::<MediaRecorder>);
    let on_data_closure_stored = StoredValue::new_local(None::<Closure<dyn FnMut(BlobEvent)>>);
    let on_stop_closure_stored = StoredValue::new_local(None::<Closure<dyn FnMut(Event)>>);

    // Store collected blobs
    let chunks_stored = StoredValue::new_local(Vec::<Blob>::new());

    // Store AudioContext using OnceCell for lazy initialization
    let audio_context_stored = StoredValue::new_local(OnceCell::<AudioContext>::new());

    let process_audio = move |blob: Blob| {
        // Create object URL for playback
        if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
            let old_url = last_audio_url.get_untracked();
            if !old_url.is_empty() {
                let _ = web_sys::Url::revoke_object_url(&old_url);
            }
            set_last_audio_url.set(url);
        } else {
            leptos::logging::warn!("Failed to create object URL for audio blob.");
        }

        leptos::task::spawn_local(async move {
            // Retrieve or create the AudioContext lazily
            let audio_context = audio_context_stored.with_value(|cell| {
                cell.get_or_init(|| AudioContext::new().expect("Failed to create AudioContext"))
                    .clone()
            });

            // Resume context if it's suspended (common browser policy requirement)
            if audio_context.state() == web_sys::AudioContextState::Suspended
                && let Ok(promise) = audio_context.resume()
            {
                let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
            }

            let file_reader = FileReader::new().expect("Failed to create FileReader");

            let reader_promise = js_sys::Promise::new(&mut |resolve, reject| {
                // file_reader.set_onload - sets up the file reader
                let closure = Closure::once(move |e: web_sys::ProgressEvent| {
                    if let Some(target) = e.target()
                        && let Ok(result) = target.unchecked_into::<FileReader>().result()
                        && let Ok(res) = resolve.call1(&JsValue::NULL, &result)
                    {
                        leptos::logging::log!("resolve call working: {res:?}");
                    }
                });
                file_reader.set_onload(Some(closure.as_ref().unchecked_ref()));

                let on_error_closure = Closure::once(move |e: web_sys::ProgressEvent| {
                    leptos::logging::error!("FileReader error: {e:?}");
                    let _ = reject.call1(&JsValue::NULL, &e);
                });
                file_reader.set_onerror(Some(on_error_closure.as_ref().unchecked_ref()));

                match file_reader.read_as_array_buffer(&blob) {
                    Ok(_) => {}
                    Err(e) => leptos::logging::warn!("blob reader error: {e:?}"),
                };
                closure.forget(); // Keep it alive until called
                on_error_closure.forget();
            });

            // Get the new samples average from the reader so we can turn webm audio into wav audio
            let new_samples = if let Ok(array_buffer_val) =
                wasm_bindgen_futures::JsFuture::from(reader_promise).await
                && let array_buffer = array_buffer_val.unchecked_into::<js_sys::ArrayBuffer>()
                && let Ok(decoded_promise) = audio_context.decode_audio_data(&array_buffer)
                && let Ok(audio_buffer) =
                    wasm_bindgen_futures::JsFuture::from(decoded_promise).await
                && let audio_buffer = audio_buffer.unchecked_into::<web_sys::AudioBuffer>()
                && let Ok(float_data) = audio_buffer.get_channel_data(0)
            {
                float_data
            } else {
                vec![]
            };
            set_samples.set(new_samples);
        });
    };

    let start_recording = move || {
        let navigator = window().navigator();
        set_samples.set(vec![]);
        chunks_stored.update_value(|c| c.clear());

        leptos::task::spawn_local(async move {
            let audio_constraints = serde_wasm_bindgen::to_value(&AudioConstraints::default())
                .expect("create constraints failed");

            let constraints = MediaStreamConstraints::new();
            constraints.set_audio(&audio_constraints);

            // Create all the handlers for the recorder
            if let Ok(media_devices) = navigator.media_devices()
                && let Ok(promise) = media_devices.get_user_media_with_constraints(&constraints)
                && let Ok(js_stream) = wasm_bindgen_futures::JsFuture::from(promise).await
                && let stream = MediaStream::from(js_stream)
                && let Ok(recorder) = MediaRecorder::new_with_media_stream(&stream)
            {
                leptos::logging::log!("creating handlers for recording");
                let on_data_handler = Closure::new(move |event: BlobEvent| {
                    if let Some(blob) = event.data() {
                        leptos::logging::log!("updating chunks stored");
                        chunks_stored.update_value(|c| c.push(blob));
                    }
                });

                let on_stop_handler = Closure::new(move |_event: Event| {
                    set_is_recording.set(false);

                    // Gather all blobs
                    let blobs = chunks_stored.get_value();
                    let js_array = js_sys::Array::new();
                    for blob in blobs {
                        js_array.push(&blob);
                    }

                    // Create a single blob from chunks
                    let blob_options = web_sys::BlobPropertyBag::new();
                    blob_options.set_type("audio/webm");

                    if let Ok(final_blob) =
                        Blob::new_with_blob_sequence_and_options(&js_array, &blob_options)
                    {
                        process_audio(final_blob);
                    }
                });

                recorder.set_ondataavailable(Some(on_data_handler.as_ref().unchecked_ref()));
                recorder.set_onstop(Some(on_stop_handler.as_ref().unchecked_ref()));

                // Store references
                on_data_closure_stored.set_value(Some(on_data_handler));
                on_stop_closure_stored.set_value(Some(on_stop_handler));
                recorder_stored.set_value(Some(recorder.clone()));

                match recorder.start() {
                    Ok(_) => set_is_recording.set(true),
                    Err(e) => leptos::logging::warn!("recording start error: {e:?}"),
                }
            }
        });
    };

    let stop_recording = move || {
        if let Some(recorder) = recorder_stored.get_value()
            && is_recording.get()
        {
            let _ = recorder.stop();
        }
        // Do NOT cleanup closures here; let onstop handle the flow and let closures persist
        // until the next start_recording overwrites them or the component unmounts.
    };

    view! {
        <div class="media-recorder">
            <h2>"Audio Recorder"</h2>

            <div class="model-controls" style="margin-bottom: 1rem;">
                <button
                    on:click=move |_| set_is_model_enabled.set(true)
                    disabled=move || is_model_enabled.get()
                >
                    {move || if is_model_enabled.get() {
                        if worker_resource.get().flatten().is_some() {
                            "Model Loaded"
                        } else {
                            "Loading Model..."
                        }
                    } else {
                        "Load Whisper Model"
                    }}
                </button>
            </div>

            <button
                on:mousedown=move |_| start_recording()
                on:mouseup=move |_| stop_recording()
                on:touchstart=move |_| start_recording()
                on:touchend=move |_| stop_recording()
                class="record-button"
                class:recording=is_recording
            >
                {move || if is_recording.get() {
                    "Recording... (Release to stop)"
                } else {
                    "Press and hold to record"
                }}
            </button>

            <Show when=move || true>
                <div class="worker-result">
                    <h3>"Worker Calculation (Average):"</h3>
                    <Suspense fallback=move || view! { "Loading..." }>
                        {move || {
                            match average_sample.get() {
                                Some(Ok(res)) => format!("{res}"),
                                Some(Err(e)) => format!("{e}"),
                                None => "not sure what this is".to_string(),
                            }
                        }}
                    </Suspense>
                </div>
            </Show>

            <Show when=move || !last_audio_url.get().is_empty()>
                <div class="audio-player">
                    <h3>"Last Recording:"</h3>
                    <audio controls src=move || last_audio_url.get() />
                </div>
            </Show>

            <div class="transcription-result">
                <h3>Transcription Result</h3>
                <Suspense fallback=move || view! { "Loading..." }>
                    {move || transcribed_text_resource.get()}
                </Suspense>
            </div>
        </div>
    }
}

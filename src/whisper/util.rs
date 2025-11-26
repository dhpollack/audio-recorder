use super::model::ModelData;
use leptos::prelude::*;
use leptos::server_fn::codec::Cbor;

#[cfg(feature = "ssr")]
use send_wrapper::SendWrapper;
#[cfg(feature = "ssr")]
use worker::Bucket;

pub const MODEL_KEY: &str = "whisper/whisper-tiny/model.safetensors";
pub const TOKENIZER_KEY: &str = "whisper/whisper-tiny/tokenizer.json";
pub const MEL_KEY: &str = "whisper/mel_filters.safetensors";
pub const CONFIG_KEY: &str = "whisper/whisper-tiny/config.json";

#[cfg(feature = "ssr")]
fn get_key_from_bucket(
    bucket: &SendWrapper<Bucket>,
    key: &str,
) -> impl Future<Output = Result<Vec<u8>, ServerFnError>> + Send {
    let bucket_clone = bucket.clone();
    let key_string = key.to_string();
    SendWrapper::new(async move {
        let execute_future = SendWrapper::new((*bucket_clone).get(&key_string).execute());
        let obj_option = execute_future.await?;

        if let Some(obj_val) = obj_option {
            let body_option = obj_val.body();
            if let Some(body_val) = body_option {
                let bytes_future = SendWrapper::new(body_val.bytes());
                bytes_future.await
            } else {
                leptos::logging::warn!("object body not found: {key}");
                Ok(vec![])
            }
        } else {
            leptos::logging::warn!("object not found: {key}");
            Ok(vec![])
        }
        .map_err(|e| e.into())
    })
}

#[server(endpoint = "fetch_model_data", output = Cbor)]
pub async fn fetch_model_data() -> Result<ModelData, ServerFnError> {
    use axum::Extension;
    use std::sync::Arc;
    use worker::Env;

    let Extension::<Arc<Env>>(env) = leptos_axum::extract().await?;
    let r2_bucket = env
        .bucket("audio-recorder-bucket")
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let r2 = SendWrapper::new(r2_bucket);
    let model_bytes = get_key_from_bucket(&r2, MODEL_KEY).await?;
    let tokenizer_bytes = get_key_from_bucket(&r2, TOKENIZER_KEY).await?;
    let mel_filters_bytes = get_key_from_bucket(&r2, MEL_KEY).await?;
    let config_bytes = get_key_from_bucket(&r2, CONFIG_KEY).await?;

    leptos::logging::log!(
        "Fetched model data. Weights: {} bytes, Tokenizer: {} bytes, Mel: {} bytes, Config: {} bytes",
        model_bytes.len(),
        tokenizer_bytes.len(),
        mel_filters_bytes.len(),
        config_bytes.len()
    );

    if model_bytes.is_empty() {
        leptos::logging::error!("Model weights are empty!");
        return Err(ServerFnError::new("Model weights are empty"));
    }
    if mel_filters_bytes.is_empty() {
        leptos::logging::error!("Mel filters are empty!");
        return Err(ServerFnError::new("Mel filters are empty"));
    }

    Ok(ModelData {
        weights: model_bytes,
        tokenizer: tokenizer_bytes,
        mel_filters: mel_filters_bytes,
        config: config_bytes,
        quantized: false,
        timestamps: true,
        is_multilingual: true,
        language: None,
        task: Some("transcribe".to_string()),
    })
}

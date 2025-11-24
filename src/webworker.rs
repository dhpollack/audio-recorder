use leptos_workers::worker;

#[worker(AudioWorker)]
#[leptos::lazy]
pub fn audio_worker(req: Vec<f32>) -> f32 {
    let sum: f32 = req.iter().sum();
    let count = req.len() as f32;
    if count > 0.0 { sum / count } else { 0.0 }
}

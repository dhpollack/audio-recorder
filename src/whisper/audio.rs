// Audio processing code, adapted from whisper.cpp
// https://github.com/ggerganov/whisper.cpp
use super::model;

// Constants
const ZERO: f32 = 0.0;
const HALF: f32 = 0.5;
const ONE: f32 = 1.0;
const FOUR: f32 = 4.0;
const EIGHT: f32 = 8.0;
const MIN_LOG_VALUE: f32 = 1e-10;
const TWO_PI: f32 = std::f32::consts::PI * 2.0;

// https://github.com/ggerganov/whisper.cpp/blob/4774d2feb01a772a15de81ffc34b34a1f294f020/whisper.cpp#L2357
fn fft(inp: &[f32]) -> Vec<f32> {
    let n = inp.len();
    if n == 1 {
        return vec![inp[0], ZERO];
    }
    if n % 2 == 1 {
        return dft(inp);
    }
    let mut out = vec![ZERO; n * 2];

    let mut even = Vec::with_capacity(n / 2);
    let mut odd = Vec::with_capacity(n / 2);

    for (i, &inp) in inp.iter().enumerate() {
        if i % 2 == 0 {
            even.push(inp)
        } else {
            odd.push(inp);
        }
    }

    let even_fft = fft(&even);
    let odd_fft = fft(&odd);

    let n_t = n as f32;
    for k in 0..n / 2 {
        let k_t = k as f32;
        let theta = TWO_PI * k_t / n_t;
        let re = theta.cos();
        let im = -theta.sin();

        let re_odd = odd_fft[2 * k];
        let im_odd = odd_fft[2 * k + 1];

        out[2 * k] = even_fft[2 * k] + re * re_odd - im * im_odd;
        out[2 * k + 1] = even_fft[2 * k + 1] + re * im_odd + im * re_odd;

        out[2 * (k + n / 2)] = even_fft[2 * k] - re * re_odd + im * im_odd;
        out[2 * (k + n / 2) + 1] = even_fft[2 * k + 1] - re * im_odd - im * re_odd;
    }
    out
}

// https://github.com/ggerganov/whisper.cpp/blob/4774d2feb01a772a15de81ffc34b34a1f294f020/whisper.cpp#L2337
fn dft(inp: &[f32]) -> Vec<f32> {
    let n = inp.len();
    let n_t = n as f32;

    (0..n)
        .flat_map(|k| {
            let k_t = k as f32;
            let (re, im) =
                inp.iter()
                    .enumerate()
                    .fold((ZERO, ZERO), |(re_acc, im_acc), (j, &val)| {
                        let j_t = j as f32;
                        let angle = TWO_PI * k_t * j_t / n_t;
                        (re_acc + val * angle.cos(), im_acc - val * angle.sin())
                    });
            [re, im]
        })
        .collect()
}

// https://github.com/ggerganov/whisper.cpp/blob/4774d2feb01a772a15de81ffc34b34a1f294f020/whisper.cpp#L2414
fn log_mel_spectrogram_w(
    hann: &[f32],
    samples: &[f32],
    filters: &[f32],
    fft_size: usize,
    fft_step: usize,
    n_len: usize,
    n_mel: usize,
) -> Vec<f32> {
    let n_fft = 1 + fft_size / 2;

    let mut fft_in = vec![ZERO; fft_size];
    let mut mel = vec![ZERO; n_len * n_mel];

    for i in 0..n_len {
        let offset = i * fft_step;

        // apply Hanning window
        for j in 0..fft_size {
            fft_in[j] = if offset + j < samples.len() {
                hann[j] * samples[offset + j]
            } else {
                ZERO
            }
        }

        // FFT -> mag^2
        let mut fft_out: Vec<f32> = fft(&fft_in);

        for j in 0..fft_size {
            fft_out[j] = fft_out[2 * j] * fft_out[2 * j] + fft_out[2 * j + 1] * fft_out[2 * j + 1];
        }
        for j in 1..fft_size / 2 {
            let v = fft_out[fft_size - j];
            fft_out[j] += v;
        }

        // mel spectrogram
        for j in 0..n_mel {
            let mut sum = ZERO;
            for k in 0..n_fft {
                sum += fft_out[k] * filters[j * n_fft + k];
            }
            mel[j * n_len + i] = sum.max(MIN_LOG_VALUE).log10();
        }
    }
    mel
}

fn log_mel_spectrogram_(
    samples: &[f32],
    filters: &[f32],
    fft_size: usize,
    fft_step: usize,
    n_mel: usize,
) -> Vec<f32> {
    let fft_size_t = fft_size as f32;

    let hann: Vec<f32> = (0..fft_size)
        .map(|i| HALF * (ONE - ((TWO_PI * i as f32) / fft_size_t).cos()))
        .collect();
    let n_len = samples.len() / fft_step;

    // pad audio with at least one extra chunk of zeros
    let pad = 100 * model::m::CHUNK_LENGTH / 2;
    let n_len = if !n_len.is_multiple_of(pad) {
        (n_len / pad + 1) * pad
    } else {
        n_len
    };
    let n_len = n_len + pad;
    let samples = {
        let mut samples_padded = samples.to_vec();
        let to_add = n_len * fft_step - samples.len();
        samples_padded.extend(std::iter::repeat_n(ZERO, to_add));
        samples_padded
    };

    let mut mel = log_mel_spectrogram_w(&hann, &samples, filters, fft_size, fft_step, n_len, n_mel);
    let mmax = mel
        .iter()
        .max_by(|&u, &v| u.partial_cmp(v).unwrap_or(std::cmp::Ordering::Greater))
        .copied()
        .unwrap_or(ZERO)
        - EIGHT;
    for m in mel.iter_mut() {
        let v = (*m).max(mmax);
        *m = v / FOUR + ONE
    }
    mel
}

pub fn pcm_to_mel(
    cfg: &model::m::Config,
    samples: &[f32],
    filters: &[f32],
) -> anyhow::Result<Vec<f32>> {
    let mel = log_mel_spectrogram_(
        samples,
        filters,
        model::m::N_FFT,
        model::m::HOP_LENGTH,
        cfg.num_mel_bins,
    );
    Ok(mel)
}

// Tests taken from candle-transformers
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fft() {
        let input: Vec<f32> = vec![0.0, 1.0, 0.0, 0.0];
        let output = fft(&input);
        assert_eq!(
            output,
            vec![1.0, 0.0, -4.371139e-8, -1.0, -1.0, 0.0, 4.371139e-8, 1.0]
        );
    }

    #[test]
    fn test_dft() {
        let input: Vec<f32> = vec![0.0, 1.0, 0.0, 0.0];
        let output = dft(&input);
        assert_eq!(
            output,
            vec![
                1.0,
                0.0,
                -4.371139e-8,
                -1.0,
                -1.0,
                8.742278e-8,
                1.1924881e-8,
                1.0
            ]
        );
    }

    #[test]
    fn test_log_mel_spectrogram() {
        let samples: Vec<f32> = vec![0.0; 1000];
        let filters = vec![0.0; 1000];
        let output = log_mel_spectrogram_(&samples, &filters, 100, 10, 10);
        assert_eq!(output.len(), 30_000);
        let sum = output.iter().sum::<f32>().abs() - 0.0;
        assert!((sum - 45000.0).abs() < 1e-5, "{sum:.8}");
    }

    #[test]
    fn test_tiny_log_mel_spectrogram() {
        let samples: Vec<f32> = vec![0.0; 100];
        let filters = vec![0.0; 100];
        let output = log_mel_spectrogram_(&samples, &filters, 20, 2, 2);
        assert_eq!(output.len(), 6_000);
        let sum = output.iter().sum::<f32>().abs() - 0.0;
        assert!((sum - 9000.0).abs() < 1e-5, "{sum:.8}");
    }
}

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use makosh_desktop_call_recording_api::MAX_AUDIO_BYTES_V1;

const OUTPUT_SAMPLE_RATE: u32 = 16_000;
const WAV_HEADER_BYTES: usize = 44;

pub(super) struct SelectedInputV1 {
    pub(super) label: String,
    device: cpal::Device,
    config: cpal::SupportedStreamConfig,
}

pub(super) struct NativeCaptureV1 {
    _stream: Stream,
    samples: Arc<Mutex<DownsampledBufferV1>>,
    failed: Arc<AtomicBool>,
}

impl SelectedInputV1 {
    pub(super) fn system_default() -> Result<Self, &'static str> {
        let device = cpal::default_host()
            .default_input_device()
            .ok_or("audio_input_unavailable")?;
        let label = device
            .description()
            .map_err(|_| "audio_input_unavailable")?
            .name()
            .to_owned();
        let config = device
            .default_input_config()
            .map_err(|_| "audio_input_unavailable")?;
        if config.sample_rate() < OUTPUT_SAMPLE_RATE || config.channels() == 0 {
            return Err("audio_format_unsupported");
        }
        Ok(Self {
            label,
            device,
            config,
        })
    }

    pub(super) fn start(
        self,
        maximum_duration_millis: u64,
    ) -> Result<NativeCaptureV1, &'static str> {
        let max_by_duration = maximum_duration_millis
            .checked_mul(u64::from(OUTPUT_SAMPLE_RATE))
            .and_then(|value| value.checked_div(1_000))
            .and_then(|value| usize::try_from(value).ok())
            .ok_or("audio_bounds_invalid")?;
        let max_by_bytes = (MAX_AUDIO_BYTES_V1 - WAV_HEADER_BYTES) / 2;
        let max_samples = max_by_duration.min(max_by_bytes);
        if max_samples == 0 {
            return Err("audio_bounds_invalid");
        }
        let channels = usize::from(self.config.channels());
        let input_rate = self.config.sample_rate();
        let samples = Arc::new(Mutex::new(DownsampledBufferV1::new(
            input_rate,
            channels,
            max_samples,
        )));
        let failed = Arc::new(AtomicBool::new(false));
        let config: StreamConfig = self.config.into();
        let stream = match self.config.sample_format() {
            SampleFormat::F32 => build_stream(
                &self.device,
                config,
                samples.clone(),
                failed.clone(),
                |sample: f32| sample,
            ),
            SampleFormat::I16 => build_stream(
                &self.device,
                config,
                samples.clone(),
                failed.clone(),
                |sample: i16| f32::from(sample) / 32_768.0,
            ),
            SampleFormat::U16 => build_stream(
                &self.device,
                config,
                samples.clone(),
                failed.clone(),
                |sample: u16| (f32::from(sample) - 32_768.0) / 32_768.0,
            ),
            _ => return Err("audio_format_unsupported"),
        }?;
        stream.play().map_err(|error| match error.kind() {
            cpal::ErrorKind::PermissionDenied => "os_permission_denied",
            _ => "audio_capture_unavailable",
        })?;
        Ok(NativeCaptureV1 {
            _stream: stream,
            samples,
            failed,
        })
    }
}

impl NativeCaptureV1 {
    pub(super) fn reached_limit(&self) -> bool {
        self.samples
            .lock()
            .map(|samples| samples.reached_limit)
            .unwrap_or(true)
    }

    pub(super) fn failed(&self) -> bool {
        self.failed.load(Ordering::Acquire)
    }

    pub(super) fn finish(self) -> Result<Vec<u8>, &'static str> {
        let Self {
            _stream,
            samples,
            failed,
        } = self;
        drop(_stream);
        if failed.load(Ordering::Acquire) {
            return Err("audio_capture_failed");
        }
        let samples = Arc::try_unwrap(samples)
            .map_err(|_| "audio_capture_failed")?
            .into_inner()
            .map_err(|_| "audio_capture_failed")?
            .samples;
        if samples.is_empty() {
            return Err("audio_capture_empty");
        }
        encode_wav(&samples).ok_or("audio_capture_failed")
    }
}

fn build_stream<T, F>(
    device: &cpal::Device,
    config: StreamConfig,
    samples: Arc<Mutex<DownsampledBufferV1>>,
    failed: Arc<AtomicBool>,
    convert: F,
) -> Result<Stream, &'static str>
where
    T: cpal::SizedSample + Copy,
    F: Fn(T) -> f32 + Send + Copy + 'static,
{
    let failed_for_data = failed.clone();
    device
        .build_input_stream(
            config,
            move |data: &[T], _| {
                let Ok(mut buffer) = samples.lock() else {
                    failed_for_data.store(true, Ordering::Release);
                    return;
                };
                buffer.push(data, convert);
            },
            move |_| failed.store(true, Ordering::Release),
            Some(Duration::from_secs(5)),
        )
        .map_err(|error| match error.kind() {
            cpal::ErrorKind::PermissionDenied => "os_permission_denied",
            _ => "audio_capture_unavailable",
        })
}

struct DownsampledBufferV1 {
    input_rate: u32,
    channels: usize,
    phase: u32,
    max_samples: usize,
    samples: Vec<i16>,
    reached_limit: bool,
}

impl DownsampledBufferV1 {
    fn new(input_rate: u32, channels: usize, max_samples: usize) -> Self {
        Self {
            input_rate,
            channels,
            phase: 0,
            max_samples,
            samples: Vec::with_capacity(max_samples.min(OUTPUT_SAMPLE_RATE as usize * 60)),
            reached_limit: false,
        }
    }

    fn push<T, F>(&mut self, data: &[T], convert: F)
    where
        T: Copy,
        F: Fn(T) -> f32 + Copy,
    {
        if self.reached_limit {
            return;
        }
        for frame in data.chunks_exact(self.channels) {
            let mono = frame.iter().copied().map(convert).sum::<f32>() / self.channels as f32;
            self.phase = self.phase.saturating_add(OUTPUT_SAMPLE_RATE);
            if self.phase >= self.input_rate {
                self.phase -= self.input_rate;
                let scaled = (mono.clamp(-1.0, 1.0) * f32::from(i16::MAX)).round();
                self.samples.push(scaled as i16);
                if self.samples.len() >= self.max_samples {
                    self.reached_limit = true;
                    return;
                }
            }
        }
    }
}

fn encode_wav(samples: &[i16]) -> Option<Vec<u8>> {
    let data_bytes = samples.len().checked_mul(2)?;
    let data_bytes_u32 = u32::try_from(data_bytes).ok()?;
    let riff_bytes = data_bytes_u32.checked_add(36)?;
    let mut wav = Vec::with_capacity(WAV_HEADER_BYTES + data_bytes);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&riff_bytes.to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16_u32.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&1_u16.to_le_bytes());
    wav.extend_from_slice(&OUTPUT_SAMPLE_RATE.to_le_bytes());
    wav.extend_from_slice(&(OUTPUT_SAMPLE_RATE * 2).to_le_bytes());
    wav.extend_from_slice(&2_u16.to_le_bytes());
    wav.extend_from_slice(&16_u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_bytes_u32.to_le_bytes());
    for sample in samples {
        wav.extend_from_slice(&sample.to_le_bytes());
    }
    Some(wav)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downsampling_is_mono_bounded_and_wav_is_canonical() {
        let mut buffer = DownsampledBufferV1::new(48_000, 2, 16_000);
        let stereo = vec![0.5_f32; 48_000 * 2];
        buffer.push(&stereo, |sample| sample);
        assert_eq!(buffer.samples.len(), 16_000);
        assert!(buffer.reached_limit);
        let wav = encode_wav(&buffer.samples).expect("wav");
        assert_eq!(&wav[..4], b"RIFF");
        assert_eq!(&wav[8..16], b"WAVEfmt ");
        assert_eq!(u32::from_le_bytes(wav[24..28].try_into().unwrap()), 16_000);
        assert_eq!(u16::from_le_bytes(wav[22..24].try_into().unwrap()), 1);
        assert_eq!(wav.len(), 32_044);
    }
}

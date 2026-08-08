#![forbid(unsafe_code)]

use makosh_desktop_call_recording_api::{MAX_AUDIO_BYTES_V1, MAX_DURATION_MILLIS_V1};
use sha2::{Digest, Sha256};

pub const PACKAGE: &str = "makosh-desktop-call-recording-core";
pub const WAV_HEADER_BYTES_V1: usize = 44;
pub const WAV_BYTES_PER_SECOND_V1: u64 = 32_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordingStateV1 {
    AwaitingConsent,
    Capturing,
    Materializing,
    Ready,
    Rejected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartRecordingV1 {
    pub operation_id: [u8; 16],
    pub call_evidence_id: [u8; 16],
    pub expected_call_revision: u64,
    pub maximum_duration_millis: u64,
    pub consent_policy_revision: u32,
    pub logical_owner_id: String,
    pub authenticated_device_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalWavReceiptV1 {
    pub declared_bytes: u64,
    pub duration_millis: u64,
    pub sha256: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordingCoreErrorV1 {
    InvalidIdentity,
    InvalidRevision,
    InvalidDuration,
    InvalidWav,
    AudioTooLarge,
    DurationExceeded,
    InvalidTransition,
}

pub fn validate_start_v1(value: &StartRecordingV1) -> Result<(), RecordingCoreErrorV1> {
    if value.operation_id.iter().all(|byte| *byte == 0)
        || value.call_evidence_id.iter().all(|byte| *byte == 0)
        || !valid_identity(&value.logical_owner_id)
        || !valid_identity(&value.authenticated_device_id)
    {
        return Err(RecordingCoreErrorV1::InvalidIdentity);
    }
    if value.expected_call_revision == 0 || value.consent_policy_revision == 0 {
        return Err(RecordingCoreErrorV1::InvalidRevision);
    }
    if !(1_000..=MAX_DURATION_MILLIS_V1).contains(&value.maximum_duration_millis) {
        return Err(RecordingCoreErrorV1::InvalidDuration);
    }
    Ok(())
}

pub fn device_actor_sha256_v1(owner: &str, device: &str) -> Result<[u8; 32], RecordingCoreErrorV1> {
    if !valid_identity(owner) || !valid_identity(device) {
        return Err(RecordingCoreErrorV1::InvalidIdentity);
    }
    let mut digest = Sha256::new();
    digest.update(b"makosh.desktop-call-recording.device-actor.v1\0");
    digest.update(owner.as_bytes());
    digest.update([0]);
    digest.update(device.as_bytes());
    Ok(digest.finalize().into())
}

pub fn validate_canonical_wav_v1(
    bytes: &[u8],
    maximum_duration_millis: u64,
) -> Result<CanonicalWavReceiptV1, RecordingCoreErrorV1> {
    if bytes.len() > MAX_AUDIO_BYTES_V1 {
        return Err(RecordingCoreErrorV1::AudioTooLarge);
    }
    if bytes.len() < WAV_HEADER_BYTES_V1
        || &bytes[0..4] != b"RIFF"
        || &bytes[8..12] != b"WAVE"
        || &bytes[12..16] != b"fmt "
        || u32::from_le_bytes(
            bytes[16..20]
                .try_into()
                .map_err(|_| RecordingCoreErrorV1::InvalidWav)?,
        ) != 16
        || u16::from_le_bytes(
            bytes[20..22]
                .try_into()
                .map_err(|_| RecordingCoreErrorV1::InvalidWav)?,
        ) != 1
        || u16::from_le_bytes(
            bytes[22..24]
                .try_into()
                .map_err(|_| RecordingCoreErrorV1::InvalidWav)?,
        ) != 1
        || u32::from_le_bytes(
            bytes[24..28]
                .try_into()
                .map_err(|_| RecordingCoreErrorV1::InvalidWav)?,
        ) != 16_000
        || u32::from_le_bytes(
            bytes[28..32]
                .try_into()
                .map_err(|_| RecordingCoreErrorV1::InvalidWav)?,
        ) != 32_000
        || u16::from_le_bytes(
            bytes[32..34]
                .try_into()
                .map_err(|_| RecordingCoreErrorV1::InvalidWav)?,
        ) != 2
        || u16::from_le_bytes(
            bytes[34..36]
                .try_into()
                .map_err(|_| RecordingCoreErrorV1::InvalidWav)?,
        ) != 16
        || &bytes[36..40] != b"data"
    {
        return Err(RecordingCoreErrorV1::InvalidWav);
    }
    let declared_data = u32::from_le_bytes(
        bytes[40..44]
            .try_into()
            .map_err(|_| RecordingCoreErrorV1::InvalidWav)?,
    ) as usize;
    if declared_data == 0
        || !declared_data.is_multiple_of(2)
        || declared_data + WAV_HEADER_BYTES_V1 != bytes.len()
    {
        return Err(RecordingCoreErrorV1::InvalidWav);
    }
    let riff_size = u32::from_le_bytes(
        bytes[4..8]
            .try_into()
            .map_err(|_| RecordingCoreErrorV1::InvalidWav)?,
    ) as usize;
    if riff_size + 8 != bytes.len() {
        return Err(RecordingCoreErrorV1::InvalidWav);
    }
    let duration_millis =
        u64::try_from(declared_data).map_err(|_| RecordingCoreErrorV1::InvalidWav)? * 1_000
            / WAV_BYTES_PER_SECOND_V1;
    if duration_millis == 0
        || duration_millis > maximum_duration_millis
        || duration_millis > MAX_DURATION_MILLIS_V1
    {
        return Err(RecordingCoreErrorV1::DurationExceeded);
    }
    Ok(CanonicalWavReceiptV1 {
        declared_bytes: bytes.len() as u64,
        duration_millis,
        sha256: Sha256::digest(bytes).into(),
    })
}

pub fn transition_v1(
    current: RecordingStateV1,
    next: RecordingStateV1,
) -> Result<(), RecordingCoreErrorV1> {
    let valid = matches!(
        (current, next),
        (
            RecordingStateV1::AwaitingConsent,
            RecordingStateV1::Capturing | RecordingStateV1::Rejected
        ) | (
            RecordingStateV1::Capturing,
            RecordingStateV1::Materializing | RecordingStateV1::Rejected
        ) | (
            RecordingStateV1::Materializing,
            RecordingStateV1::Ready | RecordingStateV1::Rejected
        )
    );
    valid
        .then_some(())
        .ok_or(RecordingCoreErrorV1::InvalidTransition)
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wav(samples: usize) -> Vec<u8> {
        let data_size = samples * 2;
        let mut out = Vec::with_capacity(44 + data_size);
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&(36_u32 + data_size as u32).to_le_bytes());
        out.extend_from_slice(b"WAVEfmt ");
        out.extend_from_slice(&16_u32.to_le_bytes());
        out.extend_from_slice(&1_u16.to_le_bytes());
        out.extend_from_slice(&1_u16.to_le_bytes());
        out.extend_from_slice(&16_000_u32.to_le_bytes());
        out.extend_from_slice(&32_000_u32.to_le_bytes());
        out.extend_from_slice(&2_u16.to_le_bytes());
        out.extend_from_slice(&16_u16.to_le_bytes());
        out.extend_from_slice(b"data");
        out.extend_from_slice(&(data_size as u32).to_le_bytes());
        out.resize(44 + data_size, 0);
        out
    }

    #[test]
    fn accepts_only_exact_canonical_wav() {
        let bytes = wav(16_000);
        let receipt = validate_canonical_wav_v1(&bytes, 2_000).expect("canonical wav");
        assert_eq!(receipt.duration_millis, 1_000);
        let mut stereo = bytes;
        stereo[22] = 2;
        assert_eq!(
            validate_canonical_wav_v1(&stereo, 2_000),
            Err(RecordingCoreErrorV1::InvalidWav)
        );
    }

    #[test]
    fn terminal_states_are_immutable() {
        assert_eq!(
            transition_v1(RecordingStateV1::Ready, RecordingStateV1::Rejected),
            Err(RecordingCoreErrorV1::InvalidTransition)
        );
        assert_eq!(
            transition_v1(
                RecordingStateV1::AwaitingConsent,
                RecordingStateV1::Capturing
            ),
            Ok(())
        );
    }
}

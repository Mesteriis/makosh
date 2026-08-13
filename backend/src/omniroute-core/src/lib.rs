#![forbid(unsafe_code)]
use sha2::{Digest, Sha256};
pub const PACKAGE: &str = "makosh-omniroute-core";
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OmniRouteRequestReceiptV1 {
    pub request_id: [u8; 16],
    pub logical_owner_id: String,
    pub contract_name: String,
    pub request_sha256: [u8; 32],
    pub model: String,
    pub settings_revision: u64,
    pub accepted_at_unix_millis: i64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OmniRouteCoreErrorV1 {
    Invalid,
}
pub fn validate_request_receipt_v1(
    v: &OmniRouteRequestReceiptV1,
) -> Result<(), OmniRouteCoreErrorV1> {
    if v.request_id.iter().all(|b| *b == 0)
        || v.request_sha256.iter().all(|b| *b == 0)
        || !atom(&v.logical_owner_id, 128)
        || !matches!(
            v.contract_name.as_str(),
            "ai_provider_reply_generation"
                | "ai_provider_summary_generation"
                | "ai_provider_translation"
                | "ai_provider_explanation"
        )
        || !atom(&v.model, 128)
        || v.settings_revision == 0
        || v.accepted_at_unix_millis <= 0
    {
        Err(OmniRouteCoreErrorV1::Invalid)
    } else {
        Ok(())
    }
}
#[must_use]
pub fn provider_request_fingerprint_v1(v: &OmniRouteRequestReceiptV1) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"makosh.omniroute.request.v1\0");
    h.update(v.request_id);
    h.update(v.logical_owner_id.as_bytes());
    h.update(b"\0");
    h.update(v.contract_name.as_bytes());
    h.update(v.request_sha256);
    h.update(v.model.as_bytes());
    h.update(v.settings_revision.to_be_bytes());
    h.finalize().into()
}
fn atom(v: &str, max: usize) -> bool {
    !v.is_empty()
        && v.len() <= max
        && v.bytes().all(|b| {
            b.is_ascii_lowercase()
                || b.is_ascii_digit()
                || matches!(b, b'.' | b'_' | b'-' | b':' | b'/')
        })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn receipt_is_typed_and_digest_bound() {
        let v = OmniRouteRequestReceiptV1 {
            request_id: [1; 16],
            logical_owner_id: "owner-1".into(),
            contract_name: "ai_provider_reply_generation".into(),
            request_sha256: [2; 32],
            model: "route/model".into(),
            settings_revision: 1,
            accepted_at_unix_millis: 1,
        };
        assert_eq!(validate_request_receipt_v1(&v), Ok(()));
        let mut changed = v.clone();
        changed.request_sha256 = [3; 32];
        assert_ne!(
            provider_request_fingerprint_v1(&v),
            provider_request_fingerprint_v1(&changed)
        );
    }
}

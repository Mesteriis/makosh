//! Keyed digest primitive for the owner-local Communications search projection.

use sha2::{Digest, Sha256};

const HMAC_BLOCK_BYTES: usize = 64;
const SEARCH_TOKEN_DOMAIN_V1: &[u8] = b"makosh.communications.search.token.v1\0";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsSearchDigestErrorV1 {
    InvalidKey,
    InvalidToken,
}

pub fn keyed_search_token_digest_v1(
    key: &[u8],
    normalized_token: &str,
) -> Result<[u8; 32], CommunicationsSearchDigestErrorV1> {
    if key.len() != 32 {
        return Err(CommunicationsSearchDigestErrorV1::InvalidKey);
    }
    if normalized_token.is_empty() || normalized_token.len() > 512 {
        return Err(CommunicationsSearchDigestErrorV1::InvalidToken);
    }
    let mut inner_pad = [0x36_u8; HMAC_BLOCK_BYTES];
    let mut outer_pad = [0x5c_u8; HMAC_BLOCK_BYTES];
    for (index, byte) in key.iter().enumerate() {
        inner_pad[index] ^= byte;
        outer_pad[index] ^= byte;
    }
    let mut inner = Sha256::new();
    inner.update(inner_pad);
    inner.update(SEARCH_TOKEN_DOMAIN_V1);
    inner.update(normalized_token.as_bytes());
    let inner_digest = inner.finalize();
    let mut outer = Sha256::new();
    outer.update(outer_pad);
    outer.update(inner_digest);
    Ok(outer.finalize().into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_is_stable_keyed_and_token_specific() {
        let key = [7; 32];
        assert_eq!(
            keyed_search_token_digest_v1(&key, "hello"),
            keyed_search_token_digest_v1(&key, "hello"),
        );
        assert_ne!(
            keyed_search_token_digest_v1(&key, "hello"),
            keyed_search_token_digest_v1(&key, "world"),
        );
        assert_ne!(
            keyed_search_token_digest_v1(&key, "hello"),
            keyed_search_token_digest_v1(&[8; 32], "hello"),
        );
    }

    #[test]
    fn digest_rejects_unusable_inputs() {
        assert_eq!(
            keyed_search_token_digest_v1(&[1; 31], "hello"),
            Err(CommunicationsSearchDigestErrorV1::InvalidKey)
        );
        assert_eq!(
            keyed_search_token_digest_v1(&[1; 32], ""),
            Err(CommunicationsSearchDigestErrorV1::InvalidToken)
        );
    }
}

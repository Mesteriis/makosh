//! Canonical owner-control challenge proof encoding.

const PROOF_DOMAIN: &[u8] = b"makosh.owner-control-session.v1\0";

pub fn owner_control_proof_message_v1(
    kernel_instance_id: &str,
    owner_id: &str,
    device_id: &str,
    control_store_generation: u64,
    challenge_bytes: &[u8; 32],
) -> Result<Vec<u8>, String> {
    let mut message = Vec::with_capacity(PROOF_DOMAIN.len() + 160);
    message.extend_from_slice(PROOF_DOMAIN);
    for value in [kernel_instance_id, owner_id, device_id] {
        let length = u16::try_from(value.len())
            .map_err(|_| "owner control proof field is too large".to_owned())?;
        message.extend_from_slice(&length.to_be_bytes());
        message.extend_from_slice(value.as_bytes());
    }
    message.extend_from_slice(&control_store_generation.to_be_bytes());
    message.extend_from_slice(challenge_bytes);
    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::owner_control_proof_message_v1;

    #[test]
    fn proof_message_is_domain_separated_and_length_delimited() {
        let message = owner_control_proof_message_v1("kernel", "owner", "device", 7, &[3; 32])
            .expect("proof message");

        assert!(message.starts_with(b"makosh.owner-control-session.v1\0\0\x06kernel"));
        assert!(message.ends_with(&[3; 32]));
    }
}

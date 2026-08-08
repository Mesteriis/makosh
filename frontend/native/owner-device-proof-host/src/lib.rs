use std::fs::File;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use p256::ecdsa::{Signature, SigningKey, signature::Signer};
use zeroize::Zeroizing;

pub struct OwnerDeviceProofHostV1 {
    signer: SigningKey,
}

impl OwnerDeviceProofHostV1 {
    pub fn open(key_file: &Path) -> Result<Self, OwnerDeviceProofHostErrorV1> {
        if !key_file.is_absolute() {
            return Err(OwnerDeviceProofHostErrorV1::Unavailable);
        }
        let metadata = std::fs::symlink_metadata(key_file)
            .map_err(|_| OwnerDeviceProofHostErrorV1::Unavailable)?;
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.permissions().mode() & 0o077 != 0
            || metadata.len() != 32
        {
            return Err(OwnerDeviceProofHostErrorV1::Unavailable);
        }
        let mut bytes = Zeroizing::new([0_u8; 32]);
        File::open(key_file)
            .and_then(|mut file| file.read_exact(bytes.as_mut()))
            .map_err(|_| OwnerDeviceProofHostErrorV1::Unavailable)?;
        let signer = SigningKey::from_bytes((&*bytes).into())
            .map_err(|_| OwnerDeviceProofHostErrorV1::Unavailable)?;
        Ok(Self { signer })
    }

    #[must_use]
    pub fn sign_challenge(&self, challenge: &[u8; 32]) -> [u8; 64] {
        let signature: Signature = self.signer.sign(challenge);
        signature.to_bytes().into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnerDeviceProofHostErrorV1 {
    Unavailable,
}

impl std::fmt::Display for OwnerDeviceProofHostErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("owner device proof host is unavailable")
    }
}

impl std::error::Error for OwnerDeviceProofHostErrorV1 {}

#[cfg(test)]
mod tests {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    use std::sync::atomic::{AtomicU64, Ordering};

    use p256::ecdsa::{VerifyingKey, signature::Verifier};

    use super::*;

    static FIXTURE_ID: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn signs_only_fixed_owner_challenges_with_private_file_key() {
        let path = fixture_path("sign");
        let key = [7_u8; 32];
        write_private(&path, &key);
        let host = OwnerDeviceProofHostV1::open(&path).expect("open signer");
        let challenge = [9_u8; 32];
        let signature = Signature::from_slice(&host.sign_challenge(&challenge)).expect("signature");
        let signing_key = SigningKey::from_bytes((&key).into()).expect("test key");
        VerifyingKey::from(&signing_key)
            .verify(&challenge, &signature)
            .expect("verify signature");
        std::fs::remove_file(path).expect("remove signer");
    }

    #[test]
    fn rejects_relative_and_broadly_readable_key_files() {
        assert_eq!(
            OwnerDeviceProofHostV1::open(Path::new("device-es256.key"))
                .err()
                .expect("relative path rejected"),
            OwnerDeviceProofHostErrorV1::Unavailable,
        );

        let path = fixture_path("permissions");
        write_private(&path, &[8_u8; 32]);
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644))
            .expect("broaden permissions");
        assert_eq!(
            OwnerDeviceProofHostV1::open(&path)
                .err()
                .expect("broad permissions rejected"),
            OwnerDeviceProofHostErrorV1::Unavailable,
        );
        std::fs::remove_file(path).expect("remove signer");
    }

    fn fixture_path(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "makosh-owner-device-proof-{label}-{}-{}",
            std::process::id(),
            FIXTURE_ID.fetch_add(1, Ordering::Relaxed),
        ))
    }

    fn write_private(path: &Path, bytes: &[u8]) {
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .mode(0o600)
            .open(path)
            .expect("create signer");
        file.write_all(bytes).expect("write signer");
        file.sync_all().expect("sync signer");
    }
}

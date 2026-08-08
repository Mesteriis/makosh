//! Release-bound public verification keys for managed distribution manifests.

use std::{fs::File, io::Read, os::unix::fs::MetadataExt, path::Path};

use makosh_runtime_protocol::validation::distribution::{
    MAX_RELEASE_TRUST_ROOT_BYTES, decode_release_trust_root_v1,
};
use p256::ecdsa::VerifyingKey;

pub struct PinnedDistributionVerificationKey {
    key_id: String,
    public_key_sec1: [u8; 65],
}

impl PinnedDistributionVerificationKey {
    #[must_use]
    pub fn public_key_sec1(&self) -> &[u8; 65] {
        &self.public_key_sec1
    }
}

pub struct ReleaseTrustRoot {
    verification_keys: Vec<PinnedDistributionVerificationKey>,
}

impl ReleaseTrustRoot {
    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.is_absolute() {
            return Err("release trust root path must be absolute".to_owned());
        }
        let before = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
        if before.file_type().is_symlink()
            || !before.is_file()
            || before.len() > MAX_RELEASE_TRUST_ROOT_BYTES as u64
        {
            return Err("release trust root is not a bounded regular file".to_owned());
        }
        let mut file = File::open(path).map_err(|error| error.to_string())?;
        let opened = file.metadata().map_err(|error| error.to_string())?;
        if !same_file(&before, &opened) {
            return Err("release trust root changed while it was opened".to_owned());
        }
        let mut bytes = Vec::with_capacity(
            usize::try_from(opened.len())
                .map_err(|_| "release trust root is too large".to_owned())?,
        );
        file.read_to_end(&mut bytes)
            .map_err(|error| error.to_string())?;
        let after = file.metadata().map_err(|error| error.to_string())?;
        let path_after = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
        if !same_file(&opened, &after) || !same_file(&opened, &path_after) {
            return Err("release trust root changed while it was read".to_owned());
        }
        Self::decode(&bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        let root = decode_release_trust_root_v1(bytes)
            .map_err(|_| "release trust root is invalid".to_owned())?;
        let verification_keys = root
            .verification_keys
            .into_iter()
            .map(|key| {
                let public_key_sec1: [u8; 65] = key
                    .public_key_sec1
                    .try_into()
                    .map_err(|_| "release trust root is invalid".to_owned())?;
                VerifyingKey::from_sec1_bytes(&public_key_sec1)
                    .map_err(|_| "release trust root is invalid".to_owned())?;
                Ok(PinnedDistributionVerificationKey {
                    key_id: key.key_id,
                    public_key_sec1,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Self { verification_keys })
    }

    #[must_use]
    pub fn verification_key(&self, key_id: &str) -> Option<&PinnedDistributionVerificationKey> {
        self.verification_keys
            .iter()
            .find(|key| key.key_id == key_id)
    }
}

fn same_file(left: &std::fs::Metadata, right: &std::fs::Metadata) -> bool {
    left.dev() == right.dev()
        && left.ino() == right.ino()
        && left.len() == right.len()
        && left.mtime() == right.mtime()
        && left.mtime_nsec() == right.mtime_nsec()
        && left.ctime() == right.ctime()
        && left.ctime_nsec() == right.ctime_nsec()
}

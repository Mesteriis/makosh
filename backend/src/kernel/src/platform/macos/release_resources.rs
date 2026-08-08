//! Locates immutable managed-launch resources inside a signed macOS app bundle.

use std::path::{Path, PathBuf};

const RESOURCE_DIRECTORY: &str = "makosh-kernel-release";
const TRUST_ROOT_FILE: &str = "makosh-release-trust-root.pb";
const SIGNED_MANIFEST_FILE: &str = "makosh-signed-distribution-manifest.pb";
const DISTRIBUTION_DIRECTORY: &str = "distribution";

pub struct MacosReleaseResources {
    distribution_root: PathBuf,
    signed_manifest_path: PathBuf,
    trust_root_path: PathBuf,
}

impl MacosReleaseResources {
    #[must_use]
    pub fn distribution_root(&self) -> &Path {
        &self.distribution_root
    }

    #[must_use]
    pub fn signed_manifest_path(&self) -> &Path {
        &self.signed_manifest_path
    }

    #[must_use]
    pub fn trust_root_path(&self) -> &Path {
        &self.trust_root_path
    }
}

pub fn discover_from_executable(executable: &Path) -> Result<MacosReleaseResources, String> {
    ensure_regular_non_symlink(executable, "Kernel executable")?;
    let macos_directory = executable
        .parent()
        .filter(|path| path.file_name().is_some_and(|name| name == "MacOS"))
        .ok_or_else(|| "Kernel executable is not inside a macOS app bundle".to_owned())?;
    let contents_directory = macos_directory
        .parent()
        .filter(|path| path.file_name().is_some_and(|name| name == "Contents"))
        .ok_or_else(|| "Kernel executable is not inside a macOS app bundle".to_owned())?;
    let app_bundle_directory = contents_directory
        .parent()
        .filter(|path| path.extension().is_some_and(|extension| extension == "app"))
        .ok_or_else(|| "Kernel executable is not inside a macOS app bundle".to_owned())?;
    ensure_directory_non_symlink(macos_directory, "macOS executable directory")?;
    ensure_directory_non_symlink(contents_directory, "macOS Contents directory")?;
    ensure_directory_non_symlink(app_bundle_directory, "macOS app bundle")?;
    let resources_directory = contents_directory.join("Resources");
    ensure_directory_non_symlink(&resources_directory, "macOS Resources directory")?;
    let resource_root = resources_directory.join(RESOURCE_DIRECTORY);
    ensure_directory_non_symlink(&resource_root, "managed launch resource directory")?;
    let distribution_root = resource_root.join(DISTRIBUTION_DIRECTORY);
    let signed_manifest_path = resource_root.join(SIGNED_MANIFEST_FILE);
    let trust_root_path = resource_root.join(TRUST_ROOT_FILE);
    ensure_directory_non_symlink(&distribution_root, "managed distribution bundle")?;
    ensure_regular_non_symlink(&signed_manifest_path, "signed distribution manifest")?;
    ensure_regular_non_symlink(&trust_root_path, "release trust root")?;
    Ok(MacosReleaseResources {
        distribution_root,
        signed_manifest_path,
        trust_root_path,
    })
}

fn ensure_regular_non_symlink(path: &Path, label: &str) -> Result<(), String> {
    if !path.is_absolute() {
        return Err(format!("{label} path must be absolute"));
    }
    let metadata = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!("{label} must be a regular non-symlink file"));
    }
    Ok(())
}

fn ensure_directory_non_symlink(path: &Path, label: &str) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!("{label} must be a non-symlink directory"));
    }
    Ok(())
}

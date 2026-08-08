//! Explicit local operator ceremony for admitting one browser device.

use std::io::{self, Write};
use std::path::PathBuf;

use makosh_gateway_protocol::owner_control_client::{
    OwnerControlClientV1, OwnerControlProofSignerV1,
};

use crate::identity::device::signer::{DeviceSigner, FileDeviceSigner};
use crate::infrastructure::filesystem::{resolve_data_directory, resolve_runtime_directory};

pub(crate) fn create_browser_pairing(data_dir_override: Option<PathBuf>) -> Result<(), String> {
    confirm_creation()?;
    let data_dir = resolve_data_directory(data_dir_override)?;
    let runtime_dir = resolve_runtime_directory(&data_dir)?;
    let client = OwnerControlClientV1::new(&runtime_dir);
    let signer = LocalDeviceSigner(FileDeviceSigner::open_for_instance(&data_dir)?);
    let session_id = client.open_owner_session(&signer)?;
    let pairing_id = client.begin_browser_pairing(&session_id)?;
    println!("browser_pairing_id={pairing_id}");
    println!("browser_pairing_state=approved_pending_registration");
    Ok(())
}

fn confirm_creation() -> Result<(), String> {
    eprint!("Create one-time browser pairing for a new device? [y/N] ");
    io::stderr().flush().map_err(|error| error.to_string())?;
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .map_err(|error| error.to_string())?;
    matches!(answer.trim(), "y" | "Y" | "yes" | "YES")
        .then_some(())
        .ok_or_else(|| "browser pairing was not confirmed".to_owned())
}

struct LocalDeviceSigner(FileDeviceSigner);

impl OwnerControlProofSignerV1 for LocalDeviceSigner {
    fn sign_owner_control_proof(&self, message: &[u8]) -> Result<[u8; 64], String> {
        Ok(self.0.sign(message))
    }
}

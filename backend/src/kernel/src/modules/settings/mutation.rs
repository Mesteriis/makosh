//! Validates and commits one operator-managed Settings snapshot revision.

use makosh_kernel_control_store::{
    HealthRecoveryStore, OwnerIdentityStore, SettingsDesiredSnapshot, SettingsRegistryStore,
};
use makosh_kernel_control_store_sqlite::StoreError;
use makosh_runtime_protocol::v1::{
    SettingClientVisibilityV1, SettingMutationAuthorityV1, SettingsSnapshotV1,
};
use makosh_runtime_protocol::validation::descriptor::{
    decode_settings_schema_v1, decode_settings_snapshot_v1,
    validate_settings_snapshot_against_schema_v1,
};
use prost::Message;
use sha2::{Digest, Sha256};

use crate::identity::owner::authorization::{authorize as authorize_file_owner, operation_digest};

pub fn commit<S>(
    data_dir: &std::path::Path,
    store: &S,
    registration_id: &str,
    expected_revision: u64,
    snapshot_bytes: &[u8],
) -> Result<u64, String>
where
    S: HealthRecoveryStore
        + OwnerIdentityStore<Error = StoreError>
        + SettingsRegistryStore<Error = StoreError>,
{
    commit_target(
        data_dir,
        store,
        registration_id,
        registration_id,
        expected_revision,
        snapshot_bytes,
    )
}

pub fn commit_target<S>(
    data_dir: &std::path::Path,
    store: &S,
    registration_id: &str,
    configuration_instance_id: &str,
    expected_revision: u64,
    snapshot_bytes: &[u8],
) -> Result<u64, String>
where
    S: HealthRecoveryStore
        + OwnerIdentityStore<Error = StoreError>
        + SettingsRegistryStore<Error = StoreError>,
{
    let (_, schema_sha256) = validate(
        store,
        registration_id,
        configuration_instance_id,
        expected_revision,
        snapshot_bytes,
    )?;
    let expected = expected_revision.to_string();
    let schema_hex = hex_digest(&schema_sha256);
    let snapshot_hex = hex_digest(&Sha256::digest(snapshot_bytes).into());
    authorize_file_owner(
        data_dir,
        store,
        "settings.operator_update.v1",
        operation_digest(&[
            registration_id,
            configuration_instance_id,
            &expected,
            &schema_hex,
            &snapshot_hex,
        ])?,
    )?;
    commit_after_owner_authorization_for_target(
        store,
        registration_id,
        configuration_instance_id,
        expected_revision,
        snapshot_bytes,
    )
}

pub fn commit_after_owner_authorization<S>(
    store: &S,
    registration_id: &str,
    expected_revision: u64,
    snapshot_bytes: &[u8],
) -> Result<u64, String>
where
    S: SettingsRegistryStore<Error = StoreError>,
{
    commit_after_owner_authorization_for_target(
        store,
        registration_id,
        registration_id,
        expected_revision,
        snapshot_bytes,
    )
}

pub fn commit_after_owner_authorization_for_target<S>(
    store: &S,
    registration_id: &str,
    configuration_instance_id: &str,
    expected_revision: u64,
    snapshot_bytes: &[u8],
) -> Result<u64, String>
where
    S: SettingsRegistryStore<Error = StoreError>,
{
    let (snapshot, _) = validate(
        store,
        registration_id,
        configuration_instance_id,
        expected_revision,
        snapshot_bytes,
    )?;
    store
        .commit_desired_settings_snapshot(&SettingsDesiredSnapshot {
            registration_id: registration_id.to_owned(),
            configuration_instance_id: configuration_instance_id.to_owned(),
            expected_revision,
            snapshot_bytes: snapshot.encode_to_vec(),
        })
        .map_err(|error| format!("{error:?}"))
}

fn validate<S>(
    store: &S,
    registration_id: &str,
    configuration_instance_id: &str,
    expected_revision: u64,
    snapshot_bytes: &[u8],
) -> Result<(SettingsSnapshotV1, [u8; 32]), String>
where
    S: SettingsRegistryStore<Error = StoreError>,
{
    let binding = store
        .settings_schema_binding(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "settings schema is not admitted for this registration".to_owned())?;
    let target = store
        .settings_configuration_target(registration_id, configuration_instance_id)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "settings configuration target does not exist".to_owned())?;
    if target.desired_revision() != expected_revision {
        return Err("settings desired revision conflicts with the current revision".to_owned());
    }
    let schema_bytes = store
        .settings_schema_artifact(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "settings schema artifact is unavailable".to_owned())?;
    let schema_sha256: [u8; 32] = Sha256::digest(&schema_bytes).into();
    if schema_sha256 != *binding.schema_sha256() {
        return Err("settings schema artifact does not match its binding".to_owned());
    }
    let schema = decode_settings_schema_v1(&schema_bytes)
        .map_err(|_| "settings schema artifact is invalid".to_owned())?;
    let snapshot = decode_settings_snapshot_v1(snapshot_bytes)
        .map_err(|_| "settings snapshot is invalid or exceeds protocol limits".to_owned())?;
    let next_revision = expected_revision
        .checked_add(1)
        .ok_or_else(|| "settings desired revision overflowed".to_owned())?;
    if snapshot.target_id != configuration_instance_id || snapshot.revision != next_revision {
        return Err("settings snapshot target or revision is invalid".to_owned());
    }
    validate_settings_snapshot_against_schema_v1(&schema, &snapshot)
        .map_err(|_| "settings snapshot does not match its schema".to_owned())?;
    for entry in &snapshot.values {
        let definition = schema
            .definitions
            .binary_search_by(|item| item.setting_id.cmp(&entry.setting_id))
            .map(|index| &schema.definitions[index])
            .map_err(|_| "settings snapshot references an unknown setting".to_owned())?;
        if definition.mutation_authority != SettingMutationAuthorityV1::OperatorManaged as i32
            || definition.client_visibility != SettingClientVisibilityV1::Editable as i32
        {
            return Err("settings snapshot contains a non-editable setting".to_owned());
        }
    }
    Ok((snapshot, schema_sha256))
}

fn hex_digest(digest: &[u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

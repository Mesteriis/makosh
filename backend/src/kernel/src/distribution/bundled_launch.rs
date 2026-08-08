//! Admits one verified signed-release module artifact as a durable managed launch binding.

use makosh_kernel_control_store::{BundledManagedLaunchBinding, ModuleRegistration};
use makosh_kernel_control_store_sqlite::SqliteControlStore;

use crate::distribution::bundle_verifier::VerifiedDistributionBundle;
use crate::modules::settings::schema as settings_schema;

pub fn admit(
    store: &SqliteControlStore,
    registration_id: &str,
    bundle: &VerifiedDistributionBundle,
    artifact_id: &str,
) -> Result<BundledManagedLaunchBinding, String> {
    let registration = store
        .module_registration(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "managed launch registration does not exist".to_owned())?;
    let artifact = bundle
        .artifacts()
        .iter()
        .find(|artifact| artifact.artifact_id() == artifact_id)
        .ok_or_else(|| "managed launch artifact is absent from distribution manifest".to_owned())?;
    validate_registration_contract(&registration, artifact)?;
    if let Some(schema_bytes) = artifact.settings_schema_bytes() {
        settings_schema::admit_bundled_and_materialize_initial(
            store,
            registration_id,
            artifact
                .module_descriptor_bytes()
                .ok_or_else(|| "managed launch artifact is not a module runtime".to_owned())?,
            schema_bytes,
        )?;
    }
    let binding = BundledManagedLaunchBinding::new(
        registration_id,
        next_binding_revision(store, registration_id)?,
        &bundle.manifest().distribution_id,
        artifact_id,
        *artifact.expected_sha256(),
        *artifact
            .descriptor_sha256()
            .ok_or_else(|| "managed launch artifact is not a module runtime".to_owned())?,
        artifact.settings_schema_sha256().copied(),
    );
    store
        .record_bundled_managed_launch_binding(&binding)
        .map_err(|error| format!("{error:?}"))?;
    Ok(binding)
}

fn validate_registration_contract(
    registration: &ModuleRegistration,
    artifact: &crate::distribution::bundle_verifier::VerifiedDistributionArtifact,
) -> Result<(), String> {
    let descriptor = artifact
        .module_descriptor()
        .ok_or_else(|| "managed launch artifact is not a module runtime".to_owned())?;
    if descriptor.module_id != registration.module_id()
        || artifact.descriptor_sha256() != Some(registration.descriptor_sha256())
    {
        return Err("managed launch artifact does not match its approved registration".to_owned());
    }
    Ok(())
}

fn next_binding_revision(store: &SqliteControlStore, registration_id: &str) -> Result<u64, String> {
    store
        .effective_bundled_managed_launch_binding(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .map_or(Ok(1), |binding| {
            binding
                .binding_revision()
                .checked_add(1)
                .ok_or_else(|| "managed launch binding revision overflowed".to_owned())
        })
}

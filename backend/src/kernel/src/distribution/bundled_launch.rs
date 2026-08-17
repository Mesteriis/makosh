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
    if let Some(current) = store
        .effective_bundled_managed_launch_binding(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .filter(|binding| exact_artifact_replay(binding, bundle, artifact_id, artifact))
    {
        return Ok(current);
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

fn exact_artifact_replay(
    binding: &BundledManagedLaunchBinding,
    bundle: &VerifiedDistributionBundle,
    artifact_id: &str,
    artifact: &crate::distribution::bundle_verifier::VerifiedDistributionArtifact,
) -> bool {
    exact_binding_matches(
        binding,
        &bundle.manifest().distribution_id,
        artifact_id,
        artifact.expected_sha256(),
        artifact.descriptor_sha256(),
        artifact.settings_schema_sha256(),
    )
}

fn exact_binding_matches(
    binding: &BundledManagedLaunchBinding,
    distribution_id: &str,
    artifact_id: &str,
    executable_sha256: &[u8; 32],
    descriptor_sha256: Option<&[u8; 32]>,
    settings_schema_sha256: Option<&[u8; 32]>,
) -> bool {
    binding.distribution_id() == distribution_id
        && binding.artifact_id() == artifact_id
        && binding.executable_sha256() == executable_sha256
        && descriptor_sha256 == Some(binding.descriptor_sha256())
        && settings_schema_sha256 == binding.settings_schema_sha256()
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

#[cfg(test)]
mod tests {
    use makosh_kernel_control_store::BundledManagedLaunchBinding;

    use super::exact_binding_matches;

    fn binding() -> BundledManagedLaunchBinding {
        BundledManagedLaunchBinding::new(
            "registration-a",
            7,
            "distribution-a",
            "artifact-a",
            [1; 32],
            [2; 32],
            Some([3; 32]),
        )
    }

    #[test]
    fn exact_byte_equivalent_artifact_reuses_the_current_binding() {
        assert!(exact_binding_matches(
            &binding(),
            "distribution-a",
            "artifact-a",
            &[1; 32],
            Some(&[2; 32]),
            Some(&[3; 32]),
        ));
    }

    #[test]
    fn any_launch_contract_change_requires_a_successor_binding() {
        assert!(!exact_binding_matches(
            &binding(),
            "distribution-a",
            "artifact-a",
            &[9; 32],
            Some(&[2; 32]),
            Some(&[3; 32]),
        ));
        assert!(!exact_binding_matches(
            &binding(),
            "distribution-a",
            "artifact-a",
            &[1; 32],
            Some(&[9; 32]),
            Some(&[3; 32]),
        ));
        assert!(!exact_binding_matches(
            &binding(),
            "distribution-a",
            "artifact-a",
            &[1; 32],
            Some(&[2; 32]),
            None,
        ));
    }
}

//! Connects canonical bundle validation to per-step PostgreSQL AST admission.

use makosh_storage_protocol::{
    v1::StorageBundleV1,
    validation::{StorageBundleValidationErrorV1, validate_storage_bundle},
};

use super::{MigrationAdmissionErrorV1, ast::admit_owner_local_additive_sql};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MigrationBundleAdmissionErrorV1 {
    Bundle(StorageBundleValidationErrorV1),
    Step {
        revision: u32,
        error: MigrationAdmissionErrorV1,
    },
}

pub fn admit_storage_bundle(
    bundle: &StorageBundleV1,
) -> Result<(), MigrationBundleAdmissionErrorV1> {
    validate_storage_bundle(bundle).map_err(MigrationBundleAdmissionErrorV1::Bundle)?;
    for step in &bundle.steps {
        let sql = std::str::from_utf8(&step.forward_sql_utf8)
            .expect("storage bundle validator already checked UTF-8");
        admit_owner_local_additive_sql(&bundle.owner_id, sql).map_err(|error| {
            MigrationBundleAdmissionErrorV1::Step {
                revision: step.revision,
                error,
            }
        })?;
    }
    Ok(())
}

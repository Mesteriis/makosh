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
    let exact_mail_person_source_admission = is_exact_mail_person_source_v32_step(bundle);
    let exact_bulk_action_rls_admission = is_exact_bulk_action_rls_v3_step(bundle);
    let exact_delayed_delivery_rls_admission = is_exact_delayed_delivery_rls_v5_step(bundle);
    let exact_ai_inference_rls_admission = is_exact_ai_inference_rls_v6_step(bundle);
    let exact_ollama_ai_rls_admission = is_exact_ollama_ai_rls_v5_step(bundle);
    let exact_speech_to_text_rls_admission = is_exact_speech_to_text_rls_v2_step(bundle);
    let exact_whisper_stt_rls_admission = is_exact_whisper_stt_rls_v2_step(bundle);
    let exact_telegram_owner_rls_admission = is_exact_telegram_owner_rls_v10_step(bundle);
    let exact_whatsapp_owner_rls_admission = is_exact_whatsapp_owner_rls_v5_step(bundle);
    let exact_zulip_owner_rls_admission = is_exact_zulip_owner_rls_v7_step(bundle);
    let exact_review_attention_rls_admission = is_exact_review_attention_rls_v3_step(bundle);
    let exact_review_task_candidate_rls_admission =
        is_exact_review_task_candidate_rls_v2_step(bundle);
    let exact_review_note_candidate_rls_admission =
        is_exact_review_note_candidate_rls_v2_step(bundle);
    let exact_tasks_lifecycle_rls_admission = is_exact_tasks_lifecycle_rls_v2_step(bundle);
    let exact_knowledge_lifecycle_rls_admission = is_exact_knowledge_lifecycle_rls_v2_step(bundle);
    let exact_task21_owner_rls_admission = is_exact_task21_owner_storage_bundle(bundle);
    if is_exact_persons_v3_bundle(bundle) {
        return Ok(());
    }
    if is_exact_mail_persons_sync_bundle(bundle) {
        return Ok(());
    }
    if is_exact_review_person_match_candidate_v1_bundle(bundle) {
        return Ok(());
    }
    if is_exact_reviewed_person_match_candidate_promotion_v1_bundle(bundle) {
        return Ok(());
    }
    if is_exact_calendar_v1_bundle(bundle) {
        return Ok(());
    }
    if is_exact_organizations_v1_bundle(bundle) {
        return Ok(());
    }
    if is_exact_documents_v1_bundle(bundle) {
        return Ok(());
    }
    if is_exact_relationships_v1_bundle(bundle) {
        return Ok(());
    }
    if is_exact_projects_v1_bundle(bundle) {
        return Ok(());
    }
    if is_exact_decisions_v1_bundle(bundle) {
        return Ok(());
    }
    if is_exact_identity_resolution_v1_bundle(bundle) {
        return Ok(());
    }
    if is_exact_search_v1_bundle(bundle) {
        return Ok(());
    }
    if is_exact_timeline_v1_bundle(bundle) {
        return Ok(());
    }
    if is_exact_graph_v1_bundle(bundle) {
        return Ok(());
    }
    if is_exact_memory_v1_bundle(bundle)
        || is_exact_consistency_v1_bundle(bundle)
        || is_exact_risk_v1_bundle(bundle)
        || is_exact_zoom_v1_bundle(bundle)
        || is_exact_telemost_v1_bundle(bundle)
        || is_exact_omniroute_v1_bundle(bundle)
    {
        return Ok(());
    }
    for step in &bundle.steps {
        if exact_mail_person_source_admission && step.revision == 32 {
            continue;
        }
        if exact_bulk_action_rls_admission && step.revision == 3 {
            continue;
        }
        if exact_delayed_delivery_rls_admission && step.revision == 5 {
            continue;
        }
        if exact_ai_inference_rls_admission && step.revision == 6 {
            continue;
        }
        if exact_ollama_ai_rls_admission && step.revision == 5 {
            continue;
        }
        if exact_speech_to_text_rls_admission && step.revision == 2 {
            continue;
        }
        if exact_whisper_stt_rls_admission && step.revision == 2 {
            continue;
        }
        if exact_telegram_owner_rls_admission && step.revision == 10 {
            continue;
        }
        if exact_whatsapp_owner_rls_admission && step.revision == 5 {
            continue;
        }
        if exact_zulip_owner_rls_admission && step.revision == 7 {
            continue;
        }
        if exact_review_attention_rls_admission && step.revision == 3 {
            continue;
        }
        if exact_review_task_candidate_rls_admission && step.revision == 2 {
            continue;
        }
        if exact_review_note_candidate_rls_admission && step.revision == 2 {
            continue;
        }
        if exact_tasks_lifecycle_rls_admission && step.revision == 2 {
            continue;
        }
        if exact_knowledge_lifecycle_rls_admission && step.revision == 2 {
            continue;
        }
        if exact_task21_owner_rls_admission && step.revision >= 2 {
            continue;
        }
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

fn is_exact_organizations_v1_bundle(bundle: &StorageBundleV1) -> bool {
    const SCHEMA_SHA256: [u8; 32] = [
        0x23, 0xc3, 0x15, 0xea, 0x12, 0x24, 0xb6, 0xbf, 0x97, 0x35, 0xae, 0xc6, 0x05, 0x3c, 0x77,
        0x3e, 0x6d, 0x93, 0xf7, 0x10, 0x34, 0xa7, 0xc1, 0x02, 0xd1, 0x94, 0x83, 0xe4, 0xf9, 0xf4,
        0x87, 0x11,
    ];
    bundle.major == 1
        && bundle.revision == 1
        && bundle.bundle_id == "organizations"
        && bundle.owner_id == "organizations"
        && matches!(bundle.steps.as_slice(), [step]
            if step.revision == 1
                && step.migration_id == "organizations_owner_initial"
                && step.sha256.as_slice() == SCHEMA_SHA256)
}

fn is_exact_documents_v1_bundle(bundle: &StorageBundleV1) -> bool {
    const SCHEMA_SHA256: [u8; 32] = [
        0x0c, 0xb2, 0x17, 0xc2, 0x69, 0x85, 0xda, 0x79, 0x0f, 0x0f, 0x41, 0x98, 0xa0, 0x39, 0xbe,
        0x5a, 0x7f, 0x6b, 0xcd, 0x82, 0x31, 0x20, 0x65, 0xe5, 0x61, 0xcb, 0x96, 0xc9, 0xa2, 0xcf,
        0xa2, 0x2e,
    ];
    bundle.major == 1
        && bundle.revision == 1
        && bundle.bundle_id == "documents"
        && bundle.owner_id == "documents"
        && matches!(bundle.steps.as_slice(), [step]
            if step.revision == 1
                && step.migration_id == "documents_owner_initial"
                && step.sha256.as_slice() == SCHEMA_SHA256)
}

fn is_exact_relationships_v1_bundle(bundle: &StorageBundleV1) -> bool {
    const SCHEMA_SHA256: [u8; 32] = [
        0xa8, 0x4c, 0xbe, 0x67, 0xea, 0x22, 0xdf, 0xab, 0x4f, 0x10, 0xec, 0x2e, 0x9d, 0xee, 0x23,
        0x1a, 0x1c, 0x55, 0xcc, 0x1c, 0xaf, 0x58, 0x50, 0x0f, 0xf1, 0xb3, 0xd3, 0xc9, 0x7d, 0xb3,
        0x86, 0xb9,
    ];
    bundle.major == 1
        && bundle.revision == 1
        && bundle.bundle_id == "relationships"
        && bundle.owner_id == "relationships"
        && matches!(bundle.steps.as_slice(), [step]
            if step.revision == 1
                && step.migration_id == "relationships_owner_initial"
                && step.sha256.as_slice() == SCHEMA_SHA256)
}

fn is_exact_projects_v1_bundle(bundle: &StorageBundleV1) -> bool {
    const SCHEMA_SHA256: [u8; 32] = [
        0x62, 0x0c, 0x6c, 0x41, 0xb1, 0x6c, 0xc7, 0x7e, 0x8f, 0x28, 0xd0, 0x15, 0xe8, 0x4b, 0xdc,
        0xe7, 0x5d, 0x0f, 0x91, 0xf3, 0x79, 0x06, 0xbc, 0x33, 0x8b, 0x53, 0x8d, 0xf9, 0x45, 0xc8,
        0x10, 0x7e,
    ];
    bundle.major == 1
        && bundle.revision == 1
        && bundle.bundle_id == "projects"
        && bundle.owner_id == "projects"
        && matches!(bundle.steps.as_slice(), [step]
            if step.revision == 1
                && step.migration_id == "projects_owner_initial"
                && step.sha256.as_slice() == SCHEMA_SHA256)
}

fn is_exact_decisions_v1_bundle(bundle: &StorageBundleV1) -> bool {
    const SCHEMA_SHA256: [u8; 32] = [
        0x74, 0x32, 0x50, 0x95, 0x0f, 0x82, 0xa2, 0xbe, 0x13, 0xa4, 0xa9, 0xbd, 0x92, 0x5f, 0x0c,
        0xd9, 0x57, 0x5f, 0x2a, 0x38, 0xd9, 0xb3, 0x03, 0x71, 0x7c, 0x4a, 0x49, 0xee, 0x2a, 0xd5,
        0xea, 0xfd,
    ];
    bundle.major == 1
        && bundle.revision == 1
        && bundle.bundle_id == "decisions"
        && bundle.owner_id == "decisions"
        && matches!(bundle.steps.as_slice(), [step]
            if step.revision == 1
                && step.migration_id == "decisions_owner_initial"
                && step.sha256.as_slice() == SCHEMA_SHA256)
}

fn is_exact_identity_resolution_v1_bundle(bundle: &StorageBundleV1) -> bool {
    const SCHEMA_SHA256: [u8; 32] = [
        0xd1, 0x30, 0x70, 0xb7, 0xfd, 0x3f, 0x17, 0x5a, 0xd4, 0xb3, 0xb9, 0x06, 0xdf, 0x07, 0x18,
        0x7d, 0x8d, 0x63, 0xb3, 0xb3, 0xce, 0x68, 0xd1, 0x6b, 0xb1, 0xd0, 0x6f, 0x89, 0x3d, 0xd5,
        0xd0, 0x52,
    ];
    bundle.major == 1
        && bundle.revision == 1
        && bundle.bundle_id == "identity_resolution"
        && bundle.owner_id == "identity_resolution"
        && matches!(bundle.steps.as_slice(),[step] if step.revision==1&&step.migration_id=="identity_resolution_initial"&&step.sha256.as_slice()==SCHEMA_SHA256)
}

fn is_exact_search_v1_bundle(bundle: &StorageBundleV1) -> bool {
    const SCHEMA_SHA256: [u8; 32] = [
        0xf8, 0x7f, 0x2e, 0x29, 0xd2, 0xc1, 0x41, 0xdb, 0xdb, 0x9d, 0x2e, 0x0c, 0x8a, 0xe1, 0x2f,
        0x10, 0x8f, 0x8d, 0xa1, 0x68, 0x8f, 0xb6, 0xa4, 0x35, 0x41, 0x32, 0xdf, 0x8c, 0x2b, 0x24,
        0xf3, 0x94,
    ];
    bundle.major == 1
        && bundle.revision == 1
        && bundle.bundle_id == "search"
        && bundle.owner_id == "search"
        && matches!(bundle.steps.as_slice(), [step]
            if step.revision == 1
                && step.migration_id == "search_projection_initial"
                && step.sha256.as_slice() == SCHEMA_SHA256)
}

fn is_exact_timeline_v1_bundle(bundle: &StorageBundleV1) -> bool {
    const SCHEMA_SHA256: [u8; 32] = [
        0x69, 0x9a, 0x78, 0x23, 0x80, 0x36, 0x73, 0xf5, 0xb3, 0x96, 0x5d, 0xc0, 0x5d, 0xb6, 0x18,
        0x19, 0x05, 0x80, 0xa2, 0x57, 0x70, 0x12, 0x95, 0xb5, 0x3e, 0xce, 0xa0, 0xdd, 0x22, 0x92,
        0x1d, 0x43,
    ];
    bundle.major == 1
        && bundle.revision == 1
        && bundle.bundle_id == "timeline"
        && bundle.owner_id == "timeline"
        && matches!(bundle.steps.as_slice(), [step]
            if step.revision == 1
                && step.migration_id == "timeline_projection_initial"
                && step.sha256.as_slice() == SCHEMA_SHA256)
}

fn is_exact_graph_v1_bundle(bundle: &StorageBundleV1) -> bool {
    const SCHEMA_SHA256: [u8; 32] = [
        0x54, 0xb5, 0x4b, 0xb3, 0xa9, 0x67, 0xc0, 0x86, 0xbc, 0xf6, 0xa3, 0x34, 0xa5, 0xb2, 0xda,
        0xa4, 0x52, 0x74, 0xb0, 0xa6, 0x8a, 0xc5, 0x34, 0x8c, 0x42, 0x76, 0x2c, 0x6d, 0x6d, 0x50,
        0x84, 0x52,
    ];
    bundle.major == 1
        && bundle.revision == 1
        && bundle.bundle_id == "graph"
        && bundle.owner_id == "graph"
        && matches!(bundle.steps.as_slice(), [step]
            if step.revision == 1
                && step.migration_id == "graph_projection_initial"
                && step.sha256.as_slice() == SCHEMA_SHA256)
}

fn is_exact_memory_v1_bundle(bundle: &StorageBundleV1) -> bool {
    const SHA: [u8; 32] = [
        0x3e, 0x68, 0x2d, 0x0c, 0x2e, 0x0b, 0xe1, 0x27, 0xe5, 0x99, 0xc9, 0x81, 0x7c, 0xbb, 0x4d,
        0xc1, 0x67, 0x56, 0xf6, 0x94, 0xe8, 0xac, 0x35, 0xa0, 0x7f, 0x8e, 0x33, 0x7c, 0x1d, 0x04,
        0x1e, 0xaa,
    ];
    exact_projection_bundle(bundle, "memory", "memory_projection_initial", &SHA)
}

fn is_exact_consistency_v1_bundle(bundle: &StorageBundleV1) -> bool {
    const SHA: [u8; 32] = [
        0xff, 0xdb, 0xdc, 0x3a, 0xee, 0x74, 0x61, 0xc2, 0x3f, 0x75, 0xf8, 0x03, 0x82, 0x09, 0xcc,
        0x36, 0xb7, 0x82, 0x32, 0xbd, 0xaf, 0xec, 0x1b, 0xe8, 0xe1, 0x1a, 0xc1, 0x05, 0xc5, 0x22,
        0x94, 0x4e,
    ];
    exact_projection_bundle(
        bundle,
        "consistency",
        "consistency_projection_initial",
        &SHA,
    )
}

fn is_exact_risk_v1_bundle(bundle: &StorageBundleV1) -> bool {
    const SHA: [u8; 32] = [
        0x17, 0xe1, 0xd3, 0xa7, 0x44, 0x64, 0xdb, 0x64, 0x29, 0x68, 0x82, 0x3a, 0x80, 0xa2, 0x6d,
        0x93, 0xff, 0x6f, 0x14, 0x35, 0xe6, 0xf9, 0xda, 0x1b, 0xfc, 0x44, 0x36, 0x12, 0xbf, 0xc9,
        0x64, 0xb0,
    ];
    exact_projection_bundle(bundle, "risk", "risk_projection_initial", &SHA)
}

fn is_exact_zoom_v1_bundle(bundle: &StorageBundleV1) -> bool {
    const SHA: [u8; 32] = [
        0x4e, 0x76, 0xfa, 0xe4, 0x84, 0x94, 0x4b, 0x5b, 0x3f, 0x7e, 0xc3, 0x45, 0x88, 0xe7, 0x41,
        0x8a, 0x1e, 0x4f, 0x01, 0x28, 0xa2, 0xea, 0xb4, 0xfe, 0x35, 0x5a, 0xb9, 0xdc, 0x58, 0xf4,
        0xdb, 0x9e,
    ];
    exact_projection_bundle(bundle, "zoom", "zoom_initial", &SHA)
}

fn is_exact_telemost_v1_bundle(bundle: &StorageBundleV1) -> bool {
    const SHA: [u8; 32] = [
        0x38, 0x41, 0x62, 0xdc, 0x4d, 0xb4, 0x04, 0xa9, 0x85, 0x0f, 0xc1, 0x1f, 0xd3, 0xee, 0x04,
        0xcf, 0xb0, 0x94, 0x1d, 0x47, 0xe4, 0x2c, 0xc5, 0xc8, 0xb9, 0x17, 0x00, 0x19, 0x1b, 0x73,
        0x12, 0x4b,
    ];
    exact_projection_bundle(bundle, "telemost", "telemost_initial", &SHA)
}

fn is_exact_omniroute_v1_bundle(bundle: &StorageBundleV1) -> bool {
    const SHA: [u8; 32] = [
        0xe4, 0xb9, 0x4a, 0x8e, 0xdb, 0x79, 0x70, 0xf4, 0xb4, 0x80, 0xe6, 0x8e, 0x40, 0x64, 0x6b,
        0x8b, 0xed, 0xb7, 0x64, 0x05, 0xb9, 0xe2, 0x35, 0x47, 0x5d, 0x16, 0xd2, 0xcd, 0x95, 0x1d,
        0x11, 0x7b,
    ];
    exact_projection_bundle(bundle, "omniroute", "omniroute_initial", &SHA)
}

fn exact_projection_bundle(
    bundle: &StorageBundleV1,
    owner: &str,
    migration_id: &str,
    sha: &[u8; 32],
) -> bool {
    bundle.major == 1
        && bundle.revision == 1
        && bundle.bundle_id == owner
        && bundle.owner_id == owner
        && matches!(bundle.steps.as_slice(), [step]
            if step.revision == 1
                && step.migration_id == migration_id
                && step.sha256.as_slice() == sha)
}

#[cfg(test)]
mod exact_zoom_telemost_omniroute_v1_tests {
    use std::path::Path;

    use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
    use sha2::{Digest, Sha256};

    use super::{
        admit_storage_bundle, is_exact_omniroute_v1_bundle, is_exact_telemost_v1_bundle,
        is_exact_zoom_v1_bundle,
    };

    fn fixture(owner: &str, migration: &str, path: &str) -> StorageBundleV1 {
        let sql =
            std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).expect("migration");
        StorageBundleV1 {
            major: 1,
            revision: 1,
            bundle_id: owner.to_owned(),
            owner_id: owner.to_owned(),
            steps: vec![StorageMigrationStepV1 {
                revision: 1,
                migration_id: migration.to_owned(),
                sha256: Sha256::digest(&sql).to_vec(),
                forward_sql_utf8: sql,
            }],
        }
    }

    fn assert_exact_only(exact: StorageBundleV1, predicate: impl Fn(&StorageBundleV1) -> bool) {
        assert!(predicate(&exact));
        admit_storage_bundle(&exact).expect("exact provider bundle");

        let mut alias = exact.clone();
        alias.owner_id.push_str("_alias");
        assert!(!predicate(&alias));
        assert!(admit_storage_bundle(&alias).is_err());

        let mut edited = exact;
        edited.steps[0]
            .forward_sql_utf8
            .extend_from_slice(b"\n-- edited\n");
        edited.steps[0].sha256 = Sha256::digest(&edited.steps[0].forward_sql_utf8).to_vec();
        assert!(admit_storage_bundle(&edited).is_err());
    }

    #[test]
    fn provider_bundles_require_exact_owner_and_bytes() {
        for (owner, migration, path, predicate) in [
            (
                "zoom",
                "zoom_initial",
                "../../../zoom-persistence/migrations/0001_zoom.sql",
                is_exact_zoom_v1_bundle as fn(&StorageBundleV1) -> bool,
            ),
            (
                "telemost",
                "telemost_initial",
                "../../../telemost-persistence/migrations/0001_telemost.sql",
                is_exact_telemost_v1_bundle,
            ),
            (
                "omniroute",
                "omniroute_initial",
                "../../../omniroute-persistence/migrations/0001_omniroute.sql",
                is_exact_omniroute_v1_bundle,
            ),
        ] {
            assert_exact_only(fixture(owner, migration, path), predicate);
        }
    }
}

#[cfg(test)]
mod exact_memory_consistency_risk_v1_tests {
    use std::path::Path;

    use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
    use sha2::{Digest, Sha256};

    use super::{
        admit_storage_bundle, is_exact_consistency_v1_bundle, is_exact_memory_v1_bundle,
        is_exact_risk_v1_bundle,
    };

    #[test]
    fn exact_projection_bundles_are_byte_bound() {
        for (owner, migration, path, exact) in [
            (
                "memory",
                "memory_projection_initial",
                "../../../memory-persistence/migrations/0001_memory.sql",
                is_exact_memory_v1_bundle as fn(&StorageBundleV1) -> bool,
            ),
            (
                "consistency",
                "consistency_projection_initial",
                "../../../consistency-persistence/migrations/0001_consistency.sql",
                is_exact_consistency_v1_bundle,
            ),
            (
                "risk",
                "risk_projection_initial",
                "../../../risk-persistence/migrations/0001_risk.sql",
                is_exact_risk_v1_bundle,
            ),
        ] {
            let sql = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap();
            let mut bundle = StorageBundleV1 {
                major: 1,
                revision: 1,
                bundle_id: owner.into(),
                owner_id: owner.into(),
                steps: vec![StorageMigrationStepV1 {
                    revision: 1,
                    migration_id: migration.into(),
                    sha256: Sha256::digest(&sql).to_vec(),
                    forward_sql_utf8: sql,
                }],
            };
            assert!(exact(&bundle));
            assert!(admit_storage_bundle(&bundle).is_ok());
            bundle.owner_id.push_str("-alias");
            assert!(!exact(&bundle));
            assert!(admit_storage_bundle(&bundle).is_err());
        }
    }
}

#[cfg(test)]
mod exact_timeline_graph_v1_tests {
    use std::path::Path;

    use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
    use sha2::{Digest, Sha256};

    use super::{admit_storage_bundle, is_exact_graph_v1_bundle, is_exact_timeline_v1_bundle};

    fn fixture(owner: &str, migration: &str, path: &str) -> StorageBundleV1 {
        let sql =
            std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).expect("migration");
        StorageBundleV1 {
            major: 1,
            revision: 1,
            bundle_id: owner.to_owned(),
            owner_id: owner.to_owned(),
            steps: vec![StorageMigrationStepV1 {
                revision: 1,
                migration_id: migration.to_owned(),
                sha256: Sha256::digest(&sql).to_vec(),
                forward_sql_utf8: sql,
            }],
        }
    }

    fn assert_exact_only(exact: StorageBundleV1, predicate: impl Fn(&StorageBundleV1) -> bool) {
        assert!(predicate(&exact));
        admit_storage_bundle(&exact).expect("exact projection bundle");
        let mut alias = exact.clone();
        alias.owner_id.push_str("_alias");
        assert!(!predicate(&alias));
        let mut edited = exact;
        edited.steps[0]
            .forward_sql_utf8
            .extend_from_slice(b"\n-- edited\n");
        edited.steps[0].sha256 = Sha256::digest(&edited.steps[0].forward_sql_utf8).to_vec();
        assert!(admit_storage_bundle(&edited).is_err());
    }

    #[test]
    fn timeline_and_graph_require_exact_bytes() {
        assert_exact_only(
            fixture(
                "timeline",
                "timeline_projection_initial",
                "../../../timeline-persistence/migrations/0001_timeline.sql",
            ),
            is_exact_timeline_v1_bundle,
        );
        assert_exact_only(
            fixture(
                "graph",
                "graph_projection_initial",
                "../../../graph-persistence/migrations/0001_graph.sql",
            ),
            is_exact_graph_v1_bundle,
        );
    }
}

#[cfg(test)]
mod exact_search_v1_tests {
    use std::path::Path;

    use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
    use sha2::{Digest, Sha256};

    use super::{admit_storage_bundle, is_exact_search_v1_bundle};

    fn fixture() -> StorageBundleV1 {
        let sql = std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../search-persistence/migrations/0001_search.sql"),
        )
        .expect("migration");
        StorageBundleV1 {
            major: 1,
            revision: 1,
            bundle_id: "search".to_owned(),
            owner_id: "search".to_owned(),
            steps: vec![StorageMigrationStepV1 {
                revision: 1,
                migration_id: "search_projection_initial".to_owned(),
                sha256: Sha256::digest(&sql).to_vec(),
                forward_sql_utf8: sql,
            }],
        }
    }

    #[test]
    fn exact_bytes_only() {
        let exact = fixture();
        assert!(is_exact_search_v1_bundle(&exact));
        admit_storage_bundle(&exact).expect("exact");
        let mut alias = exact.clone();
        alias.owner_id = "global_search".to_owned();
        assert!(!is_exact_search_v1_bundle(&alias));
        let mut edited = exact;
        edited.steps[0]
            .forward_sql_utf8
            .extend_from_slice(b"\n-- edited\n");
        edited.steps[0].sha256 = Sha256::digest(&edited.steps[0].forward_sql_utf8).to_vec();
        assert!(admit_storage_bundle(&edited).is_err());
    }
}

#[cfg(test)]
mod exact_identity_resolution_v1_tests {
    use super::{admit_storage_bundle, is_exact_identity_resolution_v1_bundle};
    use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
    use sha2::{Digest, Sha256};
    use std::path::Path;
    fn fixture() -> StorageBundleV1 {
        let sql = std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../../identity-resolution-persistence/migrations/0001_identity_resolution.sql",
        ))
        .expect("migration");
        StorageBundleV1 {
            major: 1,
            revision: 1,
            bundle_id: "identity_resolution".to_owned(),
            owner_id: "identity_resolution".to_owned(),
            steps: vec![StorageMigrationStepV1 {
                revision: 1,
                migration_id: "identity_resolution_initial".to_owned(),
                sha256: Sha256::digest(&sql).to_vec(),
                forward_sql_utf8: sql,
            }],
        }
    }
    #[test]
    fn exact_bytes_only() {
        let exact = fixture();
        assert!(is_exact_identity_resolution_v1_bundle(&exact));
        admit_storage_bundle(&exact).expect("exact");
        let mut alias = exact.clone();
        alias.owner_id = "identity-resolution".to_owned();
        assert!(!is_exact_identity_resolution_v1_bundle(&alias));
        let mut edited = exact;
        edited.steps[0]
            .forward_sql_utf8
            .extend_from_slice(b"\n-- edited\n");
        edited.steps[0].sha256 = Sha256::digest(&edited.steps[0].forward_sql_utf8).to_vec();
        assert!(admit_storage_bundle(&edited).is_err());
    }
}

#[cfg(test)]
mod exact_decisions_v1_tests {
    use std::path::Path;

    use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
    use sha2::{Digest, Sha256};

    use super::{admit_storage_bundle, is_exact_decisions_v1_bundle};

    fn fixture() -> StorageBundleV1 {
        let sql = std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../decisions-persistence/migrations/0001_decisions_owner.sql"),
        )
        .expect("Decisions migration");
        StorageBundleV1 {
            major: 1,
            revision: 1,
            bundle_id: "decisions".to_owned(),
            owner_id: "decisions".to_owned(),
            steps: vec![StorageMigrationStepV1 {
                revision: 1,
                migration_id: "decisions_owner_initial".to_owned(),
                sha256: Sha256::digest(&sql).to_vec(),
                forward_sql_utf8: sql,
            }],
        }
    }

    #[test]
    fn only_exact_owner_and_bytes_bypass_generic_force_rls_rejection() {
        let exact = fixture();
        assert!(is_exact_decisions_v1_bundle(&exact));
        admit_storage_bundle(&exact).expect("exact bundle");

        let mut alias = exact.clone();
        alias.owner_id = "decisions-alias".to_owned();
        assert!(!is_exact_decisions_v1_bundle(&alias));

        let mut edited = exact;
        edited.steps[0]
            .forward_sql_utf8
            .extend_from_slice(b"\n-- edited\n");
        edited.steps[0].sha256 = Sha256::digest(&edited.steps[0].forward_sql_utf8).to_vec();
        assert!(!is_exact_decisions_v1_bundle(&edited));
        assert!(admit_storage_bundle(&edited).is_err());
    }
}

#[cfg(test)]
mod exact_projects_v1_tests {
    use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};

    use super::{admit_storage_bundle, is_exact_projects_v1_bundle};

    const SHA: [u8; 32] = [
        0x62, 0x0c, 0x6c, 0x41, 0xb1, 0x6c, 0xc7, 0x7e, 0x8f, 0x28, 0xd0, 0x15, 0xe8, 0x4b, 0xdc,
        0xe7, 0x5d, 0x0f, 0x91, 0xf3, 0x79, 0x06, 0xbc, 0x33, 0x8b, 0x53, 0x8d, 0xf9, 0x45, 0xc8,
        0x10, 0x7e,
    ];

    fn bundle() -> StorageBundleV1 {
        StorageBundleV1 {
            major: 1,
            revision: 1,
            bundle_id: "projects".to_owned(),
            owner_id: "projects".to_owned(),
            steps: vec![StorageMigrationStepV1 {
                revision: 1,
                migration_id: "projects_owner_initial".to_owned(),
                forward_sql_utf8: b"reviewed Projects bytes".to_vec(),
                sha256: SHA.to_vec(),
            }],
        }
    }

    #[test]
    fn exception_is_exact_owner_bundle_step_and_digest() {
        let exact = bundle();
        assert!(is_exact_projects_v1_bundle(&exact));
        let mut alias = exact.clone();
        alias.owner_id = "projects-alias".to_owned();
        assert!(!is_exact_projects_v1_bundle(&alias));
        let mut edited = exact;
        edited.steps[0].forward_sql_utf8 =
            b"ALTER TABLE makosh_data.projects_records FORCE ROW LEVEL SECURITY; -- edited"
                .to_vec();
        edited.steps[0].sha256 = [9; 32].to_vec();
        assert!(!is_exact_projects_v1_bundle(&edited));
        assert!(admit_storage_bundle(&edited).is_err());
    }
}

#[cfg(test)]
mod exact_relationships_v1_tests {
    use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};

    use super::{admit_storage_bundle, is_exact_relationships_v1_bundle};

    const SHA: [u8; 32] = [
        0xa8, 0x4c, 0xbe, 0x67, 0xea, 0x22, 0xdf, 0xab, 0x4f, 0x10, 0xec, 0x2e, 0x9d, 0xee, 0x23,
        0x1a, 0x1c, 0x55, 0xcc, 0x1c, 0xaf, 0x58, 0x50, 0x0f, 0xf1, 0xb3, 0xd3, 0xc9, 0x7d, 0xb3,
        0x86, 0xb9,
    ];

    fn bundle() -> StorageBundleV1 {
        StorageBundleV1 {
            major: 1,
            revision: 1,
            bundle_id: "relationships".to_owned(),
            owner_id: "relationships".to_owned(),
            steps: vec![StorageMigrationStepV1 {
                revision: 1,
                migration_id: "relationships_owner_initial".to_owned(),
                forward_sql_utf8: b"reviewed Relationships bytes".to_vec(),
                sha256: SHA.to_vec(),
            }],
        }
    }

    #[test]
    fn exception_is_exact_owner_bundle_step_digest_and_bytes() {
        let exact = bundle();
        assert!(is_exact_relationships_v1_bundle(&exact));
        let mut alias = exact.clone();
        alias.owner_id = "relationships-alias".to_owned();
        assert!(!is_exact_relationships_v1_bundle(&alias));

        let mut edited = exact;
        edited.steps[0].forward_sql_utf8 =
            b"ALTER TABLE makosh_data.relationships_records FORCE ROW LEVEL SECURITY; -- edited"
                .to_vec();
        edited.steps[0].sha256 = [9; 32].to_vec();
        assert!(!is_exact_relationships_v1_bundle(&edited));
        assert!(admit_storage_bundle(&edited).is_err());
    }
}

#[cfg(test)]
mod exact_documents_v1_tests {
    use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};

    use super::{admit_storage_bundle, is_exact_documents_v1_bundle};

    const SHA: [u8; 32] = [
        0x0c, 0xb2, 0x17, 0xc2, 0x69, 0x85, 0xda, 0x79, 0x0f, 0x0f, 0x41, 0x98, 0xa0, 0x39, 0xbe,
        0x5a, 0x7f, 0x6b, 0xcd, 0x82, 0x31, 0x20, 0x65, 0xe5, 0x61, 0xcb, 0x96, 0xc9, 0xa2, 0xcf,
        0xa2, 0x2e,
    ];

    fn bundle() -> StorageBundleV1 {
        StorageBundleV1 {
            major: 1,
            revision: 1,
            bundle_id: "documents".to_owned(),
            owner_id: "documents".to_owned(),
            steps: vec![StorageMigrationStepV1 {
                revision: 1,
                migration_id: "documents_owner_initial".to_owned(),
                forward_sql_utf8: b"reviewed Documents bytes".to_vec(),
                sha256: SHA.to_vec(),
            }],
        }
    }

    #[test]
    fn exception_is_exact_owner_bundle_step_digest_and_bytes() {
        let exact = bundle();
        assert!(is_exact_documents_v1_bundle(&exact));
        let mut alias = exact.clone();
        alias.owner_id = "documents-alias".to_owned();
        assert!(!is_exact_documents_v1_bundle(&alias));

        let mut edited = exact;
        edited.steps[0].forward_sql_utf8 =
            b"ALTER TABLE makosh_data.documents_records FORCE ROW LEVEL SECURITY; -- edited"
                .to_vec();
        edited.steps[0].sha256 = [
            0xc9, 0x56, 0xcf, 0x70, 0x64, 0x79, 0x34, 0x9d, 0xa2, 0xe6, 0x45, 0xc1, 0xde, 0x24,
            0xfc, 0x4a, 0xbb, 0xa8, 0x8d, 0x90, 0xb1, 0x3d, 0x55, 0x88, 0x4b, 0x73, 0xea, 0x10,
            0xab, 0x52, 0x6c, 0x21,
        ]
        .to_vec();
        assert!(!is_exact_documents_v1_bundle(&edited));
        assert!(admit_storage_bundle(&edited).is_err());
    }
}

#[cfg(test)]
mod exact_organizations_v1_tests {
    use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};

    use super::{admit_storage_bundle, is_exact_organizations_v1_bundle};

    const SHA: [u8; 32] = [
        0x23, 0xc3, 0x15, 0xea, 0x12, 0x24, 0xb6, 0xbf, 0x97, 0x35, 0xae, 0xc6, 0x05, 0x3c, 0x77,
        0x3e, 0x6d, 0x93, 0xf7, 0x10, 0x34, 0xa7, 0xc1, 0x02, 0xd1, 0x94, 0x83, 0xe4, 0xf9, 0xf4,
        0x87, 0x11,
    ];

    fn bundle() -> StorageBundleV1 {
        StorageBundleV1 {
            major: 1,
            revision: 1,
            bundle_id: "organizations".to_owned(),
            owner_id: "organizations".to_owned(),
            steps: vec![StorageMigrationStepV1 {
                revision: 1,
                migration_id: "organizations_owner_initial".to_owned(),
                forward_sql_utf8: b"reviewed organizations bytes".to_vec(),
                sha256: SHA.to_vec(),
            }],
        }
    }

    #[test]
    fn exception_is_exact_owner_bundle_step_digest_and_bytes() {
        let exact = bundle();
        assert!(is_exact_organizations_v1_bundle(&exact));
        let mut alias = exact.clone();
        alias.owner_id = "organizations-alias".to_owned();
        assert!(!is_exact_organizations_v1_bundle(&alias));
        let mut edited = exact;
        edited.steps[0].forward_sql_utf8 =
            b"ALTER TABLE makosh_data.organizations FORCE ROW LEVEL SECURITY; -- edited".to_vec();
        edited.steps[0].sha256 = [
            0xd9, 0xd5, 0x40, 0xff, 0x7f, 0x82, 0x2d, 0xcb, 0x91, 0x7c, 0xd7, 0x44, 0xb6, 0x7e,
            0x80, 0x63, 0x04, 0xa2, 0x3c, 0x81, 0x6b, 0xaa, 0x17, 0x94, 0x1a, 0xf7, 0xfe, 0x4d,
            0x90, 0xac, 0x21, 0xe6,
        ]
        .to_vec();
        assert!(!is_exact_organizations_v1_bundle(&edited));
        assert!(admit_storage_bundle(&edited).is_err());
    }
}

fn is_exact_calendar_v1_bundle(bundle: &StorageBundleV1) -> bool {
    const CALENDAR_SCHEMA_SHA256: [u8; 32] = [
        0x7b, 0x6b, 0xb2, 0xc1, 0x53, 0x06, 0xc0, 0xa4, 0xb4, 0xf5, 0xdb, 0x1c, 0x98, 0x99, 0xea,
        0xd3, 0xac, 0xb5, 0x36, 0xdd, 0x45, 0x6b, 0xda, 0x2a, 0xb6, 0x09, 0x48, 0x2e, 0x7c, 0x09,
        0x98, 0xe8,
    ];
    bundle.major == 1
        && bundle.revision == 1
        && bundle.bundle_id == "calendar"
        && bundle.owner_id == "calendar"
        && matches!(bundle.steps.as_slice(), [step]
            if step.revision == 1
                && step.migration_id == "calendar_owner_initial"
                && step.sha256.as_slice() == CALENDAR_SCHEMA_SHA256)
}

#[cfg(test)]
mod exact_calendar_v1_tests {
    use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};

    use super::is_exact_calendar_v1_bundle;

    const SHA: [u8; 32] = [
        0x7b, 0x6b, 0xb2, 0xc1, 0x53, 0x06, 0xc0, 0xa4, 0xb4, 0xf5, 0xdb, 0x1c, 0x98, 0x99, 0xea,
        0xd3, 0xac, 0xb5, 0x36, 0xdd, 0x45, 0x6b, 0xda, 0x2a, 0xb6, 0x09, 0x48, 0x2e, 0x7c, 0x09,
        0x98, 0xe8,
    ];

    fn bundle() -> StorageBundleV1 {
        StorageBundleV1 {
            major: 1,
            revision: 1,
            bundle_id: "calendar".to_owned(),
            owner_id: "calendar".to_owned(),
            steps: vec![StorageMigrationStepV1 {
                revision: 1,
                migration_id: "calendar_owner_initial".to_owned(),
                forward_sql_utf8: b"reviewed calendar bytes".to_vec(),
                sha256: SHA.to_vec(),
            }],
        }
    }

    #[test]
    fn exception_is_exact_owner_bundle_step_and_digest() {
        let exact = bundle();
        assert!(is_exact_calendar_v1_bundle(&exact));
        let mut alias = exact.clone();
        alias.owner_id = "calendar-alias".to_owned();
        assert!(!is_exact_calendar_v1_bundle(&alias));
        let mut edited = exact;
        edited.steps[0].forward_sql_utf8 = b"edited".to_vec();
        edited.steps[0].sha256 = [9; 32].to_vec();
        assert!(!is_exact_calendar_v1_bundle(&edited));
    }
}

fn is_exact_knowledge_lifecycle_rls_v2_step(bundle: &StorageBundleV1) -> bool {
    const OWNER_RLS_SHA256: [u8; 32] = [
        0x67, 0xd6, 0x4e, 0x39, 0xf8, 0x15, 0x0e, 0x9c, 0x88, 0x9b, 0x6a, 0xc3, 0x54, 0x4e, 0xa8,
        0x89, 0xf9, 0x10, 0x10, 0xab, 0x1b, 0x7d, 0x58, 0x00, 0x5b, 0xe7, 0x37, 0x5c, 0xb0, 0x9e,
        0x4d, 0x6e,
    ];
    bundle.major == 1
        && bundle.revision == 2
        && bundle.bundle_id == "knowledge"
        && bundle.owner_id == "knowledge"
        && matches!(bundle.steps.last(), Some(step)
            if step.revision == 2
                && step.migration_id == "knowledge_lifecycle_owner_rls"
                && step.sha256.as_slice() == OWNER_RLS_SHA256)
}

#[cfg(test)]
mod exact_knowledge_lifecycle_rls_tests {
    use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};

    use super::is_exact_knowledge_lifecycle_rls_v2_step;

    const SHA: [u8; 32] = [
        0x67, 0xd6, 0x4e, 0x39, 0xf8, 0x15, 0x0e, 0x9c, 0x88, 0x9b, 0x6a, 0xc3, 0x54, 0x4e, 0xa8,
        0x89, 0xf9, 0x10, 0x10, 0xab, 0x1b, 0x7d, 0x58, 0x00, 0x5b, 0xe7, 0x37, 0x5c, 0xb0, 0x9e,
        0x4d, 0x6e,
    ];

    fn bundle() -> StorageBundleV1 {
        StorageBundleV1 {
            major: 1,
            revision: 2,
            bundle_id: "knowledge".to_owned(),
            owner_id: "knowledge".to_owned(),
            steps: vec![StorageMigrationStepV1 {
                revision: 2,
                migration_id: "knowledge_lifecycle_owner_rls".to_owned(),
                forward_sql_utf8: b"reviewed bytes".to_vec(),
                sha256: SHA.to_vec(),
            }],
        }
    }

    #[test]
    fn exception_is_exact_owner_bundle_id_revision_and_digest() {
        let exact = bundle();
        assert!(is_exact_knowledge_lifecycle_rls_v2_step(&exact));
        let mut alias = exact.clone();
        alias.owner_id = "knowledge-alias".to_owned();
        assert!(!is_exact_knowledge_lifecycle_rls_v2_step(&alias));
        let mut edited = exact;
        edited.steps[0].forward_sql_utf8 = b"edited".to_vec();
        edited.steps[0].sha256 = [9; 32].to_vec();
        assert!(!is_exact_knowledge_lifecycle_rls_v2_step(&edited));
    }
}

fn is_exact_tasks_lifecycle_rls_v2_step(bundle: &StorageBundleV1) -> bool {
    const OWNER_RLS_SHA256: [u8; 32] = [
        0xb0, 0x89, 0x98, 0x88, 0x77, 0xff, 0x28, 0x37, 0xe8, 0x94, 0xfd, 0x47, 0x41, 0x1a, 0x21,
        0xa7, 0xff, 0x4a, 0x00, 0xe9, 0x17, 0x08, 0x5f, 0xee, 0x29, 0x66, 0x47, 0xc4, 0xff, 0xbc,
        0x7a, 0xde,
    ];
    bundle.major == 1
        && bundle.revision == 2
        && bundle.bundle_id == "tasks"
        && bundle.owner_id == "tasks"
        && matches!(bundle.steps.last(), Some(step)
            if step.revision == 2
                && step.migration_id == "tasks_lifecycle_owner_rls"
                && step.sha256.as_slice() == OWNER_RLS_SHA256)
}

#[cfg(test)]
mod exact_tasks_lifecycle_rls_tests {
    use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};

    use super::is_exact_tasks_lifecycle_rls_v2_step;

    const SHA: [u8; 32] = [
        0xb0, 0x89, 0x98, 0x88, 0x77, 0xff, 0x28, 0x37, 0xe8, 0x94, 0xfd, 0x47, 0x41, 0x1a, 0x21,
        0xa7, 0xff, 0x4a, 0x00, 0xe9, 0x17, 0x08, 0x5f, 0xee, 0x29, 0x66, 0x47, 0xc4, 0xff, 0xbc,
        0x7a, 0xde,
    ];

    fn bundle() -> StorageBundleV1 {
        StorageBundleV1 {
            major: 1,
            revision: 2,
            bundle_id: "tasks".to_owned(),
            owner_id: "tasks".to_owned(),
            steps: vec![StorageMigrationStepV1 {
                revision: 2,
                migration_id: "tasks_lifecycle_owner_rls".to_owned(),
                forward_sql_utf8: b"reviewed bytes".to_vec(),
                sha256: SHA.to_vec(),
            }],
        }
    }

    #[test]
    fn exception_is_exact_owner_bundle_id_revision_and_digest() {
        let exact = bundle();
        assert!(is_exact_tasks_lifecycle_rls_v2_step(&exact));
        let mut alias = exact.clone();
        alias.owner_id = "tasks-alias".to_owned();
        assert!(!is_exact_tasks_lifecycle_rls_v2_step(&alias));
        let mut edited = exact;
        edited.steps[0].forward_sql_utf8 = b"edited".to_vec();
        edited.steps[0].sha256 = [9; 32].to_vec();
        assert!(!is_exact_tasks_lifecycle_rls_v2_step(&edited));
    }
}

// These admitted communication workflows predate owner-RLS support. Their
// append-only terminal migrations use FORCE RLS and CREATE POLICY, which sit
// outside the generic additive-DDL grammar. Exempt only the exact reviewed
// terminal bytes; all predecessor steps still traverse generic admission and
// any alias or edited/rehashed terminal step fails closed.
fn is_exact_bulk_action_rls_v3_step(bundle: &StorageBundleV1) -> bool {
    const OWNER_RLS_SHA256: [u8; 32] = [
        0x79, 0xf8, 0x42, 0x14, 0x67, 0x36, 0x3c, 0x3e, 0x09, 0x8c, 0xcf, 0x57, 0xee, 0x99, 0xc9,
        0xdc, 0xf6, 0xba, 0x15, 0x42, 0xdf, 0x6c, 0xca, 0x25, 0x8a, 0xb7, 0x85, 0xd4, 0x85, 0xd2,
        0xf3, 0x07,
    ];
    bundle.major == 1
        && bundle.revision == 3
        && bundle.bundle_id == "communication_bulk_action_state"
        && bundle.owner_id == "communication_bulk_action"
        && matches!(bundle.steps.last(), Some(step)
            if step.revision == 3
                && step.migration_id == "communication_bulk_action_owner_rls"
                && step.sha256.as_slice() == OWNER_RLS_SHA256)
}

fn is_exact_delayed_delivery_rls_v5_step(bundle: &StorageBundleV1) -> bool {
    const OWNER_RLS_SHA256: [u8; 32] = [
        0x6b, 0x04, 0x68, 0x06, 0xd3, 0x34, 0x68, 0x72, 0xf1, 0x07, 0x24, 0x06, 0xf6, 0x2d, 0xfd,
        0xa2, 0x3d, 0x2a, 0x57, 0xb7, 0xc6, 0x73, 0x65, 0xd1, 0x9f, 0x09, 0xad, 0xfb, 0xda, 0x10,
        0x98, 0x5d,
    ];
    bundle.major == 1
        && bundle.revision == 5
        && bundle.bundle_id == "communication_delayed_delivery_state"
        && bundle.owner_id == "communication_delayed_delivery"
        && matches!(bundle.steps.last(), Some(step)
            if step.revision == 5
                && step.migration_id == "communication_delayed_delivery_owner_rls"
                && step.sha256.as_slice() == OWNER_RLS_SHA256)
}

// AI and Ollama were implemented before owner-RLS admission. Their terminal
// migrations are append-only and outside the generic additive-DDL grammar.
// Exempt only the exact byte-bound terminal step; every predecessor remains
// subject to generic admission, and aliases or edited/rehashed SQL fail closed.
fn is_exact_ai_inference_rls_v6_step(bundle: &StorageBundleV1) -> bool {
    const OWNER_RLS_SHA256: [u8; 32] = [
        0xe2, 0x00, 0x91, 0x10, 0xc4, 0xdd, 0x11, 0x57, 0x69, 0xb7, 0x18, 0xd1, 0x2c, 0x9b, 0x51,
        0xc4, 0x19, 0xc7, 0x4b, 0x68, 0x6c, 0x18, 0x5b, 0x7e, 0xdb, 0x8f, 0xb6, 0xa2, 0x8d, 0x0a,
        0x34, 0x61,
    ];
    bundle.major == 1
        && bundle.revision == 6
        && bundle.bundle_id == "ai_inference_runs"
        && bundle.owner_id == "ai"
        && matches!(bundle.steps.last(), Some(step)
            if step.revision == 6
                && step.migration_id == "ai_inference_owner_rls"
                && step.sha256.as_slice() == OWNER_RLS_SHA256)
}

fn is_exact_ollama_ai_rls_v5_step(bundle: &StorageBundleV1) -> bool {
    const OWNER_RLS_SHA256: [u8; 32] = [
        0x51, 0x24, 0x14, 0xaf, 0x06, 0x14, 0xb6, 0x04, 0x5d, 0xa9, 0x5f, 0x2b, 0x9c, 0x66, 0xf1,
        0x03, 0x3b, 0x8a, 0x75, 0x50, 0x87, 0xec, 0x46, 0x5a, 0x1f, 0x71, 0x8c, 0xde, 0x96, 0x4a,
        0x77, 0xe5,
    ];
    bundle.major == 1
        && bundle.revision == 5
        && bundle.bundle_id == "ollama_ai_runs"
        && bundle.owner_id == "ollama"
        && matches!(bundle.steps.last(), Some(step)
            if step.revision == 5
                && step.migration_id == "ollama_ai_owner_rls"
                && step.sha256.as_slice() == OWNER_RLS_SHA256)
}

// Speech-to-Text and Whisper likewise predate owner-RLS admission. Permit only
// the byte-bound terminal step for each exact owner bundle; aliases and edited
// SQL still traverse the generic additive grammar and fail closed.
fn is_exact_speech_to_text_rls_v2_step(bundle: &StorageBundleV1) -> bool {
    const OWNER_RLS_SHA256: [u8; 32] = [
        0xdc, 0x00, 0x6a, 0xae, 0x06, 0x00, 0xa5, 0xc5, 0x30, 0x0d, 0xea, 0x8d, 0x1a, 0xf1, 0x4c,
        0x3c, 0xaf, 0x27, 0x07, 0x9f, 0xa4, 0x61, 0xc2, 0x78, 0x08, 0x00, 0x14, 0xbc, 0x6e, 0xec,
        0x98, 0x2b,
    ];
    bundle.major == 1
        && bundle.revision == 2
        && bundle.bundle_id == "speech_to_text"
        && bundle.owner_id == "speech_to_text"
        && matches!(bundle.steps.last(), Some(step)
            if step.revision == 2
                && step.migration_id == "speech_to_text_owner_rls"
                && step.sha256.as_slice() == OWNER_RLS_SHA256)
}

fn is_exact_whisper_stt_rls_v2_step(bundle: &StorageBundleV1) -> bool {
    const OWNER_RLS_SHA256: [u8; 32] = [
        0x4e, 0x9f, 0xde, 0x36, 0x2f, 0x20, 0xf1, 0x00, 0xc5, 0x37, 0x32, 0x8b, 0xe0, 0xd3, 0xf0,
        0x5f, 0xab, 0x2d, 0xdd, 0x61, 0x24, 0x55, 0x97, 0xb8, 0xf0, 0x87, 0xc8, 0x42, 0x54, 0x07,
        0x14, 0x01,
    ];
    bundle.major == 1
        && bundle.revision == 2
        && bundle.bundle_id == "whisper_stt_runs"
        && bundle.owner_id == "whisper_stt"
        && matches!(bundle.steps.last(), Some(step)
            if step.revision == 2
                && step.migration_id == "whisper_stt_owner_rls"
                && step.sha256.as_slice() == OWNER_RLS_SHA256)
}

// Telegram's revision-10 owner-scope migration applies FORCE RLS to the exact
// 45-table predecessor inventory plus its scope table. Its generated SQL is
// outside the generic additive grammar, so only the exact terminal bytes and
// owner/bundle identity are admitted.
fn is_exact_telegram_owner_rls_v10_step(bundle: &StorageBundleV1) -> bool {
    const OWNER_RLS_SHA256: [u8; 32] = [
        0xd5, 0xe7, 0xa9, 0x4b, 0xd3, 0x59, 0x64, 0x6e, 0xc0, 0x24, 0x71, 0x4e, 0x9f, 0xa7, 0x14,
        0x5e, 0xea, 0x07, 0x98, 0xba, 0x5c, 0xfd, 0x0d, 0x4b, 0x80, 0x0a, 0xa7, 0x83, 0x75, 0x6c,
        0xd6, 0x53,
    ];
    bundle.major == 1
        && bundle.revision == 10
        && bundle.bundle_id == "telegram_state"
        && bundle.owner_id == "telegram"
        && matches!(bundle.steps.last(), Some(step)
            if step.revision == 10
                && step.migration_id == "telegram_owner_scope_and_rls"
                && step.sha256.as_slice() == OWNER_RLS_SHA256)
}

// WhatsApp revision 5 applies FORCE RLS to its exact revision-4 table
// inventory plus one principal-lineage scope row. Only the byte-bound terminal
// step for the exact owner and bundle bypasses the additive-DDL grammar.
fn is_exact_whatsapp_owner_rls_v5_step(bundle: &StorageBundleV1) -> bool {
    const OWNER_RLS_SHA256: [u8; 32] = [
        0x24, 0xb5, 0x97, 0xc0, 0x03, 0xbd, 0x9f, 0x9b, 0xf0, 0xbc, 0x7e, 0xbc, 0x02, 0x60, 0x48,
        0x74, 0xfc, 0xeb, 0x61, 0x61, 0x17, 0xb5, 0x71, 0xc1, 0x9d, 0xc3, 0xec, 0xb3, 0x17, 0xae,
        0x7e, 0x22,
    ];
    bundle.major == 1
        && bundle.revision == 5
        && bundle.bundle_id == "whatsapp_state"
        && bundle.owner_id == "whatsapp"
        && matches!(bundle.steps.last(), Some(step)
            if step.revision == 5
                && step.migration_id == "whatsapp_owner_scope_and_rls"
                && step.sha256.as_slice() == OWNER_RLS_SHA256)
}

// Zulip revision 7 applies FORCE RLS to its exact revision-6 table inventory
// plus one principal-lineage scope row. Only the byte-bound terminal step for
// the exact owner and bundle bypasses the additive-DDL grammar.
fn is_exact_zulip_owner_rls_v7_step(bundle: &StorageBundleV1) -> bool {
    const OWNER_RLS_SHA256: [u8; 32] = [
        0xa5, 0x22, 0x02, 0x39, 0x3d, 0x28, 0xd8, 0x60, 0x33, 0x83, 0xd0, 0x21, 0xda, 0xef, 0xe7,
        0xf4, 0x43, 0x01, 0xd3, 0x8d, 0x29, 0x11, 0x14, 0x2f, 0x8d, 0x08, 0x2a, 0x63, 0x65, 0xec,
        0x8e, 0x3a,
    ];
    bundle.major == 1
        && bundle.revision == 7
        && bundle.bundle_id == "zulip_state"
        && bundle.owner_id == "zulip"
        && matches!(bundle.steps.last(), Some(step)
            if step.revision == 7
                && step.migration_id == "zulip_owner_scope_and_rls"
                && step.sha256.as_slice() == OWNER_RLS_SHA256)
}

// These Review stores predate owner-RLS admission. Their terminal migrations
// use FORCE RLS and CREATE POLICY outside the generic additive-DDL grammar.
// Admit only the exact owner, bundle, revision, migration ID, and reviewed
// terminal hash; aliases and edited/rehashed SQL remain fail-closed.
fn is_exact_review_attention_rls_v3_step(bundle: &StorageBundleV1) -> bool {
    const OWNER_RLS_SHA256: [u8; 32] = [
        0x02, 0x36, 0xfa, 0xf8, 0x7d, 0x2e, 0x18, 0x8e, 0xee, 0x28, 0x88, 0x11, 0xdc, 0x99, 0x20,
        0x9e, 0xae, 0xb2, 0x26, 0x78, 0x97, 0xec, 0x06, 0xc2, 0xb8, 0x63, 0xc7, 0x8a, 0x60, 0xc5,
        0x19, 0x88,
    ];
    bundle.major == 1
        && bundle.revision == 3
        && bundle.bundle_id == "review_attention_state"
        && bundle.owner_id == "review"
        && matches!(bundle.steps.last(), Some(step)
            if step.revision == 3
                && step.migration_id == "review_attention_owner_rls"
                && step.sha256.as_slice() == OWNER_RLS_SHA256)
}

fn is_exact_review_task_candidate_rls_v2_step(bundle: &StorageBundleV1) -> bool {
    const OWNER_RLS_SHA256: [u8; 32] = [
        0x20, 0x8f, 0x97, 0x4f, 0xf1, 0x65, 0xb1, 0xa3, 0xb8, 0x25, 0x48, 0x1b, 0x0c, 0x18, 0x4b,
        0x93, 0x99, 0x3b, 0x2e, 0x9e, 0x6e, 0x0d, 0xce, 0x91, 0x0a, 0xc5, 0xb4, 0x2b, 0x1e, 0x11,
        0x30, 0x4f,
    ];
    bundle.major == 1
        && bundle.revision == 2
        && bundle.bundle_id == "review_task_candidate"
        && bundle.owner_id == "review"
        && matches!(bundle.steps.last(), Some(step)
            if step.revision == 2
                && step.migration_id == "review_task_candidate_owner_rls"
                && step.sha256.as_slice() == OWNER_RLS_SHA256)
}

fn is_exact_review_note_candidate_rls_v2_step(bundle: &StorageBundleV1) -> bool {
    const OWNER_RLS_SHA256: [u8; 32] = [
        0x56, 0xa8, 0x58, 0xce, 0xa5, 0xd4, 0x63, 0xc8, 0xd9, 0x40, 0xc2, 0xb0, 0xbd, 0x11, 0x0e,
        0xc2, 0x5e, 0x5b, 0xf5, 0x08, 0x24, 0x64, 0xb4, 0xaa, 0x06, 0x54, 0x6b, 0x2c, 0xbd, 0xa5,
        0xb4, 0xc5,
    ];
    bundle.major == 1
        && bundle.revision == 2
        && bundle.bundle_id == "review_note_candidate"
        && bundle.owner_id == "review"
        && matches!(bundle.steps.last(), Some(step)
            if step.revision == 2
                && step.migration_id == "review_note_candidate_owner_rls"
                && step.sha256.as_slice() == OWNER_RLS_SHA256)
}

// Task 21 introduces three append-only FORCE RLS terminal migrations. Admit
// only their reviewed owner/bundle/migration identities and exact bytes; every
// predecessor continues through the generic additive grammar.
fn is_exact_task21_owner_storage_bundle(bundle: &StorageBundleV1) -> bool {
    const OBLIGATIONS_SHA256: [u8; 32] = [
        0x30, 0x90, 0x11, 0x01, 0xac, 0xe2, 0xfc, 0xeb, 0x79, 0xaa, 0xf3, 0x9e, 0xfd, 0xa0, 0x14,
        0x23, 0x88, 0x52, 0x2c, 0x1e, 0x88, 0xfb, 0x56, 0x04, 0x32, 0xf5, 0x56, 0x9b, 0x95, 0x32,
        0xeb, 0xce,
    ];
    const REVIEW_SHA256: [u8; 32] = [
        0x83, 0x64, 0xbc, 0x39, 0xf9, 0xc7, 0x56, 0xbf, 0xc3, 0xe0, 0xe1, 0x20, 0x90, 0xff, 0x56,
        0xdb, 0xf5, 0x57, 0x89, 0xf4, 0xce, 0xb8, 0xd3, 0x21, 0x9c, 0x71, 0x19, 0x69, 0x84, 0x0a,
        0xfa, 0xca,
    ];
    const PROMOTION_SHA256: [u8; 32] = [
        0x15, 0x25, 0x39, 0xcc, 0x19, 0xe5, 0x74, 0x6d, 0x71, 0xac, 0x5c, 0xa6, 0xac, 0x0b, 0x70,
        0xeb, 0x96, 0xc6, 0xc4, 0xfd, 0x52, 0x45, 0xe4, 0xe9, 0x49, 0xe9, 0x03, 0x8c, 0xd7, 0x63,
        0x2a, 0x8e,
    ];
    const OBLIGATIONS_V3_SHA256: [u8; 32] = [
        0xe0, 0xb9, 0x13, 0xff, 0x6f, 0xbf, 0x9c, 0xf3, 0xc5, 0x10, 0xb5, 0x71, 0x02, 0x69, 0xa9,
        0x60, 0x7e, 0xd9, 0x55, 0x36, 0x65, 0x19, 0x15, 0xb9, 0x3c, 0xeb, 0xce, 0xdd, 0x5c, 0xad,
        0xc8, 0x72,
    ];
    const REVIEW_V3_SHA256: [u8; 32] = [
        0xc9, 0xa1, 0x49, 0x9c, 0xc9, 0xf6, 0x38, 0x9e, 0x12, 0xba, 0xff, 0xef, 0xd5, 0xd8, 0x9e,
        0x80, 0xf4, 0x80, 0xe7, 0x0b, 0xb2, 0x62, 0xd3, 0xfd, 0xbc, 0x1f, 0xf7, 0x8a, 0xa9, 0xd2,
        0x21, 0x1b,
    ];
    if bundle.major != 1 {
        return false;
    }
    if bundle.bundle_id == "reviewed_obligation_candidate_promotion" {
        return bundle.revision == 2
            && bundle.owner_id == "reviewed_obligation_candidate_promotion"
            && matches!(bundle.steps.last(), Some(step)
                if step.revision == 2
                    && step.migration_id == "reviewed_obligation_candidate_promotion_owner_rls"
                    && step.sha256.as_slice() == PROMOTION_SHA256);
    }
    if bundle.revision != 3 || bundle.steps.len() != 3 {
        return false;
    }
    let owner_rls = &bundle.steps[1];
    let parties_evidence = &bundle.steps[2];
    (bundle.bundle_id == "obligations"
        && bundle.owner_id == "obligations"
        && owner_rls.revision == 2
        && owner_rls.migration_id == "obligations_lifecycle_owner_rls"
        && owner_rls.sha256.as_slice() == OBLIGATIONS_SHA256
        && parties_evidence.revision == 3
        && parties_evidence.migration_id == "obligations_parties_evidence"
        && parties_evidence.sha256.as_slice() == OBLIGATIONS_V3_SHA256)
        || (bundle.bundle_id == "review_obligation_candidate"
            && bundle.owner_id == "review"
            && owner_rls.revision == 2
            && owner_rls.migration_id == "review_obligation_candidate_owner_rls"
            && owner_rls.sha256.as_slice() == REVIEW_SHA256
            && parties_evidence.revision == 3
            && parties_evidence.migration_id == "review_obligation_candidate_parties_evidence"
            && parties_evidence.sha256.as_slice() == REVIEW_V3_SHA256)
}

#[cfg(test)]
mod exact_task21_owner_rls_tests {
    use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};

    use super::is_exact_task21_owner_storage_bundle;

    #[test]
    fn exceptions_reject_aliases_and_edited_rehashed_sql() {
        for (bundle_id, owner_id, migration_id, sha256, terminal) in [
            (
                "obligations",
                "obligations",
                "obligations_lifecycle_owner_rls",
                hex("30901101ace2fceb79aaf39efda0142388522c1e88fb560432f5569b9532ebce"),
                Some((
                    "obligations_parties_evidence",
                    hex("e0b913ff6fbf9cf3c510b5710269a9607ed95536651915b93cebcedd5cadc872"),
                )),
            ),
            (
                "review_obligation_candidate",
                "review",
                "review_obligation_candidate_owner_rls",
                hex("8364bc39f9c756bfc3e0e12090ff56dbf55789f4ceb8d3219c711969840afaca"),
                Some((
                    "review_obligation_candidate_parties_evidence",
                    hex("c9a1499cc9f6389e12baffefd5d89e80f480e70bb262d3fdbc1ff78aa9d2211b"),
                )),
            ),
            (
                "reviewed_obligation_candidate_promotion",
                "reviewed_obligation_candidate_promotion",
                "reviewed_obligation_candidate_promotion_owner_rls",
                hex("152539cc19e5746d71ac5ca6ac0b70eb96c6c4fd5245e4e949e9038cd7632a8e"),
                None,
            ),
        ] {
            let exact = bundle(bundle_id, owner_id, migration_id, sha256, terminal);
            assert!(is_exact_task21_owner_storage_bundle(&exact));
            let mut alias = exact.clone();
            alias.bundle_id.push_str("_alias");
            assert!(!is_exact_task21_owner_storage_bundle(&alias));
            let mut edited = exact;
            let last = edited.steps.last_mut().expect("terminal step");
            last.forward_sql_utf8 = b"edited".to_vec();
            last.sha256 = vec![0x7f; 32];
            assert!(!is_exact_task21_owner_storage_bundle(&edited));
        }
    }

    fn bundle(
        bundle_id: &str,
        owner_id: &str,
        migration_id: &str,
        sha256: [u8; 32],
        terminal: Option<(&str, [u8; 32])>,
    ) -> StorageBundleV1 {
        let revision = if terminal.is_some() { 3 } else { 2 };
        let mut steps = vec![
            StorageMigrationStepV1 {
                revision: 1,
                migration_id: "initial".to_owned(),
                forward_sql_utf8: b"CREATE TABLE makosh_data.seed(id BIGINT);".to_vec(),
                sha256: vec![1; 32],
            },
            StorageMigrationStepV1 {
                revision: 2,
                migration_id: migration_id.to_owned(),
                forward_sql_utf8: b"reviewed exact bytes".to_vec(),
                sha256: sha256.to_vec(),
            },
        ];
        if let Some((migration_id, sha256)) = terminal {
            steps.push(StorageMigrationStepV1 {
                revision: 3,
                migration_id: migration_id.to_owned(),
                forward_sql_utf8: b"reviewed exact terminal bytes".to_vec(),
                sha256: sha256.to_vec(),
            });
        }
        StorageBundleV1 {
            major: 1,
            revision,
            bundle_id: bundle_id.to_owned(),
            owner_id: owner_id.to_owned(),
            steps,
        }
    }

    fn hex(value: &str) -> [u8; 32] {
        let mut bytes = [0_u8; 32];
        for (index, chunk) in value.as_bytes().chunks_exact(2).enumerate() {
            bytes[index] =
                u8::from_str_radix(std::str::from_utf8(chunk).expect("hex"), 16).expect("hex byte");
        }
        bytes
    }
}

#[cfg(test)]
mod exact_communication_owner_rls_tests {
    use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};

    use super::{
        is_exact_ai_inference_rls_v6_step, is_exact_bulk_action_rls_v3_step,
        is_exact_delayed_delivery_rls_v5_step, is_exact_ollama_ai_rls_v5_step,
        is_exact_review_attention_rls_v3_step, is_exact_review_note_candidate_rls_v2_step,
        is_exact_review_task_candidate_rls_v2_step, is_exact_speech_to_text_rls_v2_step,
        is_exact_telegram_owner_rls_v10_step, is_exact_whatsapp_owner_rls_v5_step,
        is_exact_whisper_stt_rls_v2_step, is_exact_zulip_owner_rls_v7_step,
    };

    const BULK_RLS_SHA256: [u8; 32] = [
        0x79, 0xf8, 0x42, 0x14, 0x67, 0x36, 0x3c, 0x3e, 0x09, 0x8c, 0xcf, 0x57, 0xee, 0x99, 0xc9,
        0xdc, 0xf6, 0xba, 0x15, 0x42, 0xdf, 0x6c, 0xca, 0x25, 0x8a, 0xb7, 0x85, 0xd4, 0x85, 0xd2,
        0xf3, 0x07,
    ];
    const DELAYED_RLS_SHA256: [u8; 32] = [
        0x6b, 0x04, 0x68, 0x06, 0xd3, 0x34, 0x68, 0x72, 0xf1, 0x07, 0x24, 0x06, 0xf6, 0x2d, 0xfd,
        0xa2, 0x3d, 0x2a, 0x57, 0xb7, 0xc6, 0x73, 0x65, 0xd1, 0x9f, 0x09, 0xad, 0xfb, 0xda, 0x10,
        0x98, 0x5d,
    ];
    const AI_RLS_SHA256: [u8; 32] = [
        0xe2, 0x00, 0x91, 0x10, 0xc4, 0xdd, 0x11, 0x57, 0x69, 0xb7, 0x18, 0xd1, 0x2c, 0x9b, 0x51,
        0xc4, 0x19, 0xc7, 0x4b, 0x68, 0x6c, 0x18, 0x5b, 0x7e, 0xdb, 0x8f, 0xb6, 0xa2, 0x8d, 0x0a,
        0x34, 0x61,
    ];
    const OLLAMA_RLS_SHA256: [u8; 32] = [
        0x51, 0x24, 0x14, 0xaf, 0x06, 0x14, 0xb6, 0x04, 0x5d, 0xa9, 0x5f, 0x2b, 0x9c, 0x66, 0xf1,
        0x03, 0x3b, 0x8a, 0x75, 0x50, 0x87, 0xec, 0x46, 0x5a, 0x1f, 0x71, 0x8c, 0xde, 0x96, 0x4a,
        0x77, 0xe5,
    ];
    const SPEECH_RLS_SHA256: [u8; 32] = [
        0xdc, 0x00, 0x6a, 0xae, 0x06, 0x00, 0xa5, 0xc5, 0x30, 0x0d, 0xea, 0x8d, 0x1a, 0xf1, 0x4c,
        0x3c, 0xaf, 0x27, 0x07, 0x9f, 0xa4, 0x61, 0xc2, 0x78, 0x08, 0x00, 0x14, 0xbc, 0x6e, 0xec,
        0x98, 0x2b,
    ];
    const WHISPER_RLS_SHA256: [u8; 32] = [
        0x4e, 0x9f, 0xde, 0x36, 0x2f, 0x20, 0xf1, 0x00, 0xc5, 0x37, 0x32, 0x8b, 0xe0, 0xd3, 0xf0,
        0x5f, 0xab, 0x2d, 0xdd, 0x61, 0x24, 0x55, 0x97, 0xb8, 0xf0, 0x87, 0xc8, 0x42, 0x54, 0x07,
        0x14, 0x01,
    ];
    const TELEGRAM_RLS_SHA256: [u8; 32] = [
        0xd5, 0xe7, 0xa9, 0x4b, 0xd3, 0x59, 0x64, 0x6e, 0xc0, 0x24, 0x71, 0x4e, 0x9f, 0xa7, 0x14,
        0x5e, 0xea, 0x07, 0x98, 0xba, 0x5c, 0xfd, 0x0d, 0x4b, 0x80, 0x0a, 0xa7, 0x83, 0x75, 0x6c,
        0xd6, 0x53,
    ];
    const WHATSAPP_RLS_SHA256: [u8; 32] = [
        0x24, 0xb5, 0x97, 0xc0, 0x03, 0xbd, 0x9f, 0x9b, 0xf0, 0xbc, 0x7e, 0xbc, 0x02, 0x60, 0x48,
        0x74, 0xfc, 0xeb, 0x61, 0x61, 0x17, 0xb5, 0x71, 0xc1, 0x9d, 0xc3, 0xec, 0xb3, 0x17, 0xae,
        0x7e, 0x22,
    ];
    const ZULIP_RLS_SHA256: [u8; 32] = [
        0xa5, 0x22, 0x02, 0x39, 0x3d, 0x28, 0xd8, 0x60, 0x33, 0x83, 0xd0, 0x21, 0xda, 0xef, 0xe7,
        0xf4, 0x43, 0x01, 0xd3, 0x8d, 0x29, 0x11, 0x14, 0x2f, 0x8d, 0x08, 0x2a, 0x63, 0x65, 0xec,
        0x8e, 0x3a,
    ];
    const REVIEW_ATTENTION_RLS_SHA256: [u8; 32] = [
        0x02, 0x36, 0xfa, 0xf8, 0x7d, 0x2e, 0x18, 0x8e, 0xee, 0x28, 0x88, 0x11, 0xdc, 0x99, 0x20,
        0x9e, 0xae, 0xb2, 0x26, 0x78, 0x97, 0xec, 0x06, 0xc2, 0xb8, 0x63, 0xc7, 0x8a, 0x60, 0xc5,
        0x19, 0x88,
    ];
    const REVIEW_TASK_RLS_SHA256: [u8; 32] = [
        0x20, 0x8f, 0x97, 0x4f, 0xf1, 0x65, 0xb1, 0xa3, 0xb8, 0x25, 0x48, 0x1b, 0x0c, 0x18, 0x4b,
        0x93, 0x99, 0x3b, 0x2e, 0x9e, 0x6e, 0x0d, 0xce, 0x91, 0x0a, 0xc5, 0xb4, 0x2b, 0x1e, 0x11,
        0x30, 0x4f,
    ];
    const REVIEW_NOTE_RLS_SHA256: [u8; 32] = [
        0x56, 0xa8, 0x58, 0xce, 0xa5, 0xd4, 0x63, 0xc8, 0xd9, 0x40, 0xc2, 0xb0, 0xbd, 0x11, 0x0e,
        0xc2, 0x5e, 0x5b, 0xf5, 0x08, 0x24, 0x64, 0xb4, 0xaa, 0x06, 0x54, 0x6b, 0x2c, 0xbd, 0xa5,
        0xb4, 0xc5,
    ];

    #[test]
    fn exceptions_are_bound_to_only_the_reviewed_terminal_steps() {
        let bulk = bundle(
            3,
            "communication_bulk_action_state",
            "communication_bulk_action",
            "communication_bulk_action_owner_rls",
            BULK_RLS_SHA256,
        );
        assert!(is_exact_bulk_action_rls_v3_step(&bulk));
        let mut bulk_alias = bulk.clone();
        bulk_alias.owner_id = "communication_bulk_action_alias".to_owned();
        assert!(!is_exact_bulk_action_rls_v3_step(&bulk_alias));
        let mut bulk_rehashed_edit = bulk;
        bulk_rehashed_edit.steps[0].forward_sql_utf8 = b"edited".to_vec();
        bulk_rehashed_edit.steps[0].sha256 = vec![
            0x1f, 0xb9, 0xf4, 0x09, 0x72, 0x56, 0xdb, 0x2d, 0x7b, 0x1e, 0x13, 0xaf, 0xf7, 0x9c,
            0xee, 0x44, 0x33, 0x98, 0x91, 0xa3, 0x1c, 0x55, 0x6b, 0x9c, 0xf6, 0x09, 0x38, 0x85,
            0x77, 0x3b, 0x36, 0x18,
        ];
        assert!(!is_exact_bulk_action_rls_v3_step(&bulk_rehashed_edit));

        let delayed = bundle(
            5,
            "communication_delayed_delivery_state",
            "communication_delayed_delivery",
            "communication_delayed_delivery_owner_rls",
            DELAYED_RLS_SHA256,
        );
        assert!(is_exact_delayed_delivery_rls_v5_step(&delayed));
        let mut delayed_alias = delayed.clone();
        delayed_alias.bundle_id = "communication_delayed_delivery_alias".to_owned();
        assert!(!is_exact_delayed_delivery_rls_v5_step(&delayed_alias));
        let mut delayed_rehashed_edit = delayed;
        delayed_rehashed_edit.steps[0].forward_sql_utf8 = b"edited".to_vec();
        delayed_rehashed_edit.steps[0].sha256 = bulk_rehashed_edit.steps[0].sha256.clone();
        assert!(!is_exact_delayed_delivery_rls_v5_step(
            &delayed_rehashed_edit
        ));

        let ai = bundle(
            6,
            "ai_inference_runs",
            "ai",
            "ai_inference_owner_rls",
            AI_RLS_SHA256,
        );
        assert!(is_exact_ai_inference_rls_v6_step(&ai));
        let mut ai_alias = ai.clone();
        ai_alias.owner_id = "ai_alias".to_owned();
        assert!(!is_exact_ai_inference_rls_v6_step(&ai_alias));
        let mut ai_mutated = ai;
        ai_mutated.steps[0].sha256 = [0x55; 32].to_vec();
        assert!(!is_exact_ai_inference_rls_v6_step(&ai_mutated));

        let ollama = bundle(
            5,
            "ollama_ai_runs",
            "ollama",
            "ollama_ai_owner_rls",
            OLLAMA_RLS_SHA256,
        );
        assert!(is_exact_ollama_ai_rls_v5_step(&ollama));
        let mut ollama_alias = ollama.clone();
        ollama_alias.bundle_id = "ollama_ai_alias".to_owned();
        assert!(!is_exact_ollama_ai_rls_v5_step(&ollama_alias));
        let mut ollama_mutated = ollama;
        ollama_mutated.steps[0].sha256 = [0xaa; 32].to_vec();
        assert!(!is_exact_ollama_ai_rls_v5_step(&ollama_mutated));

        let speech = bundle(
            2,
            "speech_to_text",
            "speech_to_text",
            "speech_to_text_owner_rls",
            SPEECH_RLS_SHA256,
        );
        assert!(is_exact_speech_to_text_rls_v2_step(&speech));
        let mut speech_alias = speech.clone();
        speech_alias.owner_id = "speech_to_text_alias".to_owned();
        assert!(!is_exact_speech_to_text_rls_v2_step(&speech_alias));
        let mut speech_mutated = speech;
        speech_mutated.steps[0].sha256 = [0xbb; 32].to_vec();
        assert!(!is_exact_speech_to_text_rls_v2_step(&speech_mutated));

        let whisper = bundle(
            2,
            "whisper_stt_runs",
            "whisper_stt",
            "whisper_stt_owner_rls",
            WHISPER_RLS_SHA256,
        );
        assert!(is_exact_whisper_stt_rls_v2_step(&whisper));
        let mut whisper_alias = whisper.clone();
        whisper_alias.bundle_id = "whisper_stt_alias".to_owned();
        assert!(!is_exact_whisper_stt_rls_v2_step(&whisper_alias));
        let mut whisper_mutated = whisper;
        whisper_mutated.steps[0].sha256 = [0xcc; 32].to_vec();
        assert!(!is_exact_whisper_stt_rls_v2_step(&whisper_mutated));

        let telegram = bundle(
            10,
            "telegram_state",
            "telegram",
            "telegram_owner_scope_and_rls",
            TELEGRAM_RLS_SHA256,
        );
        assert!(is_exact_telegram_owner_rls_v10_step(&telegram));
        let mut telegram_alias = telegram.clone();
        telegram_alias.owner_id = "telegram_alias".to_owned();
        assert!(!is_exact_telegram_owner_rls_v10_step(&telegram_alias));
        let mut telegram_mutated = telegram;
        telegram_mutated.steps[0].sha256 = [0xdd; 32].to_vec();
        assert!(!is_exact_telegram_owner_rls_v10_step(&telegram_mutated));

        let whatsapp = bundle(
            5,
            "whatsapp_state",
            "whatsapp",
            "whatsapp_owner_scope_and_rls",
            WHATSAPP_RLS_SHA256,
        );
        assert!(is_exact_whatsapp_owner_rls_v5_step(&whatsapp));
        let mut whatsapp_alias = whatsapp.clone();
        whatsapp_alias.owner_id = "whatsapp_alias".to_owned();
        assert!(!is_exact_whatsapp_owner_rls_v5_step(&whatsapp_alias));
        let mut whatsapp_mutated = whatsapp;
        whatsapp_mutated.steps[0].forward_sql_utf8 = b"edited".to_vec();
        whatsapp_mutated.steps[0].sha256 = [0xee; 32].to_vec();
        assert!(!is_exact_whatsapp_owner_rls_v5_step(&whatsapp_mutated));

        let zulip = bundle(
            7,
            "zulip_state",
            "zulip",
            "zulip_owner_scope_and_rls",
            ZULIP_RLS_SHA256,
        );
        assert!(is_exact_zulip_owner_rls_v7_step(&zulip));
        let mut zulip_alias = zulip.clone();
        zulip_alias.bundle_id = "zulip_state_alias".to_owned();
        assert!(!is_exact_zulip_owner_rls_v7_step(&zulip_alias));
        let mut zulip_mutated = zulip;
        zulip_mutated.steps[0].forward_sql_utf8 = b"edited".to_vec();
        zulip_mutated.steps[0].sha256 = [0xef; 32].to_vec();
        assert!(!is_exact_zulip_owner_rls_v7_step(&zulip_mutated));

        for (bundle, matcher) in [
            (
                bundle(
                    3,
                    "review_attention_state",
                    "review",
                    "review_attention_owner_rls",
                    REVIEW_ATTENTION_RLS_SHA256,
                ),
                is_exact_review_attention_rls_v3_step as fn(&StorageBundleV1) -> bool,
            ),
            (
                bundle(
                    2,
                    "review_task_candidate",
                    "review",
                    "review_task_candidate_owner_rls",
                    REVIEW_TASK_RLS_SHA256,
                ),
                is_exact_review_task_candidate_rls_v2_step,
            ),
            (
                bundle(
                    2,
                    "review_note_candidate",
                    "review",
                    "review_note_candidate_owner_rls",
                    REVIEW_NOTE_RLS_SHA256,
                ),
                is_exact_review_note_candidate_rls_v2_step,
            ),
        ] {
            assert!(matcher(&bundle));
            let mut alias = bundle.clone();
            alias.owner_id = "review_alias".to_owned();
            assert!(!matcher(&alias));
            let mut mutated = bundle;
            mutated.steps[0].forward_sql_utf8 = b"edited".to_vec();
            mutated.steps[0].sha256 = [0xf0; 32].to_vec();
            assert!(!matcher(&mutated));
        }
    }

    fn bundle(
        revision: u32,
        bundle_id: &str,
        owner_id: &str,
        migration_id: &str,
        sha256: [u8; 32],
    ) -> StorageBundleV1 {
        StorageBundleV1 {
            major: 1,
            revision,
            bundle_id: bundle_id.to_owned(),
            owner_id: owner_id.to_owned(),
            steps: vec![StorageMigrationStepV1 {
                revision,
                migration_id: migration_id.to_owned(),
                forward_sql_utf8: b"reviewed-bytes-validated-by-bundle".to_vec(),
                sha256: sha256.to_vec(),
            }],
        }
    }
}

// Mail revision 32 appends the reviewed FORCE-RLS Person-source schema to the
// established Mail lineage. Exempt only that final byte-bound step; every
// predecessor step still traverses the generic additive SQL admission above.
// An edited/rehashed revision 32 therefore loses the exception, while no
// unrelated Mail bundle or later revision inherits it.
fn is_exact_mail_person_source_v32_step(bundle: &StorageBundleV1) -> bool {
    const PERSON_SOURCE_SHA256: [u8; 32] = [
        0xfb, 0xb3, 0x01, 0x61, 0x1d, 0xc8, 0x67, 0xd3, 0x4a, 0x26, 0xe9, 0x37, 0xec, 0x37, 0xda,
        0xd1, 0x5d, 0x0e, 0x50, 0x05, 0xff, 0x65, 0x1e, 0xe2, 0xc0, 0x93, 0x07, 0xff, 0xd6, 0x2d,
        0x52, 0x14,
    ];
    bundle.major == 1
        && bundle.revision == 32
        && bundle.bundle_id == "mail_state"
        && bundle.owner_id == "mail"
        && matches!(bundle.steps.last(), Some(step)
            if step.revision == 32
                && step.migration_id == "mail_address_book_person_source_admitted"
                && step.sha256.as_slice() == PERSON_SOURCE_SHA256)
}

// The dormant reviewed Person-match promotion workflow also uses owner-local
// FORCE RLS. Keep its exception byte-bound to the single reviewed initial
// migration; aliases and edited/rehashed SQL fall back to fail-closed AST
// admission.
fn is_exact_reviewed_person_match_candidate_promotion_v1_bundle(bundle: &StorageBundleV1) -> bool {
    const INITIAL_SHA256: [u8; 32] = [
        0x04, 0x56, 0xad, 0x28, 0xa9, 0x0b, 0x92, 0x05, 0x90, 0x14, 0x29, 0x78, 0xec, 0x73, 0x0e,
        0xca, 0xff, 0x1d, 0x8a, 0x6a, 0x47, 0x15, 0xb8, 0xeb, 0x56, 0x61, 0x80, 0x77, 0x51, 0x14,
        0xe9, 0xd4,
    ];
    bundle.major == 1
        && bundle.revision == 1
        && bundle.bundle_id == "reviewed_person_match_candidate_promotion"
        && bundle.owner_id == "reviewed_person_match_candidate_promotion"
        && matches!(bundle.steps.as_slice(), [initial]
            if initial.revision == 1
                && initial.migration_id == "reviewed_person_match_candidate_promotion_initial"
                && initial.sha256.as_slice() == INITIAL_SHA256)
}

// The dormant Review Person-match queue is FORCE-RLS owner storage. Admit
// only the exact reviewed initial bytes; every alias or edited/rehashed step
// returns to the generic fail-closed AST contour.
fn is_exact_review_person_match_candidate_v1_bundle(bundle: &StorageBundleV1) -> bool {
    const INITIAL_SHA256: [u8; 32] = [
        0x5b, 0x64, 0x01, 0x93, 0x49, 0xbd, 0x71, 0xb8, 0xd3, 0xa0, 0xc1, 0x3a, 0xe1, 0x13, 0xc5,
        0x3b, 0x4e, 0xc7, 0x99, 0x5d, 0xc7, 0x59, 0x97, 0x46, 0x2f, 0x79, 0xd6, 0xf8, 0xe6, 0x9d,
        0xa6, 0xc4,
    ];
    bundle.major == 1
        && bundle.revision == 1
        && bundle.bundle_id == "review_person_match_candidate"
        && bundle.owner_id == "review"
        && matches!(bundle.steps.as_slice(), [initial]
            if initial.revision == 1
                && initial.migration_id == "review_person_match_candidate_initial"
                && initial.sha256.as_slice() == INITIAL_SHA256)
}

// The dormant Mail-to-Person workflow uses owner-scoped FORCE RLS in its
// initial schema. PostgreSQL represents those statements outside the generic
// additive-DDL grammar, so admit only the exact reviewed bytes. Any owner,
// bundle, migration name, revision, or SQL change returns to fail-closed AST
// admission.
fn is_exact_mail_persons_sync_bundle(bundle: &StorageBundleV1) -> bool {
    const INITIAL_SHA256: [u8; 32] = [
        0x2a, 0xc6, 0x91, 0xeb, 0xcd, 0xe3, 0x18, 0x1b, 0x15, 0x2d, 0xfb, 0x2b, 0xf2, 0x10, 0xed,
        0xa1, 0xb3, 0x54, 0xcf, 0x4f, 0xc4, 0x42, 0x14, 0xdc, 0x26, 0x8e, 0x3e, 0xa1, 0xff, 0xab,
        0xa5, 0x0c,
    ];
    const ACCOUNT_SCHEDULER_BINDING_SHA256: [u8; 32] = [
        0x02, 0x63, 0x6d, 0x0e, 0x04, 0x51, 0x8d, 0x8d, 0x7a, 0xb0, 0xab, 0x47, 0x5c, 0x03, 0x53,
        0xd8, 0x3c, 0x8c, 0xaf, 0xd7, 0xf3, 0xa3, 0x41, 0x93, 0xa8, 0x17, 0x14, 0xbb, 0x2f, 0x46,
        0x5e, 0x26,
    ];
    if bundle.major != 1 {
        return false;
    }
    if bundle.bundle_id != "mail_persons_sync" || bundle.owner_id != "mail_persons_sync" {
        return false;
    }
    match bundle.steps.as_slice() {
        [initial] => {
            bundle.revision == 1
                && initial.revision == 1
                && initial.migration_id == "mail_persons_sync_initial"
                && initial.sha256.as_slice() == INITIAL_SHA256
        }
        [initial, account_scheduler_binding] => {
            bundle.revision == 2
                && initial.revision == 1
                && initial.migration_id == "mail_persons_sync_initial"
                && initial.sha256.as_slice() == INITIAL_SHA256
                && account_scheduler_binding.revision == 2
                && account_scheduler_binding.migration_id
                    == "mail_persons_sync_account_scheduler_binding"
                && account_scheduler_binding.sha256.as_slice() == ACCOUNT_SCHEDULER_BINDING_SHA256
        }
        _ => false,
    }
}

// Persons V3 contains the audited immutable-history trigger/RLS policy and the
// additive durable outbox-order upgrade.
// Statements remain outside the generic additive-DDL grammar. Keep the
// exception byte-bound to the exact reviewed three-step bundle; no
// owner/name alias or newly edited SQL inherits this admission.
fn is_exact_persons_v3_bundle(bundle: &StorageBundleV1) -> bool {
    const INITIAL_SHA256: [u8; 32] = [
        0xcf, 0xdb, 0x08, 0x13, 0xf8, 0x48, 0x85, 0x64, 0x08, 0xa8, 0x99, 0x6d, 0x76, 0x85, 0xfd,
        0xe1, 0xd3, 0x5e, 0x72, 0xd0, 0x2c, 0x15, 0x8a, 0x0e, 0xbc, 0x06, 0x9e, 0xbe, 0x54, 0xd7,
        0xea, 0xcd,
    ];
    const DURABLE_SHA256: [u8; 32] = [
        0x70, 0x26, 0x6e, 0x14, 0x76, 0x8c, 0x15, 0x0e, 0x7e, 0xc0, 0xa2, 0xb3, 0x96, 0x14, 0xd6,
        0x92, 0x03, 0x1a, 0x62, 0x93, 0x3e, 0x95, 0x00, 0xac, 0x63, 0x53, 0x3a, 0xbb, 0xe0, 0x53,
        0xe4, 0x45,
    ];
    const OUTBOX_ORDER_SHA256: [u8; 32] = [
        0x6a, 0x9d, 0x99, 0xcb, 0xa1, 0x7a, 0xa1, 0x82, 0x2a, 0x40, 0xee, 0x27, 0xab, 0xd9, 0x02,
        0x81, 0xb6, 0xf1, 0x6f, 0xdb, 0x9f, 0xb3, 0x1a, 0x5e, 0x2a, 0xaa, 0x2f, 0xb4, 0x89, 0x4b,
        0x53, 0x55,
    ];
    bundle.major == 1
        && bundle.revision == 3
        && bundle.bundle_id == "persons"
        && bundle.owner_id == "persons"
        && matches!(bundle.steps.as_slice(), [initial, durable, outbox_order]
            if initial.revision == 1
                && initial.migration_id == "persons_initial"
                && initial.sha256.as_slice() == INITIAL_SHA256
                && durable.revision == 2
                && durable.migration_id == "persons_durable"
                && durable.sha256.as_slice() == DURABLE_SHA256
                && outbox_order.revision == 3
                && outbox_order.migration_id == "persons_outbox_order"
                && outbox_order.sha256.as_slice() == OUTBOX_ORDER_SHA256)
}

#[cfg(test)]
mod exact_mail_person_source_v32_tests {
    use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};

    use super::is_exact_mail_person_source_v32_step;

    #[test]
    fn exception_is_bound_to_only_the_reviewed_mail_v32_terminal_step() {
        let exact = StorageBundleV1 {
            major: 1,
            revision: 32,
            bundle_id: "mail_state".to_owned(),
            owner_id: "mail".to_owned(),
            steps: vec![StorageMigrationStepV1 {
                revision: 32,
                migration_id: "mail_address_book_person_source_admitted".to_owned(),
                forward_sql_utf8: b"reviewed-bytes-validated-by-bundle".to_vec(),
                sha256: vec![
                    0xfb, 0xb3, 0x01, 0x61, 0x1d, 0xc8, 0x67, 0xd3, 0x4a, 0x26, 0xe9, 0x37, 0xec,
                    0x37, 0xda, 0xd1, 0x5d, 0x0e, 0x50, 0x05, 0xff, 0x65, 0x1e, 0xe2, 0xc0, 0x93,
                    0x07, 0xff, 0xd6, 0x2d, 0x52, 0x14,
                ],
            }],
        };
        assert!(is_exact_mail_person_source_v32_step(&exact));

        let mut alias = exact.clone();
        alias.owner_id = "mail_alias".to_owned();
        assert!(!is_exact_mail_person_source_v32_step(&alias));

        let mut rehashed = exact;
        rehashed.steps[0].sha256 = vec![0x55; 32];
        assert!(!is_exact_mail_person_source_v32_step(&rehashed));
    }
}

#[cfg(test)]
mod exact_mail_persons_sync_tests {
    use super::admit_storage_bundle;
    use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
    use makosh_storage_protocol::validation::validate_storage_bundle;

    const INITIAL: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../mail-persons-sync-persistence/migrations/0001_mail_persons_sync.sql"
    ));
    const ACCOUNT_SCHEDULER_BINDING: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../mail-persons-sync-persistence/migrations/0002_account_scheduler_binding.sql"
    ));

    #[test]
    fn admits_only_the_exact_reviewed_mail_persons_sync_v1_bundle() {
        let exact = exact_bundle();
        admit_storage_bundle(&exact).expect("exact Mail Persons Sync V1 bundle");

        let mut alias = exact.clone();
        alias.bundle_id = "mail_persons_sync_alias".to_owned();
        assert!(admit_storage_bundle(&alias).is_err());

        let mut mutated = exact;
        mutated.steps[0]
            .forward_sql_utf8
            .extend_from_slice(b" SELECT 1;");
        mutated.steps[0].sha256 = vec![
            0x3a, 0x81, 0xf0, 0x5b, 0xa4, 0x7e, 0x15, 0x15, 0x31, 0x99, 0x98, 0x7b, 0xd7, 0x55,
            0x0a, 0x7c, 0xed, 0xb0, 0x77, 0x2e, 0x61, 0xce, 0x01, 0xa7, 0xc8, 0xc4, 0x9a, 0x56,
            0xaf, 0xbd, 0x50, 0x71,
        ];
        validate_storage_bundle(&mutated).expect("mutated bundle digest is exact");
        assert!(admit_storage_bundle(&mutated).is_err());
    }

    #[test]
    fn admits_only_the_exact_reviewed_mail_persons_sync_v2_successor() {
        let mut exact = exact_bundle();
        exact.revision = 2;
        exact.steps.push(StorageMigrationStepV1 {
            revision: 2,
            migration_id: "mail_persons_sync_account_scheduler_binding".to_owned(),
            forward_sql_utf8: ACCOUNT_SCHEDULER_BINDING.to_vec(),
            sha256: vec![
                0x02, 0x63, 0x6d, 0x0e, 0x04, 0x51, 0x8d, 0x8d, 0x7a, 0xb0, 0xab, 0x47, 0x5c, 0x03,
                0x53, 0xd8, 0x3c, 0x8c, 0xaf, 0xd7, 0xf3, 0xa3, 0x41, 0x93, 0xa8, 0x17, 0x14, 0xbb,
                0x2f, 0x46, 0x5e, 0x26,
            ],
        });
        admit_storage_bundle(&exact).expect("exact Mail Persons Sync V2 bundle");

        let mut rehashed = exact;
        rehashed.steps[1].sha256 = vec![0x44; 32];
        assert!(admit_storage_bundle(&rehashed).is_err());
    }

    fn exact_bundle() -> StorageBundleV1 {
        StorageBundleV1 {
            major: 1,
            revision: 1,
            bundle_id: "mail_persons_sync".to_owned(),
            owner_id: "mail_persons_sync".to_owned(),
            steps: vec![StorageMigrationStepV1 {
                revision: 1,
                migration_id: "mail_persons_sync_initial".to_owned(),
                forward_sql_utf8: INITIAL.to_vec(),
                sha256: vec![
                    0x2a, 0xc6, 0x91, 0xeb, 0xcd, 0xe3, 0x18, 0x1b, 0x15, 0x2d, 0xfb, 0x2b, 0xf2,
                    0x10, 0xed, 0xa1, 0xb3, 0x54, 0xcf, 0x4f, 0xc4, 0x42, 0x14, 0xdc, 0x26, 0x8e,
                    0x3e, 0xa1, 0xff, 0xab, 0xa5, 0x0c,
                ],
            }],
        }
    }
}

#[cfg(test)]
mod exact_review_person_match_candidate_tests {
    use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};

    use super::admit_storage_bundle;

    const INITIAL: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../review-person-match-candidate-persistence/migrations/0001_review_person_match_candidate.sql"
    ));

    #[test]
    fn admits_only_exact_review_person_match_candidate_bundle() {
        let exact = exact_bundle();
        admit_storage_bundle(&exact).expect("exact Review Person Match bundle");
        let mut alias = exact.clone();
        alias.bundle_id.push_str("_alias");
        assert!(admit_storage_bundle(&alias).is_err());
        let mut mutated = exact;
        mutated.steps[0]
            .forward_sql_utf8
            .extend_from_slice(b" SELECT 1;");
        mutated.steps[0].sha256 = vec![
            0xa3, 0xfb, 0xef, 0x9d, 0x85, 0x32, 0x1d, 0x8d, 0x8d, 0x9a, 0x60, 0x23, 0xbb, 0x6a,
            0x98, 0xa3, 0x96, 0x3b, 0x7d, 0x47, 0xf6, 0xa6, 0x7b, 0x1e, 0x9f, 0x52, 0xbb, 0xe1,
            0xd0, 0xc9, 0xf1, 0x9e,
        ];
        assert!(admit_storage_bundle(&mutated).is_err());
    }

    fn exact_bundle() -> StorageBundleV1 {
        StorageBundleV1 {
            major: 1,
            revision: 1,
            bundle_id: "review_person_match_candidate".to_owned(),
            owner_id: "review".to_owned(),
            steps: vec![StorageMigrationStepV1 {
                revision: 1,
                migration_id: "review_person_match_candidate_initial".to_owned(),
                forward_sql_utf8: INITIAL.to_vec(),
                sha256: vec![
                    0x5b, 0x64, 0x01, 0x93, 0x49, 0xbd, 0x71, 0xb8, 0xd3, 0xa0, 0xc1, 0x3a, 0xe1,
                    0x13, 0xc5, 0x3b, 0x4e, 0xc7, 0x99, 0x5d, 0xc7, 0x59, 0x97, 0x46, 0x2f, 0x79,
                    0xd6, 0xf8, 0xe6, 0x9d, 0xa6, 0xc4,
                ],
            }],
        }
    }
}

#[cfg(test)]
mod exact_reviewed_person_match_candidate_promotion_tests {
    use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::admit_storage_bundle;

    const INITIAL: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../reviewed-person-match-candidate-promotion-persistence/migrations/0001_reviewed_person_match_candidate_promotion.sql"
    ));

    #[test]
    fn admits_only_exact_reviewed_person_match_candidate_promotion_bundle() {
        let exact = exact_bundle();
        admit_storage_bundle(&exact).expect("exact reviewed Person match promotion bundle");

        let mut alias = exact.clone();
        alias.owner_id.push_str("_alias");
        assert!(admit_storage_bundle(&alias).is_err());

        let mut mutated = exact;
        mutated.steps[0]
            .forward_sql_utf8
            .extend_from_slice(b" SELECT 1;");
        mutated.steps[0].sha256 = vec![
            0xfb, 0x0e, 0x4e, 0xeb, 0xa3, 0xc6, 0x51, 0x95, 0x68, 0x3b, 0x18, 0xec, 0x8d, 0xa1,
            0x21, 0xa3, 0x8f, 0x94, 0x26, 0xa1, 0x9d, 0xce, 0x94, 0xb4, 0x64, 0x7c, 0xd5, 0x76,
            0x5f, 0xe3, 0xe4, 0x81,
        ];
        validate_storage_bundle(&mutated).expect("mutated digest remains structurally valid");
        assert!(admit_storage_bundle(&mutated).is_err());
    }

    fn exact_bundle() -> StorageBundleV1 {
        StorageBundleV1 {
            major: 1,
            revision: 1,
            bundle_id: "reviewed_person_match_candidate_promotion".to_owned(),
            owner_id: "reviewed_person_match_candidate_promotion".to_owned(),
            steps: vec![StorageMigrationStepV1 {
                revision: 1,
                migration_id: "reviewed_person_match_candidate_promotion_initial".to_owned(),
                forward_sql_utf8: INITIAL.to_vec(),
                sha256: vec![
                    0x04, 0x56, 0xad, 0x28, 0xa9, 0x0b, 0x92, 0x05, 0x90, 0x14, 0x29, 0x78, 0xec,
                    0x73, 0x0e, 0xca, 0xff, 0x1d, 0x8a, 0x6a, 0x47, 0x15, 0xb8, 0xeb, 0x56, 0x61,
                    0x80, 0x77, 0x51, 0x14, 0xe9, 0xd4,
                ],
            }],
        }
    }
}

#[cfg(test)]
mod exact_persons_tests {
    use makosh_storage_protocol::v1::{StorageBundleV1, StorageMigrationStepV1};
    use makosh_storage_protocol::validation::validate_storage_bundle;

    use super::admit_storage_bundle;

    const INITIAL: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../persons-persistence/migrations/0001_persons.sql"
    ));
    const DURABLE: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../persons-persistence/migrations/0002_persons_durable.sql"
    ));
    const OUTBOX_ORDER: &[u8] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../persons-persistence/migrations/0003_persons_outbox_order.sql"
    ));

    #[test]
    fn admits_only_the_exact_reviewed_persons_v3_bundle() {
        let exact = exact_bundle();
        admit_storage_bundle(&exact).expect("exact Persons V3 bundle");

        let mut alias = exact.clone();
        alias.steps[0].migration_id = "persons_initial_alias".to_owned();
        assert!(admit_storage_bundle(&alias).is_err());

        let mut mutated = exact;
        mutated.steps[0]
            .forward_sql_utf8
            .extend_from_slice(b" SELECT 1;");
        // Rebind the canonical bundle digest to the changed bytes. This first
        // proves ordinary bundle validation still succeeds, so the rejection
        // below exercises the exact reviewed-byte admission rather than the
        // generic hash-mismatch guard.
        mutated.steps[0].sha256 = vec![
            0x71, 0xb3, 0xc4, 0x47, 0x9b, 0x5b, 0x39, 0xfb, 0xf3, 0x39, 0x5b, 0xba, 0x78, 0xf4,
            0x04, 0x4d, 0x11, 0x8b, 0xcb, 0x0d, 0x69, 0xaa, 0x53, 0xe6, 0xdf, 0x74, 0x10, 0xa1,
            0xf2, 0x54, 0xe6, 0xb1,
        ];
        validate_storage_bundle(&mutated).expect("mutated bundle digest is exact");
        assert!(admit_storage_bundle(&mutated).is_err());
    }

    fn exact_bundle() -> StorageBundleV1 {
        StorageBundleV1 {
            major: 1,
            revision: 3,
            bundle_id: "persons".to_owned(),
            owner_id: "persons".to_owned(),
            steps: vec![
                StorageMigrationStepV1 {
                    revision: 1,
                    migration_id: "persons_initial".to_owned(),
                    forward_sql_utf8: INITIAL.to_vec(),
                    sha256: vec![
                        0xcf, 0xdb, 0x08, 0x13, 0xf8, 0x48, 0x85, 0x64, 0x08, 0xa8, 0x99, 0x6d,
                        0x76, 0x85, 0xfd, 0xe1, 0xd3, 0x5e, 0x72, 0xd0, 0x2c, 0x15, 0x8a, 0x0e,
                        0xbc, 0x06, 0x9e, 0xbe, 0x54, 0xd7, 0xea, 0xcd,
                    ],
                },
                StorageMigrationStepV1 {
                    revision: 2,
                    migration_id: "persons_durable".to_owned(),
                    forward_sql_utf8: DURABLE.to_vec(),
                    sha256: vec![
                        0x70, 0x26, 0x6e, 0x14, 0x76, 0x8c, 0x15, 0x0e, 0x7e, 0xc0, 0xa2, 0xb3,
                        0x96, 0x14, 0xd6, 0x92, 0x03, 0x1a, 0x62, 0x93, 0x3e, 0x95, 0x00, 0xac,
                        0x63, 0x53, 0x3a, 0xbb, 0xe0, 0x53, 0xe4, 0x45,
                    ],
                },
                StorageMigrationStepV1 {
                    revision: 3,
                    migration_id: "persons_outbox_order".to_owned(),
                    forward_sql_utf8: OUTBOX_ORDER.to_vec(),
                    sha256: vec![
                        0x6a, 0x9d, 0x99, 0xcb, 0xa1, 0x7a, 0xa1, 0x82, 0x2a, 0x40, 0xee, 0x27,
                        0xab, 0xd9, 0x02, 0x81, 0xb6, 0xf1, 0x6f, 0xdb, 0x9f, 0xb3, 0x1a, 0x5e,
                        0x2a, 0xaa, 0x2f, 0xb4, 0x89, 0x4b, 0x53, 0x55,
                    ],
                },
            ],
        }
    }
}

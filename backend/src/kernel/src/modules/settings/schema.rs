//! Verifies and persists the immutable SettingsSchema artifact for one module registration.

use makosh_kernel_control_store::{
    ModuleRegistrationState, ModuleRegistryStore, SettingsApplyState, SettingsConfigurationTarget,
    SettingsConfigurationTargetInputV1, SettingsInitialSnapshot, SettingsRegistryStore,
    SettingsSchemaBinding, SettingsSchemaBindingInputV1, SettingsSchemaTargetSuccessor,
};
use makosh_kernel_control_store_sqlite::StoreError;
use makosh_runtime_protocol::v1::{SettingsSchemaV1, SettingsSnapshotV1, SettingsValueEntryV1};
use makosh_runtime_protocol::validation::descriptor::{
    decode_descriptor_v1, decode_settings_schema_v1, decode_settings_snapshot_v1,
    settings_snapshot_is_complete_v1, validate_settings_snapshot_against_schema_v1,
};
use prost::Message;
use sha2::{Digest, Sha256};

pub fn admit<S>(
    store: &S,
    registration_id: &str,
    descriptor_bytes: &[u8],
    schema_bytes: &[u8],
) -> Result<SettingsSchemaBinding, String>
where
    S: ModuleRegistryStore<Error = StoreError> + SettingsRegistryStore<Error = StoreError>,
{
    let binding = validated_binding(store, registration_id, descriptor_bytes, schema_bytes)?;
    store
        .admit_settings_schema(&binding, schema_bytes)
        .map_err(|error| format!("{error:?}"))?;
    Ok(binding)
}

pub fn admit_bundled_and_materialize_initial<S>(
    store: &S,
    registration_id: &str,
    descriptor_bytes: &[u8],
    schema_bytes: &[u8],
) -> Result<SettingsSchemaBinding, String>
where
    S: ModuleRegistryStore<Error = StoreError> + SettingsRegistryStore<Error = StoreError>,
{
    let binding = validated_binding(store, registration_id, descriptor_bytes, schema_bytes)?;
    let existing = store
        .settings_schema_binding(registration_id)
        .map_err(|error| format!("{error:?}"))?;
    match existing {
        None => store
            .admit_settings_schema(&binding, schema_bytes)
            .map_err(|error| format!("{error:?}"))?,
        Some(existing) if same_schema(&existing, &binding) => {
            let existing_bytes = store
                .settings_schema_artifact(registration_id)
                .map_err(|error| format!("{error:?}"))?;
            if existing_bytes.as_deref() != Some(schema_bytes) {
                return Err("bundled settings schema conflicts with the registration".to_owned());
            }
        }
        Some(existing) if schema_version_advances(&existing, &binding) => {
            return upgrade_bundled_schema(
                store,
                registration_id,
                &existing,
                &binding,
                schema_bytes,
            );
        }
        Some(_) => {
            return Err("bundled settings schema conflicts with the registration".to_owned());
        }
    }

    let current = store
        .settings_schema_binding(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "bundled settings schema admission is unavailable".to_owned())?;
    if current.desired_revision() == 0
        && current.effective_revision() == 0
        && current.apply_state() == SettingsApplyState::Current
    {
        let schema = decode_settings_schema_v1(schema_bytes).map_err(|_| {
            "module settings schema is invalid or exceeds protocol limits".to_owned()
        })?;
        let values = schema
            .definitions
            .iter()
            .filter_map(|definition| {
                definition
                    .default_value
                    .clone()
                    .map(|value| SettingsValueEntryV1 {
                        setting_id: definition.setting_id.clone(),
                        value: Some(value),
                    })
            })
            .collect::<Vec<_>>();
        let snapshot = SettingsSnapshotV1 {
            target_id: registration_id.to_owned(),
            revision: 1,
            values,
        };
        validate_settings_snapshot_against_schema_v1(&schema, &snapshot)
            .map_err(|_| "initial module settings snapshot is invalid".to_owned())?;
        let complete = settings_snapshot_is_complete_v1(&schema, &snapshot)
            .map_err(|_| "initial module settings snapshot is invalid".to_owned())?;
        store
            .materialize_initial_settings_snapshot(&SettingsInitialSnapshot {
                registration_id: registration_id.to_owned(),
                configuration_instance_id: registration_id.to_owned(),
                created_operation_id: None,
                snapshot_bytes: snapshot.encode_to_vec(),
                complete,
            })
            .map_err(|error| format!("{error:?}"))?;
    }
    Ok(binding)
}

pub(crate) fn materialize_configuration_target<S>(
    store: &S,
    registration_id: &str,
    configuration_instance_id: &str,
    created_operation_id: [u8; 16],
) -> Result<makosh_kernel_control_store::SettingsConfigurationTarget, String>
where
    S: SettingsRegistryStore<Error = StoreError>,
{
    let schema_bytes = store
        .settings_schema_artifact(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "settings schema artifact is unavailable".to_owned())?;
    let schema = decode_settings_schema_v1(&schema_bytes)
        .map_err(|_| "settings schema is invalid or exceeds protocol limits".to_owned())?;
    if schema.definitions.is_empty()
        || schema.definitions.iter().any(|definition| {
            definition.target_scope
                != makosh_runtime_protocol::v1::SettingTargetScopeV1::ConfigurationInstance as i32
        })
    {
        return Err("settings schema does not admit configuration targets".to_owned());
    }
    let values = schema
        .definitions
        .iter()
        .filter_map(|definition| {
            definition
                .default_value
                .clone()
                .map(|value| SettingsValueEntryV1 {
                    setting_id: definition.setting_id.clone(),
                    value: Some(value),
                })
        })
        .collect::<Vec<_>>();
    let snapshot = SettingsSnapshotV1 {
        target_id: configuration_instance_id.to_owned(),
        revision: 1,
        values,
    };
    validate_settings_snapshot_against_schema_v1(&schema, &snapshot)
        .map_err(|_| "initial configuration target snapshot is invalid".to_owned())?;
    let complete = settings_snapshot_is_complete_v1(&schema, &snapshot)
        .map_err(|_| "initial configuration target snapshot is invalid".to_owned())?;
    store
        .materialize_initial_settings_snapshot(&SettingsInitialSnapshot {
            registration_id: registration_id.to_owned(),
            configuration_instance_id: configuration_instance_id.to_owned(),
            created_operation_id: Some(created_operation_id),
            snapshot_bytes: snapshot.encode_to_vec(),
            complete,
        })
        .map_err(|error| format!("{error:?}"))?;
    store
        .settings_configuration_target(registration_id, configuration_instance_id)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "configuration target was not materialized".to_owned())
}

fn upgrade_bundled_schema<S>(
    store: &S,
    registration_id: &str,
    existing: &SettingsSchemaBinding,
    candidate: &SettingsSchemaBinding,
    schema_bytes: &[u8],
) -> Result<SettingsSchemaBinding, String>
where
    S: SettingsRegistryStore<Error = StoreError>,
{
    let existing_schema_bytes = store
        .settings_schema_artifact(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "existing settings schema artifact is unavailable".to_owned())?;
    let existing_sha256: [u8; 32] = Sha256::digest(&existing_schema_bytes).into();
    if existing_sha256 != *existing.schema_sha256() {
        return Err("existing settings schema artifact does not match its binding".to_owned());
    }
    let existing_schema = decode_settings_schema_v1(&existing_schema_bytes)
        .map_err(|_| "existing settings schema artifact is invalid".to_owned())?;
    let candidate_schema = decode_settings_schema_v1(schema_bytes)
        .map_err(|_| "settings schema is invalid or exceeds protocol limits".to_owned())?;
    let targets = store
        .settings_configuration_targets(registration_id)
        .map_err(|error| format!("{error:?}"))?;
    if targets.is_empty() {
        return Err("existing settings configuration targets are unavailable".to_owned());
    }
    let mut target_successors = Vec::with_capacity(targets.len());
    for target in targets {
        let desired = existing_desired_snapshot_for_target(store, registration_id, &target)?;
        validate_settings_snapshot_against_schema_v1(&existing_schema, &desired)
            .map_err(|_| "existing settings snapshot does not match its schema".to_owned())?;
        let next_revision = target
            .desired_revision()
            .checked_add(1)
            .ok_or_else(|| "settings desired revision overflowed".to_owned())?;
        let (successor_snapshot, complete) = project_successor_snapshot(
            target.configuration_instance_id(),
            next_revision,
            &existing_schema,
            &desired,
            &candidate_schema,
        )?;
        target_successors.push(SettingsSchemaTargetSuccessor {
            target: SettingsConfigurationTarget::new(SettingsConfigurationTargetInputV1 {
                registration_id: registration_id.to_owned(),
                configuration_instance_id: target.configuration_instance_id().to_owned(),
                desired_revision: next_revision,
                effective_revision: target.effective_revision(),
                apply_state: if complete {
                    SettingsApplyState::PendingValidation
                } else {
                    SettingsApplyState::BlockedConfig
                },
                sanitized_reason_code: (!complete).then(|| "required_settings_missing".to_owned()),
                created_operation_id: target.created_operation_id().copied(),
            }),
            snapshot_bytes: successor_snapshot.encode_to_vec(),
        });
    }
    let legacy_successor = target_successors
        .iter()
        .find(|target| target.target.configuration_instance_id() == registration_id)
        .ok_or_else(|| "legacy settings configuration target is unavailable".to_owned())?;
    let legacy_target = &legacy_successor.target;
    let successor = SettingsSchemaBinding::new(SettingsSchemaBindingInputV1 {
        registration_id: registration_id.to_owned(),
        schema_major: candidate.schema_major(),
        schema_revision: candidate.schema_revision(),
        schema_sha256: *candidate.schema_sha256(),
        desired_revision: legacy_target.desired_revision(),
        effective_revision: legacy_target.effective_revision(),
        apply_state: legacy_target.apply_state(),
        sanitized_reason_code: legacy_target.sanitized_reason_code().map(str::to_owned),
    });
    store
        .upgrade_settings_schema_with_successor(
            existing,
            &successor,
            schema_bytes,
            &target_successors,
        )
        .map_err(|error| format!("{error:?}"))?;
    Ok(successor)
}

fn existing_desired_snapshot_for_target<S>(
    store: &S,
    registration_id: &str,
    target: &SettingsConfigurationTarget,
) -> Result<SettingsSnapshotV1, String>
where
    S: SettingsRegistryStore<Error = StoreError>,
{
    match store
        .desired_settings_snapshot_for_target(registration_id, target.configuration_instance_id())
        .map_err(|error| format!("{error:?}"))?
    {
        Some((revision, bytes)) if revision == target.desired_revision() => {
            let snapshot = decode_settings_snapshot_v1(&bytes)
                .map_err(|_| "existing settings snapshot is invalid".to_owned())?;
            if snapshot.target_id != target.configuration_instance_id()
                || snapshot.revision != revision
            {
                return Err("existing settings snapshot target or revision is invalid".to_owned());
            }
            Ok(snapshot)
        }
        None if target.desired_revision() == 0 => Ok(SettingsSnapshotV1 {
            target_id: target.configuration_instance_id().to_owned(),
            revision: 0,
            values: Vec::new(),
        }),
        _ => Err("existing settings snapshot revision conflicts with its binding".to_owned()),
    }
}

fn project_successor_snapshot(
    registration_id: &str,
    revision: u64,
    existing_schema: &SettingsSchemaV1,
    existing_snapshot: &SettingsSnapshotV1,
    successor_schema: &SettingsSchemaV1,
) -> Result<(SettingsSnapshotV1, bool), String> {
    let existing_values = existing_snapshot
        .values
        .iter()
        .map(|entry| (entry.setting_id.as_str(), entry))
        .collect::<std::collections::BTreeMap<_, _>>();
    let existing_definitions = existing_schema
        .definitions
        .iter()
        .map(|definition| (definition.setting_id.as_str(), definition))
        .collect::<std::collections::BTreeMap<_, _>>();
    let values = successor_schema
        .definitions
        .iter()
        .filter_map(|definition| {
            let preserved = existing_definitions
                .get(definition.setting_id.as_str())
                .filter(|existing| existing.value_type == definition.value_type)
                .and_then(|_| existing_values.get(definition.setting_id.as_str()))
                .map(|entry| (*entry).clone());
            preserved.or_else(|| {
                definition
                    .default_value
                    .clone()
                    .map(|value| SettingsValueEntryV1 {
                        setting_id: definition.setting_id.clone(),
                        value: Some(value),
                    })
            })
        })
        .collect::<Vec<_>>();
    let snapshot = SettingsSnapshotV1 {
        target_id: registration_id.to_owned(),
        revision,
        values,
    };
    validate_settings_snapshot_against_schema_v1(successor_schema, &snapshot)
        .map_err(|_| "settings schema successor snapshot is invalid".to_owned())?;
    let complete = settings_snapshot_is_complete_v1(successor_schema, &snapshot)
        .map_err(|_| "settings schema successor snapshot is invalid".to_owned())?;
    Ok((snapshot, complete))
}

fn same_schema(existing: &SettingsSchemaBinding, candidate: &SettingsSchemaBinding) -> bool {
    existing.schema_major() == candidate.schema_major()
        && existing.schema_revision() == candidate.schema_revision()
        && existing.schema_sha256() == candidate.schema_sha256()
}

fn schema_version_advances(
    existing: &SettingsSchemaBinding,
    candidate: &SettingsSchemaBinding,
) -> bool {
    candidate.schema_major() > existing.schema_major()
        || (candidate.schema_major() == existing.schema_major()
            && candidate.schema_revision() > existing.schema_revision())
}

fn validated_binding<S>(
    store: &S,
    registration_id: &str,
    descriptor_bytes: &[u8],
    schema_bytes: &[u8],
) -> Result<SettingsSchemaBinding, String>
where
    S: ModuleRegistryStore<Error = StoreError>,
{
    let registration = store
        .module_registration(registration_id)
        .map_err(|error| format!("{error:?}"))?
        .ok_or_else(|| "module registration does not exist".to_owned())?;
    if registration.state() != ModuleRegistrationState::Approved {
        return Err(
            "settings schema admission requires an approved module registration".to_owned(),
        );
    }
    let descriptor = decode_descriptor_v1(descriptor_bytes)
        .map_err(|_| "module descriptor is invalid or exceeds protocol limits".to_owned())?;
    let descriptor_sha256: [u8; 32] = Sha256::digest(descriptor_bytes).into();
    if descriptor_sha256 != *registration.descriptor_sha256() {
        return Err("settings schema descriptor does not match the registration".to_owned());
    }
    let schema_ref = descriptor
        .settings_schema_ref
        .as_ref()
        .ok_or_else(|| "module descriptor does not declare a settings schema".to_owned())?;
    let schema = decode_settings_schema_v1(schema_bytes)
        .map_err(|_| "settings schema is invalid or exceeds protocol limits".to_owned())?;
    let schema_sha256: [u8; 32] = Sha256::digest(schema_bytes).into();
    if schema_ref.major != schema.major
        || schema_ref.revision != schema.revision
        || schema_ref.artifact_size_bytes != schema_bytes.len() as u64
        || schema_ref.sha256.as_slice() != schema_sha256
    {
        return Err("settings schema does not match the descriptor binding".to_owned());
    }
    validate_capability_bindings(&descriptor, &schema)?;
    let binding = SettingsSchemaBinding::new(SettingsSchemaBindingInputV1 {
        registration_id: registration_id.to_owned(),
        schema_major: schema.major,
        schema_revision: schema.revision,
        schema_sha256,
        desired_revision: 0,
        effective_revision: 0,
        apply_state: SettingsApplyState::Current,
        sanitized_reason_code: None,
    });
    Ok(binding)
}

fn validate_capability_bindings(
    descriptor: &makosh_runtime_protocol::v1::ModuleDescriptorV1,
    schema: &makosh_runtime_protocol::v1::SettingsSchemaV1,
) -> Result<(), String> {
    for definition in &schema.definitions {
        if definition.capability_id.is_empty() {
            continue;
        }
        let capability = descriptor
            .capabilities
            .binary_search_by(|item| item.capability_id.cmp(&definition.capability_id))
            .map(|index| &descriptor.capabilities[index])
            .map_err(|_| "settings schema references an unknown capability".to_owned())?;
        if capability
            .settings_definition_ids
            .binary_search(&definition.setting_id)
            .is_err()
        {
            return Err("settings schema definition is absent from its capability".to_owned());
        }
    }
    for capability in &descriptor.capabilities {
        for setting_id in &capability.settings_definition_ids {
            let definition = schema
                .definitions
                .binary_search_by(|item| item.setting_id.cmp(setting_id))
                .map(|index| &schema.definitions[index])
                .map_err(|_| {
                    "module descriptor references an unknown settings definition".to_owned()
                })?;
            if definition.capability_id != capability.capability_id {
                return Err("settings definition is bound to the wrong capability".to_owned());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use makosh_runtime_protocol::v1::{
        SettingApplyModeV1, SettingClientVisibilityV1, SettingDefinitionV1,
        SettingMutationAuthorityV1, SettingTargetScopeV1, SettingValueTypeV1, SettingValueV1,
        setting_value_v1::Value,
    };

    use super::*;

    #[test]
    fn schema_successor_preserves_only_same_id_and_type_values() {
        let existing_schema = schema(2, &["account_id", "bot_email", "realm_url"]);
        let existing_snapshot = SettingsSnapshotV1 {
            target_id: "registration-zulip".to_owned(),
            revision: 1,
            values: vec![
                value("account_id", "account-1"),
                value("bot_email", "bot@example.test"),
                value("realm_url", "https://zulip.example.test"),
            ],
        };
        let successor_schema = schema(3, &["account_email", "account_id", "realm_url"]);

        let (successor, complete) = project_successor_snapshot(
            "registration-zulip",
            2,
            &existing_schema,
            &existing_snapshot,
            &successor_schema,
        )
        .expect("project successor");

        assert!(!complete);
        assert_eq!(successor.revision, 2);
        assert_eq!(
            successor
                .values
                .iter()
                .map(|entry| entry.setting_id.as_str())
                .collect::<Vec<_>>(),
            ["account_id", "realm_url"],
        );
    }

    #[test]
    fn schema_successor_does_not_block_on_missing_optional_definition() {
        let existing_schema = schema(2, &["account_id"]);
        let existing_snapshot = SettingsSnapshotV1 {
            target_id: "registration-mail".to_owned(),
            revision: 7,
            values: vec![value("account_id", "account-1")],
        };
        let mut successor_schema = schema(2, &["account_id", "branch_endpoint"]);
        successor_schema.revision = 2;
        successor_schema.definitions[1].optional = true;

        let (successor, complete) = project_successor_snapshot(
            "registration-mail",
            8,
            &existing_schema,
            &existing_snapshot,
            &successor_schema,
        )
        .expect("project optional successor");

        assert!(complete);
        assert_eq!(successor.values, existing_snapshot.values);
    }

    fn schema(major: u32, setting_ids: &[&str]) -> SettingsSchemaV1 {
        SettingsSchemaV1 {
            major,
            revision: 1,
            definitions: setting_ids.iter().map(|id| definition(id)).collect(),
        }
    }

    fn definition(setting_id: &str) -> SettingDefinitionV1 {
        SettingDefinitionV1 {
            setting_id: setting_id.to_owned(),
            capability_id: String::new(),
            value_type: SettingValueTypeV1::String as i32,
            mutation_authority: SettingMutationAuthorityV1::OperatorManaged as i32,
            target_scope: SettingTargetScopeV1::ConfigurationInstance as i32,
            apply_mode: SettingApplyModeV1::RestartModule as i32,
            client_visibility: SettingClientVisibilityV1::Editable as i32,
            fresh_owner_proof_required: true,
            kernel_controller_id: String::new(),
            display_name: setting_id.to_owned(),
            default_value: None,
            optional: false,
        }
    }

    fn value(setting_id: &str, value: &str) -> SettingsValueEntryV1 {
        SettingsValueEntryV1 {
            setting_id: setting_id.to_owned(),
            value: Some(SettingValueV1 {
                value: Some(Value::StringValue(value.to_owned())),
            }),
        }
    }
}

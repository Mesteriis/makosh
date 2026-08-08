//! Kernel-owned Settings Registry records.

use makosh_kernel_control_store::{
    ModuleRegistrationState, SettingsApplyState, SettingsConfigurationTarget,
    SettingsConfigurationTargetInputV1, SettingsDesiredSnapshot, SettingsInitialSnapshot,
    SettingsSchemaBinding, SettingsSchemaBindingInputV1, SettingsSchemaTargetSuccessor,
};
use rusqlite::{Connection, OptionalExtension, params};

use crate::module_state::registry::read_required_registration;
use crate::{
    SqliteControlStore, StoreError, settings_apply_state_from_str, valid_identity_token,
    valid_sanitized_reason_code, valid_settings_binding_state, valid_settings_configuration_target,
};

const MAX_SETTINGS_BYTES: usize = 256 * 1024;
const MAX_CONFIGURATION_TARGETS: i64 = 32;

impl SqliteControlStore {
    pub fn register_settings_schema(
        &self,
        binding: &SettingsSchemaBinding,
    ) -> Result<(), StoreError> {
        validate_binding(binding)?;
        let binding = binding.clone();
        self.with_connection(move |connection| {
            let transaction = connection.transaction()?;
            require_approved_registration(&transaction, binding.registration_id())?;
            write_schema_binding(&transaction, &binding)?;
            ensure_legacy_configuration_target(&transaction, &binding)?;
            transaction.commit()?;
            Ok(())
        })
    }

    pub fn admit_settings_schema(
        &self,
        binding: &SettingsSchemaBinding,
        schema_bytes: &[u8],
    ) -> Result<(), StoreError> {
        validate_binding(binding)?;
        validate_bounded_bytes(schema_bytes)?;
        let binding = binding.clone();
        let schema_bytes = schema_bytes.to_vec();
        self.with_connection(move |connection| {
            let transaction = connection.transaction()?;
            require_approved_registration(&transaction, binding.registration_id())?;
            write_schema_binding(&transaction, &binding)?;
            ensure_legacy_configuration_target(&transaction, &binding)?;
            transaction.execute(
                "INSERT INTO makosh_kernel_settings_schema_artifact (registration_id, schema_bytes)
                 VALUES (?1, ?2) ON CONFLICT(registration_id)
                 DO UPDATE SET schema_bytes=excluded.schema_bytes",
                params![binding.registration_id(), schema_bytes],
            )?;
            transaction.commit()?;
            Ok(())
        })
    }

    pub fn upgrade_settings_schema_with_successor(
        &self,
        expected: &SettingsSchemaBinding,
        successor: &SettingsSchemaBinding,
        schema_bytes: &[u8],
        target_successors: &[SettingsSchemaTargetSuccessor],
    ) -> Result<(), StoreError> {
        validate_binding(expected)?;
        validate_binding(successor)?;
        validate_bounded_bytes(schema_bytes)?;
        if target_successors.is_empty()
            || i64::try_from(target_successors.len())
                .ok()
                .is_none_or(|count| count > MAX_CONFIGURATION_TARGETS)
        {
            return Err(StoreError::SettingsSchemaRevisionCollision);
        }
        let mut previous_id = "";
        for target_successor in target_successors {
            validate_bounded_bytes(&target_successor.snapshot_bytes)?;
            let target = &target_successor.target;
            if !valid_settings_configuration_target(target)
                || target.registration_id() != successor.registration_id()
                || target.configuration_instance_id() <= previous_id
                || target.desired_revision() == 0
                || target.desired_revision() <= target.effective_revision()
                || !matches!(
                    (target.apply_state(), target.sanitized_reason_code()),
                    (SettingsApplyState::PendingValidation, None)
                        | (
                            SettingsApplyState::BlockedConfig,
                            Some("required_settings_missing")
                        )
                )
            {
                return Err(StoreError::SettingsSchemaRevisionCollision);
            }
            previous_id = target.configuration_instance_id();
        }
        let successor_revision = expected
            .desired_revision()
            .checked_add(1)
            .ok_or(StoreError::RecoveryFenceOverflow)?;
        let version_advances = successor.schema_major() > expected.schema_major()
            || (successor.schema_major() == expected.schema_major()
                && successor.schema_revision() > expected.schema_revision());
        let valid_successor_state = matches!(
            (successor.apply_state(), successor.sanitized_reason_code()),
            (SettingsApplyState::PendingValidation, None)
                | (
                    SettingsApplyState::BlockedConfig,
                    Some("required_settings_missing")
                )
        );
        if expected.registration_id() != successor.registration_id()
            || !version_advances
            || successor.desired_revision() != successor_revision
            || successor.effective_revision() != expected.effective_revision()
            || !valid_successor_state
        {
            return Err(StoreError::SettingsSchemaRevisionCollision);
        }
        let legacy_successor = target_successors
            .iter()
            .find(|target| target.target.configuration_instance_id() == successor.registration_id())
            .ok_or(StoreError::SettingsSchemaRevisionCollision)?;
        if legacy_successor.target.desired_revision() != successor.desired_revision()
            || legacy_successor.target.effective_revision() != successor.effective_revision()
            || legacy_successor.target.apply_state() != successor.apply_state()
            || legacy_successor.target.sanitized_reason_code() != successor.sanitized_reason_code()
        {
            return Err(StoreError::SettingsSchemaRevisionCollision);
        }
        let expected = expected.clone();
        let successor = successor.clone();
        let schema_bytes = schema_bytes.to_vec();
        let target_successors = target_successors.to_vec();
        self.with_connection(move |connection| {
            let transaction = connection.transaction()?;
            require_approved_registration(&transaction, successor.registration_id())?;
            let current = read_settings_binding(&transaction, successor.registration_id())?
                .ok_or(StoreError::SettingsSchemaRevisionCollision)?;
            if current != expected {
                return Err(StoreError::SettingsSchemaRevisionCollision);
            }
            let target_count: i64 = transaction.query_row(
                "SELECT COUNT(*)
                 FROM makosh_kernel_settings_configuration_target
                 WHERE registration_id=?1",
                [successor.registration_id()],
                |row| row.get(0),
            )?;
            if usize::try_from(target_count).ok() != Some(target_successors.len()) {
                return Err(StoreError::SettingsSchemaRevisionCollision);
            }
            write_schema_binding(&transaction, &successor)?;
            transaction.execute(
                "INSERT INTO makosh_kernel_settings_schema_artifact
                 (registration_id, schema_bytes) VALUES (?1, ?2)
                 ON CONFLICT(registration_id) DO UPDATE SET schema_bytes=excluded.schema_bytes",
                params![successor.registration_id(), schema_bytes],
            )?;
            for target_successor in &target_successors {
                let target = &target_successor.target;
                let current_target = read_configuration_target(
                    &transaction,
                    successor.registration_id(),
                    target.configuration_instance_id(),
                )?
                .ok_or(StoreError::SettingsSchemaRevisionCollision)?;
                if current_target
                    .desired_revision()
                    .checked_add(1)
                    .ok_or(StoreError::RecoveryFenceOverflow)?
                    != target.desired_revision()
                    || current_target.effective_revision() != target.effective_revision()
                    || current_target.created_operation_id() != target.created_operation_id()
                {
                    return Err(StoreError::SettingsRevisionConflict);
                }
                let changed = transaction.execute(
                    "UPDATE makosh_kernel_settings_configuration_target
                     SET desired_revision=?3, apply_state=?4, sanitized_reason_code=?5
                     WHERE registration_id=?1 AND configuration_instance_id=?2
                       AND desired_revision=?6 AND effective_revision=?7",
                    params![
                        successor.registration_id(),
                        target.configuration_instance_id(),
                        as_sql(target.desired_revision())?,
                        target.apply_state().as_str(),
                        target.sanitized_reason_code(),
                        as_sql(current_target.desired_revision())?,
                        as_sql(current_target.effective_revision())?,
                    ],
                )?;
                if changed != 1 {
                    return Err(StoreError::SettingsRevisionConflict);
                }
                transaction.execute(
                    "INSERT INTO makosh_kernel_settings_desired_snapshot
                     (registration_id, configuration_instance_id, revision, snapshot_bytes)
                     VALUES (?1, ?2, ?3, ?4)
                     ON CONFLICT(registration_id, configuration_instance_id) DO UPDATE SET
                     revision=excluded.revision, snapshot_bytes=excluded.snapshot_bytes",
                    params![
                        successor.registration_id(),
                        target.configuration_instance_id(),
                        as_sql(target.desired_revision())?,
                        &target_successor.snapshot_bytes,
                    ],
                )?;
            }
            transaction.commit()?;
            Ok(())
        })
    }

    pub fn settings_schema_artifact(
        &self,
        registration_id: &str,
    ) -> Result<Option<Vec<u8>>, StoreError> {
        let registration_id = registration_id.to_owned();
        self.with_connection(move |connection| {
            connection
                .query_row(
                    "SELECT schema_bytes FROM makosh_kernel_settings_schema_artifact
                     WHERE registration_id=?1",
                    [&registration_id],
                    |row| row.get(0),
                )
                .optional()
                .map_err(StoreError::from)
        })
    }

    pub fn settings_schema_binding(
        &self,
        registration_id: &str,
    ) -> Result<Option<SettingsSchemaBinding>, StoreError> {
        let registration_id = registration_id.to_owned();
        self.with_connection(move |connection| read_settings_binding(connection, &registration_id))
    }

    pub fn settings_configuration_target(
        &self,
        registration_id: &str,
        configuration_instance_id: &str,
    ) -> Result<Option<SettingsConfigurationTarget>, StoreError> {
        let registration_id = registration_id.to_owned();
        let configuration_instance_id = configuration_instance_id.to_owned();
        self.with_connection(move |connection| {
            read_configuration_target(connection, &registration_id, &configuration_instance_id)
        })
    }

    pub fn settings_configuration_targets(
        &self,
        registration_id: &str,
    ) -> Result<Vec<SettingsConfigurationTarget>, StoreError> {
        let registration_id = registration_id.to_owned();
        self.with_connection(move |connection| {
            let mut statement = connection.prepare(
                "SELECT configuration_instance_id,
                        desired_revision,
                        effective_revision,
                        apply_state,
                        sanitized_reason_code,
                        created_operation_id
                 FROM makosh_kernel_settings_configuration_target
                 WHERE registration_id=?1
                 ORDER BY configuration_instance_id",
            )?;
            let rows = statement.query_map([&registration_id], |row| {
                decode_configuration_target(row, &registration_id)
            })?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(StoreError::from)
        })
    }

    pub fn commit_desired_settings_snapshot(
        &self,
        update: &SettingsDesiredSnapshot,
    ) -> Result<u64, StoreError> {
        validate_bounded_bytes(&update.snapshot_bytes)?;
        validate_configuration_target_identity(
            &update.registration_id,
            &update.configuration_instance_id,
        )?;
        let update = update.clone();
        self.with_connection(move |connection| {
            let next = update
                .expected_revision
                .checked_add(1)
                .ok_or(StoreError::RecoveryFenceOverflow)?;
            let transaction = connection.transaction()?;
            let changed = transaction.execute(
                "UPDATE makosh_kernel_settings_configuration_target
                 SET desired_revision=?1, apply_state='pending_validation', sanitized_reason_code=NULL
                 WHERE registration_id=?2 AND configuration_instance_id=?3
                   AND desired_revision=?4",
                params![
                    as_sql(next)?,
                    update.registration_id,
                    update.configuration_instance_id,
                    as_sql(update.expected_revision)?
                ],
            )?;
            if changed != 1 {
                return Err(StoreError::SettingsRevisionConflict);
            }
            transaction.execute(
                "INSERT INTO makosh_kernel_settings_desired_snapshot
                 (registration_id, configuration_instance_id, revision, snapshot_bytes)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(registration_id, configuration_instance_id) DO UPDATE SET
                 revision=excluded.revision, snapshot_bytes=excluded.snapshot_bytes",
                params![
                    update.registration_id,
                    update.configuration_instance_id,
                    as_sql(next)?,
                    update.snapshot_bytes
                ],
            )?;
            mirror_legacy_target_state(
                &transaction,
                &update.registration_id,
                &update.configuration_instance_id,
            )?;
            transaction.commit()?;
            Ok(next)
        })
    }

    pub fn materialize_initial_settings_snapshot(
        &self,
        update: &SettingsInitialSnapshot,
    ) -> Result<u64, StoreError> {
        validate_bounded_bytes(&update.snapshot_bytes)?;
        validate_configuration_target_identity(
            &update.registration_id,
            &update.configuration_instance_id,
        )?;
        if update
            .created_operation_id
            .is_some_and(|operation_id| operation_id.iter().all(|byte| *byte == 0))
        {
            return Err(StoreError::SettingsRevisionConflict);
        }
        let update = update.clone();
        self.with_connection(move |connection| {
            let transaction = connection.transaction()?;
            require_settings_schema(&transaction, &update.registration_id)?;
            let existing_by_operation = update
                .created_operation_id
                .map(|operation_id| {
                    read_configuration_target_by_operation(
                        &transaction,
                        &update.registration_id,
                        &operation_id,
                    )
                })
                .transpose()?
                .flatten();
            if existing_by_operation.as_ref().is_some_and(|target| {
                target.configuration_instance_id() != update.configuration_instance_id
            }) {
                return Err(StoreError::SettingsRevisionConflict);
            }
            if let Some(target) = existing_by_operation {
                let desired_revision = target.desired_revision();
                transaction.commit()?;
                return Ok(desired_revision);
            }
            let new_configuration_target = update.created_operation_id.is_some();
            let expected_effective = u64::from(update.complete && !new_configuration_target);
            let expected_state = if !update.complete {
                SettingsApplyState::BlockedConfig
            } else if new_configuration_target {
                SettingsApplyState::PendingValidation
            } else {
                SettingsApplyState::Current
            };
            let existing = read_configuration_target(
                &transaction,
                &update.registration_id,
                &update.configuration_instance_id,
            )?;
            if existing.as_ref().is_some_and(|target| {
                target.desired_revision() == 1
                    && target.effective_revision() == expected_effective
                    && target.apply_state() == expected_state
                    && target.created_operation_id() == update.created_operation_id.as_ref()
            }) {
                let existing = transaction
                    .query_row(
                        "SELECT snapshot_bytes
                         FROM makosh_kernel_settings_desired_snapshot
                         WHERE registration_id=?1 AND configuration_instance_id=?2
                           AND revision=1",
                        params![update.registration_id, update.configuration_instance_id],
                        |row| row.get::<_, Vec<u8>>(0),
                    )
                    .optional()?;
                if existing.as_deref() == Some(update.snapshot_bytes.as_slice()) {
                    transaction.commit()?;
                    return Ok(1);
                }
                return Err(StoreError::SettingsRevisionConflict);
            }
            match existing {
                Some(target)
                    if target.desired_revision() == 0
                        && target.effective_revision() == 0
                        && target.apply_state() == SettingsApplyState::Current
                        && target.created_operation_id()
                            == update.created_operation_id.as_ref() => {}
                Some(_) => return Err(StoreError::SettingsRevisionConflict),
                None => {
                    let count: i64 = transaction.query_row(
                        "SELECT COUNT(*)
                         FROM makosh_kernel_settings_configuration_target
                         WHERE registration_id=?1",
                        [&update.registration_id],
                        |row| row.get(0),
                    )?;
                    if count >= MAX_CONFIGURATION_TARGETS {
                        return Err(StoreError::SettingsRevisionConflict);
                    }
                    transaction.execute(
                        "INSERT INTO makosh_kernel_settings_configuration_target (
                            registration_id,
                            configuration_instance_id,
                            desired_revision,
                            effective_revision,
                            apply_state,
                            sanitized_reason_code,
                            created_operation_id
                         ) VALUES (?1, ?2, 0, 0, 'current', NULL, ?3)",
                        params![
                            update.registration_id,
                            update.configuration_instance_id,
                            update.created_operation_id.map(|value| value.to_vec())
                        ],
                    )?;
                }
            }
            let inserted = transaction.execute(
                "INSERT INTO makosh_kernel_settings_desired_snapshot
                 (registration_id, configuration_instance_id, revision, snapshot_bytes)
                 VALUES (?1, ?2, 1, ?3)",
                params![
                    update.registration_id,
                    update.configuration_instance_id,
                    update.snapshot_bytes
                ],
            )?;
            let changed = transaction.execute(
                "UPDATE makosh_kernel_settings_configuration_target
                 SET desired_revision=1, effective_revision=?2,
                     apply_state=?3, sanitized_reason_code=?4
                 WHERE registration_id=?1 AND configuration_instance_id=?5
                   AND desired_revision=0
                   AND effective_revision=0 AND apply_state='current'",
                params![
                    update.registration_id,
                    as_sql(expected_effective)?,
                    expected_state.as_str(),
                    (!update.complete).then_some("required_settings_missing"),
                    update.configuration_instance_id,
                ],
            )?;
            if inserted != 1 || changed != 1 {
                return Err(StoreError::SettingsRevisionConflict);
            }
            mirror_legacy_target_state(
                &transaction,
                &update.registration_id,
                &update.configuration_instance_id,
            )?;
            transaction.commit()?;
            Ok(1)
        })
    }

    pub fn desired_settings_snapshot(
        &self,
        registration_id: &str,
    ) -> Result<Option<(u64, Vec<u8>)>, StoreError> {
        self.desired_settings_snapshot_for_target(registration_id, registration_id)
    }

    pub fn desired_settings_snapshot_for_target(
        &self,
        registration_id: &str,
        configuration_instance_id: &str,
    ) -> Result<Option<(u64, Vec<u8>)>, StoreError> {
        let registration_id = registration_id.to_owned();
        let configuration_instance_id = configuration_instance_id.to_owned();
        self.with_connection(move |connection| {
            connection
                .query_row(
                    "SELECT revision, snapshot_bytes FROM makosh_kernel_settings_desired_snapshot
                 WHERE registration_id=?1 AND configuration_instance_id=?2",
                    params![registration_id, configuration_instance_id],
                    |row| Ok((as_u64(row.get(0)?, 0)?, row.get(1)?)),
                )
                .optional()
                .map_err(StoreError::from)
        })
    }

    pub fn transition_settings_apply_state(
        &self,
        registration_id: &str,
        revision: u64,
        next: SettingsApplyState,
        sanitized_reason_code: Option<&str>,
    ) -> Result<(), StoreError> {
        self.transition_settings_apply_state_for_target(
            registration_id,
            registration_id,
            revision,
            next,
            sanitized_reason_code,
        )
    }

    pub fn transition_settings_apply_state_for_target(
        &self,
        registration_id: &str,
        configuration_instance_id: &str,
        revision: u64,
        next: SettingsApplyState,
        sanitized_reason_code: Option<&str>,
    ) -> Result<(), StoreError> {
        validate_apply_transition(
            registration_id,
            configuration_instance_id,
            next,
            sanitized_reason_code,
        )?;
        let registration_id = registration_id.to_owned();
        let configuration_instance_id = configuration_instance_id.to_owned();
        let reason = sanitized_reason_code.map(str::to_owned);
        self.with_connection(move |connection| {
            transition_apply_state(
                connection,
                &registration_id,
                &configuration_instance_id,
                revision,
                next,
                reason.as_deref(),
            )
        })
    }

    pub fn confirm_effective_settings_revision(
        &self,
        registration_id: &str,
        revision: u64,
    ) -> Result<(), StoreError> {
        self.confirm_effective_settings_revision_for_target(
            registration_id,
            registration_id,
            revision,
        )
    }

    pub fn confirm_effective_settings_revision_for_target(
        &self,
        registration_id: &str,
        configuration_instance_id: &str,
        revision: u64,
    ) -> Result<(), StoreError> {
        let registration_id = registration_id.to_owned();
        let configuration_instance_id = configuration_instance_id.to_owned();
        self.with_connection(move |connection| {
            let transaction = connection.transaction()?;
            let changed = transaction.execute(
                "UPDATE makosh_kernel_settings_configuration_target
                 SET effective_revision=?1, apply_state='current', sanitized_reason_code=NULL
                 WHERE registration_id=?2 AND configuration_instance_id=?3
                   AND desired_revision=?1 AND effective_revision < ?1
                 AND apply_state IN ('applying', 'awaiting_external_restart')",
                params![
                    as_sql(revision)?,
                    registration_id,
                    configuration_instance_id
                ],
            )?;
            if changed == 1 {
                mirror_legacy_target_state(
                    &transaction,
                    &registration_id,
                    &configuration_instance_id,
                )?;
                transaction.commit()?;
                Ok(())
            } else {
                Err(StoreError::SettingsRevisionConflict)
            }
        })
    }
}

fn validate_binding(binding: &SettingsSchemaBinding) -> Result<(), StoreError> {
    let valid = valid_identity_token(binding.registration_id())
        && binding.schema_major() > 0
        && binding.schema_revision() > 0
        && valid_settings_binding_state(binding);
    valid
        .then_some(())
        .ok_or(StoreError::InvalidSettingsSchemaBinding)
}

fn validate_bounded_bytes(bytes: &[u8]) -> Result<(), StoreError> {
    (!bytes.is_empty() && bytes.len() <= MAX_SETTINGS_BYTES)
        .then_some(())
        .ok_or(StoreError::InvalidSettingsSchemaBinding)
}

fn require_approved_registration(
    connection: &Connection,
    registration_id: &str,
) -> Result<(), StoreError> {
    let registration = read_required_registration(connection, registration_id)?;
    if registration.state() == ModuleRegistrationState::Approved {
        Ok(())
    } else {
        Err(StoreError::InvalidSettingsSchemaBinding)
    }
}

fn write_schema_binding(
    connection: &Connection,
    binding: &SettingsSchemaBinding,
) -> Result<(), StoreError> {
    let changed = connection.execute(
        "INSERT INTO makosh_kernel_settings_schema_binding
         (registration_id, schema_major, schema_revision, schema_sha256,
          desired_revision, effective_revision, apply_state, sanitized_reason_code)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(registration_id) DO UPDATE SET schema_major=excluded.schema_major,
         schema_revision=excluded.schema_revision, schema_sha256=excluded.schema_sha256,
         desired_revision=excluded.desired_revision, effective_revision=excluded.effective_revision,
         apply_state=excluded.apply_state, sanitized_reason_code=excluded.sanitized_reason_code
         WHERE excluded.schema_major > makosh_kernel_settings_schema_binding.schema_major
         OR (excluded.schema_major = makosh_kernel_settings_schema_binding.schema_major
             AND excluded.schema_revision > makosh_kernel_settings_schema_binding.schema_revision)",
        params![
            binding.registration_id(),
            binding.schema_major(),
            binding.schema_revision(),
            binding.schema_sha256().as_slice(),
            as_sql(binding.desired_revision())?,
            as_sql(binding.effective_revision())?,
            binding.apply_state().as_str(),
            binding.sanitized_reason_code()
        ],
    )?;
    if changed == 1 {
        Ok(())
    } else {
        Err(StoreError::SettingsSchemaRevisionCollision)
    }
}

fn ensure_legacy_configuration_target(
    connection: &Connection,
    binding: &SettingsSchemaBinding,
) -> Result<(), StoreError> {
    connection.execute(
        "INSERT INTO makosh_kernel_settings_configuration_target (
            registration_id,
            configuration_instance_id,
            desired_revision,
            effective_revision,
            apply_state,
            sanitized_reason_code,
            created_operation_id
         ) VALUES (?1, ?1, ?2, ?3, ?4, ?5, NULL)
         ON CONFLICT(registration_id, configuration_instance_id) DO NOTHING",
        params![
            binding.registration_id(),
            as_sql(binding.desired_revision())?,
            as_sql(binding.effective_revision())?,
            binding.apply_state().as_str(),
            binding.sanitized_reason_code()
        ],
    )?;
    Ok(())
}

fn read_settings_binding(
    connection: &Connection,
    registration_id: &str,
) -> Result<Option<SettingsSchemaBinding>, StoreError> {
    connection
        .query_row(
            "SELECT schema_major, schema_revision, schema_sha256, desired_revision,
         effective_revision, apply_state, sanitized_reason_code
         FROM makosh_kernel_settings_schema_binding WHERE registration_id=?1",
            [registration_id],
            |row| {
                let digest: Vec<u8> = row.get(2)?;
                let state = settings_apply_state_from_str(&row.get::<_, String>(5)?)
                    .ok_or(rusqlite::Error::InvalidQuery)?;
                Ok(SettingsSchemaBinding::new(SettingsSchemaBindingInputV1 {
                    registration_id: registration_id.to_owned(),
                    schema_major: row.get(0)?,
                    schema_revision: row.get(1)?,
                    schema_sha256: digest
                        .try_into()
                        .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(2, 32))?,
                    desired_revision: as_u64(row.get(3)?, 3)?,
                    effective_revision: as_u64(row.get(4)?, 4)?,
                    apply_state: state,
                    sanitized_reason_code: row.get(6)?,
                }))
            },
        )
        .optional()
        .map_err(StoreError::from)
}

fn validate_apply_transition(
    registration_id: &str,
    configuration_instance_id: &str,
    next: SettingsApplyState,
    reason: Option<&str>,
) -> Result<(), StoreError> {
    let valid = valid_identity_token(registration_id)
        && valid_identity_token(configuration_instance_id)
        && valid_sanitized_reason_code(reason)
        && next != SettingsApplyState::Current
        && (next == SettingsApplyState::BlockedConfig) == reason.is_some();
    valid
        .then_some(())
        .ok_or(StoreError::InvalidSettingsApplyState)
}

fn transition_apply_state(
    connection: &mut Connection,
    registration_id: &str,
    configuration_instance_id: &str,
    revision: u64,
    next: SettingsApplyState,
    reason: Option<&str>,
) -> Result<(), StoreError> {
    let transaction = connection.transaction()?;
    let (desired, effective, current) =
        read_apply_state(&transaction, registration_id, configuration_instance_id)?;
    if desired != revision || effective >= revision || !current.can_transition_to(next) {
        return Err(StoreError::SettingsRevisionConflict);
    }
    let changed = transaction.execute(
        "UPDATE makosh_kernel_settings_configuration_target
         SET apply_state=?1, sanitized_reason_code=?2
         WHERE registration_id=?3 AND configuration_instance_id=?4
           AND desired_revision=?5 AND apply_state=?6",
        params![
            next.as_str(),
            reason,
            registration_id,
            configuration_instance_id,
            as_sql(revision)?,
            current.as_str()
        ],
    )?;
    if changed != 1 {
        return Err(StoreError::SettingsRevisionConflict);
    }
    mirror_legacy_target_state(&transaction, registration_id, configuration_instance_id)?;
    transaction.commit()?;
    Ok(())
}

fn read_apply_state(
    connection: &Connection,
    registration_id: &str,
    configuration_instance_id: &str,
) -> Result<(u64, u64, SettingsApplyState), StoreError> {
    connection
        .query_row(
            "SELECT desired_revision, effective_revision, apply_state
         FROM makosh_kernel_settings_configuration_target
         WHERE registration_id=?1 AND configuration_instance_id=?2",
            params![registration_id, configuration_instance_id],
            |row| {
                let state = settings_apply_state_from_str(&row.get::<_, String>(2)?)
                    .ok_or(rusqlite::Error::InvalidQuery)?;
                Ok((as_u64(row.get(0)?, 0)?, as_u64(row.get(1)?, 1)?, state))
            },
        )
        .optional()?
        .ok_or(StoreError::SettingsRevisionConflict)
}

fn read_configuration_target(
    connection: &Connection,
    registration_id: &str,
    configuration_instance_id: &str,
) -> Result<Option<SettingsConfigurationTarget>, StoreError> {
    connection
        .query_row(
            "SELECT configuration_instance_id,
                    desired_revision,
                    effective_revision,
                    apply_state,
                    sanitized_reason_code,
                    created_operation_id
             FROM makosh_kernel_settings_configuration_target
             WHERE registration_id=?1 AND configuration_instance_id=?2",
            params![registration_id, configuration_instance_id],
            |row| decode_configuration_target(row, registration_id),
        )
        .optional()
        .map_err(StoreError::from)
}

fn read_configuration_target_by_operation(
    connection: &Connection,
    registration_id: &str,
    operation_id: &[u8; 16],
) -> Result<Option<SettingsConfigurationTarget>, StoreError> {
    connection
        .query_row(
            "SELECT configuration_instance_id,
                    desired_revision,
                    effective_revision,
                    apply_state,
                    sanitized_reason_code,
                    created_operation_id
             FROM makosh_kernel_settings_configuration_target
             WHERE registration_id=?1 AND created_operation_id=?2",
            params![registration_id, operation_id.as_slice()],
            |row| decode_configuration_target(row, registration_id),
        )
        .optional()
        .map_err(StoreError::from)
}

fn decode_configuration_target(
    row: &rusqlite::Row<'_>,
    registration_id: &str,
) -> Result<SettingsConfigurationTarget, rusqlite::Error> {
    let state = settings_apply_state_from_str(&row.get::<_, String>(3)?)
        .ok_or(rusqlite::Error::InvalidQuery)?;
    let operation_id = row
        .get::<_, Option<Vec<u8>>>(5)?
        .map(|bytes| {
            bytes
                .try_into()
                .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(5, 16))
        })
        .transpose()?;
    let target = SettingsConfigurationTarget::new(SettingsConfigurationTargetInputV1 {
        registration_id: registration_id.to_owned(),
        configuration_instance_id: row.get(0)?,
        desired_revision: as_u64(row.get(1)?, 1)?,
        effective_revision: as_u64(row.get(2)?, 2)?,
        apply_state: state,
        sanitized_reason_code: row.get(4)?,
        created_operation_id: operation_id,
    });
    if valid_settings_configuration_target(&target) {
        Ok(target)
    } else {
        Err(rusqlite::Error::InvalidQuery)
    }
}

fn validate_configuration_target_identity(
    registration_id: &str,
    configuration_instance_id: &str,
) -> Result<(), StoreError> {
    (valid_identity_token(registration_id) && valid_identity_token(configuration_instance_id))
        .then_some(())
        .ok_or(StoreError::SettingsRevisionConflict)
}

fn require_settings_schema(
    connection: &Connection,
    registration_id: &str,
) -> Result<(), StoreError> {
    let exists: bool = connection.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM makosh_kernel_settings_schema_binding
            WHERE registration_id=?1
         )",
        [registration_id],
        |row| row.get(0),
    )?;
    exists
        .then_some(())
        .ok_or(StoreError::SettingsRevisionConflict)
}

fn mirror_legacy_target_state(
    connection: &Connection,
    registration_id: &str,
    configuration_instance_id: &str,
) -> Result<(), StoreError> {
    if registration_id != configuration_instance_id {
        return Ok(());
    }
    let changed = connection.execute(
        "UPDATE makosh_kernel_settings_schema_binding
         SET desired_revision=target.desired_revision,
             effective_revision=target.effective_revision,
             apply_state=target.apply_state,
             sanitized_reason_code=target.sanitized_reason_code
         FROM makosh_kernel_settings_configuration_target AS target
         WHERE makosh_kernel_settings_schema_binding.registration_id=?1
           AND target.registration_id=?1
           AND target.configuration_instance_id=?1",
        [registration_id],
    )?;
    if changed == 1 {
        Ok(())
    } else {
        Err(StoreError::SettingsRevisionConflict)
    }
}

fn as_sql(value: u64) -> Result<i64, StoreError> {
    i64::try_from(value).map_err(|_| StoreError::RecoveryFenceOverflow)
}

fn as_u64(value: i64, index: usize) -> Result<u64, rusqlite::Error> {
    u64::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, 0))
}

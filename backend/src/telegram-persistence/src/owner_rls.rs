//! Immutable owner-scope and RLS migration for the complete Telegram store.

use std::fmt::Write as _;

use makosh_storage_protocol::v1::StorageMigrationStepV1;
use sha2::{Digest, Sha256};

pub const TELEGRAM_OWNER_RLS_STORAGE_REVISION_V1: u32 = 10;

/// Every Telegram-owned table present after storage revision 9, plus the
/// revision-10 owner-scope table. Keep this list sorted and exact: the generated
/// migration is the signed storage artifact.
pub const TELEGRAM_OWNER_RLS_TABLES_V1: [&str; 46] = [
    "telegram_accounts",
    "telegram_attachment_projections",
    "telegram_automation_mutation_receipts",
    "telegram_automation_policies",
    "telegram_automation_policy_chat_scopes",
    "telegram_automation_preview_receipts",
    "telegram_automation_template_variables",
    "telegram_automation_templates",
    "telegram_call_evidence_outbox",
    "telegram_call_local_mute",
    "telegram_call_media_projection",
    "telegram_call_media_state_history",
    "telegram_call_operation_history",
    "telegram_call_operations",
    "telegram_call_realtime_backfill_jobs",
    "telegram_call_realtime_events",
    "telegram_call_realtime_frames",
    "telegram_call_realtime_replay_cursor",
    "telegram_call_realtime_replay_order",
    "telegram_call_sessions",
    "telegram_call_state_history",
    "telegram_chat_avatar_projections",
    "telegram_chat_folder_projections",
    "telegram_chat_operational_states",
    "telegram_chat_position_projections",
    "telegram_chat_projections",
    "telegram_chat_states",
    "telegram_communications_outbox",
    "telegram_delivery_intent_inbox",
    "telegram_delivery_intent_jobs",
    "telegram_delivery_intent_result_outbox",
    "telegram_delivery_route_accounts",
    "telegram_delivery_route_conversations",
    "telegram_delivery_route_messages",
    "telegram_file_projections",
    "telegram_message_mutations",
    "telegram_message_projections",
    "telegram_message_reactions",
    "telegram_message_tombstones",
    "telegram_message_versions",
    "telegram_owner_scope",
    "telegram_participant_projections",
    "telegram_provider_event_journal",
    "telegram_runtime_operations",
    "telegram_runtime_reconfigurations",
    "telegram_topic_projections",
];

const OWNER_SCOPE_TABLE: &str = "telegram_owner_scope";
const CURRENT_PRINCIPAL_PREFIX: &str = "regexp_replace(current_user::text, '_[0-9]+$', '')";
const CURRENT_RUNTIME_PRINCIPAL: &str = "current_user::text ~ '^storage_[a-f0-9]{16}_[1-9][0-9]*$'";

#[must_use]
pub fn telegram_owner_rls_sql_v1() -> String {
    let mut sql = String::from(
        "CREATE TABLE IF NOT EXISTS makosh_data.telegram_owner_scope (\n\
             singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),\n\
             logical_owner_id TEXT NOT NULL UNIQUE CHECK (\n\
                 length(logical_owner_id) BETWEEN 1 AND 128\n\
                 AND logical_owner_id = lower(logical_owner_id)\n\
                 AND logical_owner_id ~ '^[a-z0-9][a-z0-9_-]*$'\n\
             ),\n\
             runtime_principal_prefix TEXT NOT NULL UNIQUE CHECK (\n\
                 runtime_principal_prefix ~ '^storage_[a-f0-9]{16}$'\n\
             )\n\
         );\n",
    );

    for table in TELEGRAM_OWNER_RLS_TABLES_V1 {
        writeln!(
            sql,
            "ALTER TABLE makosh_data.{table} ENABLE ROW LEVEL SECURITY;\n\
             ALTER TABLE makosh_data.{table} FORCE ROW LEVEL SECURITY;"
        )
        .expect("writing owner RLS SQL to String cannot fail");
        if table == OWNER_SCOPE_TABLE {
            writeln!(
                sql,
                "CREATE POLICY {table}_owner_isolation_v1\n\
                 ON makosh_data.{table}\n\
                 USING (\n\
                     {CURRENT_RUNTIME_PRINCIPAL}\n\
                     AND runtime_principal_prefix = {CURRENT_PRINCIPAL_PREFIX}\n\
                 )\n\
                 WITH CHECK (\n\
                     {CURRENT_RUNTIME_PRINCIPAL}\n\
                     AND runtime_principal_prefix = {CURRENT_PRINCIPAL_PREFIX}\n\
                 );"
            )
            .expect("writing owner-scope policy to String cannot fail");
        } else {
            writeln!(
                sql,
                "CREATE POLICY {table}_owner_isolation_v1\n\
                 ON makosh_data.{table}\n\
                 USING (EXISTS (\n\
                     SELECT 1 FROM makosh_data.telegram_owner_scope AS scope\n\
                     WHERE {CURRENT_RUNTIME_PRINCIPAL}\n\
                       AND scope.runtime_principal_prefix = {CURRENT_PRINCIPAL_PREFIX}\n\
                 ))\n\
                 WITH CHECK (EXISTS (\n\
                     SELECT 1 FROM makosh_data.telegram_owner_scope AS scope\n\
                     WHERE {CURRENT_RUNTIME_PRINCIPAL}\n\
                       AND scope.runtime_principal_prefix = {CURRENT_PRINCIPAL_PREFIX}\n\
                 ));"
            )
            .expect("writing owner policy to String cannot fail");
        }
    }
    sql
}

#[must_use]
pub fn telegram_owner_rls_storage_migration_v1() -> StorageMigrationStepV1 {
    let sql = telegram_owner_rls_sql_v1();
    StorageMigrationStepV1 {
        revision: TELEGRAM_OWNER_RLS_STORAGE_REVISION_V1,
        migration_id: "telegram_owner_scope_and_rls".to_owned(),
        sha256: Sha256::digest(sql.as_bytes()).to_vec(),
        forward_sql_utf8: sql.into_bytes(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn migration_is_revisioned_and_force_isolates_the_exact_table_inventory() {
        let migration = telegram_owner_rls_storage_migration_v1();
        let sql = std::str::from_utf8(&migration.forward_sql_utf8).expect("owner RLS SQL");

        assert_eq!(migration.revision, 10);
        assert_eq!(migration.migration_id, "telegram_owner_scope_and_rls");
        assert_eq!(migration.sha256, Sha256::digest(sql.as_bytes()).to_vec());
        assert_eq!(
            TELEGRAM_OWNER_RLS_TABLES_V1
                .into_iter()
                .collect::<BTreeSet<_>>()
                .len(),
            TELEGRAM_OWNER_RLS_TABLES_V1.len(),
        );
        assert_eq!(sql.matches("ENABLE ROW LEVEL SECURITY").count(), 46);
        assert_eq!(sql.matches("FORCE ROW LEVEL SECURITY").count(), 46);
        assert_eq!(sql.matches("CREATE POLICY").count(), 46);
        assert_eq!(sql.matches(CURRENT_RUNTIME_PRINCIPAL).count(), 92);
        assert_eq!(sql.matches(CURRENT_PRINCIPAL_PREFIX).count(), 92);
        for table in TELEGRAM_OWNER_RLS_TABLES_V1 {
            assert!(
                sql.contains(&format!(
                    "ALTER TABLE makosh_data.{table} ENABLE ROW LEVEL SECURITY"
                )),
                "missing RLS for {table}"
            );
            assert!(
                sql.contains(&format!(
                    "ALTER TABLE makosh_data.{table} FORCE ROW LEVEL SECURITY"
                )),
                "missing FORCE RLS for {table}"
            );
        }
    }
}

//! Immutable owner-scope and RLS migration for the complete Zulip store.

use std::fmt::Write as _;

use makosh_storage_protocol::v1::StorageMigrationStepV1;
use sha2::{Digest, Sha256};

pub const ZULIP_OWNER_RLS_STORAGE_REVISION_V1: u32 = 7;

/// Every Zulip-owned table present after storage revision 6, plus the
/// revision-7 owner-scope table. This sorted inventory is part of the signed
/// storage artifact.
pub const ZULIP_OWNER_RLS_TABLES_V1: [&str; 19] = [
    "zulip_account_credential_bindings",
    "zulip_command_operations",
    "zulip_command_queue",
    "zulip_communications_outbox",
    "zulip_delivery_intent_inbox",
    "zulip_delivery_intent_jobs",
    "zulip_delivery_intent_result_outbox",
    "zulip_delivery_route_accounts",
    "zulip_delivery_route_conversations",
    "zulip_delivery_route_messages",
    "zulip_operational_account_state",
    "zulip_operational_attachments",
    "zulip_operational_conversations",
    "zulip_operational_events",
    "zulip_operational_message_mutations",
    "zulip_operational_messages",
    "zulip_operational_reactions",
    "zulip_owner_scope",
    "zulip_provider_cursor",
];

const OWNER_SCOPE_TABLE: &str = "zulip_owner_scope";
const CURRENT_PRINCIPAL_PREFIX: &str = "regexp_replace(current_user::text, '_[0-9]+$', '')";
const CURRENT_RUNTIME_PRINCIPAL: &str = "current_user::text ~ '^storage_[a-f0-9]{16}_[1-9][0-9]*$'";

#[must_use]
pub fn zulip_owner_rls_sql_v1() -> String {
    let mut sql = String::from(
        "CREATE TABLE IF NOT EXISTS makosh_data.zulip_owner_scope (\n\
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

    for table in ZULIP_OWNER_RLS_TABLES_V1 {
        writeln!(
            sql,
            "ALTER TABLE makosh_data.{table} ENABLE ROW LEVEL SECURITY;\n\
             ALTER TABLE makosh_data.{table} FORCE ROW LEVEL SECURITY;"
        )
        .expect("writing Zulip owner RLS SQL cannot fail");
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
            .expect("writing Zulip owner-scope policy cannot fail");
        } else {
            writeln!(
                sql,
                "CREATE POLICY {table}_owner_isolation_v1\n\
                 ON makosh_data.{table}\n\
                 USING (EXISTS (\n\
                     SELECT 1 FROM makosh_data.zulip_owner_scope AS scope\n\
                     WHERE {CURRENT_RUNTIME_PRINCIPAL}\n\
                       AND scope.runtime_principal_prefix = {CURRENT_PRINCIPAL_PREFIX}\n\
                 ))\n\
                 WITH CHECK (EXISTS (\n\
                     SELECT 1 FROM makosh_data.zulip_owner_scope AS scope\n\
                     WHERE {CURRENT_RUNTIME_PRINCIPAL}\n\
                       AND scope.runtime_principal_prefix = {CURRENT_PRINCIPAL_PREFIX}\n\
                 ));"
            )
            .expect("writing Zulip owner policy cannot fail");
        }
    }
    sql
}

#[must_use]
pub fn zulip_owner_rls_storage_migration_v1() -> StorageMigrationStepV1 {
    let sql = zulip_owner_rls_sql_v1();
    StorageMigrationStepV1 {
        revision: ZULIP_OWNER_RLS_STORAGE_REVISION_V1,
        migration_id: "zulip_owner_scope_and_rls".to_owned(),
        sha256: Sha256::digest(sql.as_bytes()).to_vec(),
        forward_sql_utf8: sql.into_bytes(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn migration_force_isolates_the_exact_zulip_table_inventory() {
        let migration = zulip_owner_rls_storage_migration_v1();
        let sql = std::str::from_utf8(&migration.forward_sql_utf8).expect("owner RLS SQL");

        assert_eq!(migration.revision, 7);
        assert_eq!(migration.migration_id, "zulip_owner_scope_and_rls");
        assert_eq!(migration.sha256, Sha256::digest(sql.as_bytes()).to_vec());
        assert_eq!(
            Sha256::digest(sql.as_bytes())
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>(),
            "a52202393d28d8603383d021daefe7f44301d38d2911142f8d082a6365ec8e3a"
        );
        assert_eq!(
            ZULIP_OWNER_RLS_TABLES_V1
                .into_iter()
                .collect::<BTreeSet<_>>()
                .len(),
            ZULIP_OWNER_RLS_TABLES_V1.len(),
        );
        assert_eq!(sql.matches("ENABLE ROW LEVEL SECURITY").count(), 19);
        assert_eq!(sql.matches("FORCE ROW LEVEL SECURITY").count(), 19);
        assert_eq!(sql.matches("CREATE POLICY").count(), 19);
        for table in ZULIP_OWNER_RLS_TABLES_V1 {
            assert!(sql.contains(&format!(
                "ALTER TABLE makosh_data.{table} ENABLE ROW LEVEL SECURITY"
            )));
            assert!(sql.contains(&format!(
                "ALTER TABLE makosh_data.{table} FORCE ROW LEVEL SECURITY"
            )));
        }
    }
}

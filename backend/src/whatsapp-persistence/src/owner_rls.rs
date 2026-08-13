//! Immutable owner-scope and RLS migration for the complete WhatsApp store.

use std::fmt::Write as _;

use makosh_storage_protocol::v1::StorageMigrationStepV1;
use sha2::{Digest, Sha256};

pub const WHATSAPP_OWNER_RLS_STORAGE_REVISION_V1: u32 = 5;

/// Every WhatsApp-owned table present after storage revision 4, plus the
/// revision-5 owner-scope table. This sorted inventory is part of the signed
/// storage artifact.
pub const WHATSAPP_OWNER_RLS_TABLES_V1: [&str; 17] = [
    "whatsapp_communications_outbox",
    "whatsapp_delivery_intent_inbox",
    "whatsapp_delivery_intent_jobs",
    "whatsapp_delivery_intent_result_outbox",
    "whatsapp_delivery_route_accounts",
    "whatsapp_delivery_route_conversations",
    "whatsapp_delivery_route_messages",
    "whatsapp_host_observations",
    "whatsapp_operational_controls",
    "whatsapp_operational_dialogs",
    "whatsapp_operational_events",
    "whatsapp_operational_messages",
    "whatsapp_operational_participants",
    "whatsapp_operational_runtime_status",
    "whatsapp_operational_tombstones",
    "whatsapp_owner_scope",
    "whatsapp_provider_commands",
];

const OWNER_SCOPE_TABLE: &str = "whatsapp_owner_scope";
const CURRENT_PRINCIPAL_PREFIX: &str = "regexp_replace(current_user::text, '_[0-9]+$', '')";
const CURRENT_RUNTIME_PRINCIPAL: &str = "current_user::text ~ '^storage_[a-f0-9]{16}_[1-9][0-9]*$'";

#[must_use]
pub fn whatsapp_owner_rls_sql_v1() -> String {
    let mut sql = String::from(
        "CREATE TABLE IF NOT EXISTS makosh_data.whatsapp_owner_scope (\n\
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

    for table in WHATSAPP_OWNER_RLS_TABLES_V1 {
        writeln!(
            sql,
            "ALTER TABLE makosh_data.{table} ENABLE ROW LEVEL SECURITY;\n\
             ALTER TABLE makosh_data.{table} FORCE ROW LEVEL SECURITY;"
        )
        .expect("writing WhatsApp owner RLS SQL cannot fail");
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
            .expect("writing WhatsApp owner-scope policy cannot fail");
        } else {
            writeln!(
                sql,
                "CREATE POLICY {table}_owner_isolation_v1\n\
                 ON makosh_data.{table}\n\
                 USING (EXISTS (\n\
                     SELECT 1 FROM makosh_data.whatsapp_owner_scope AS scope\n\
                     WHERE {CURRENT_RUNTIME_PRINCIPAL}\n\
                       AND scope.runtime_principal_prefix = {CURRENT_PRINCIPAL_PREFIX}\n\
                 ))\n\
                 WITH CHECK (EXISTS (\n\
                     SELECT 1 FROM makosh_data.whatsapp_owner_scope AS scope\n\
                     WHERE {CURRENT_RUNTIME_PRINCIPAL}\n\
                       AND scope.runtime_principal_prefix = {CURRENT_PRINCIPAL_PREFIX}\n\
                 ));"
            )
            .expect("writing WhatsApp owner policy cannot fail");
        }
    }
    sql
}

#[must_use]
pub fn whatsapp_owner_rls_storage_migration_v1() -> StorageMigrationStepV1 {
    let sql = whatsapp_owner_rls_sql_v1();
    StorageMigrationStepV1 {
        revision: WHATSAPP_OWNER_RLS_STORAGE_REVISION_V1,
        migration_id: "whatsapp_owner_scope_and_rls".to_owned(),
        sha256: Sha256::digest(sql.as_bytes()).to_vec(),
        forward_sql_utf8: sql.into_bytes(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn migration_force_isolates_the_exact_whatsapp_table_inventory() {
        let migration = whatsapp_owner_rls_storage_migration_v1();
        let sql = std::str::from_utf8(&migration.forward_sql_utf8).expect("owner RLS SQL");

        assert_eq!(migration.revision, 5);
        assert_eq!(migration.migration_id, "whatsapp_owner_scope_and_rls");
        assert_eq!(migration.sha256, Sha256::digest(sql.as_bytes()).to_vec());
        assert_eq!(
            WHATSAPP_OWNER_RLS_TABLES_V1
                .into_iter()
                .collect::<BTreeSet<_>>()
                .len(),
            WHATSAPP_OWNER_RLS_TABLES_V1.len(),
        );
        assert_eq!(sql.matches("ENABLE ROW LEVEL SECURITY").count(), 17);
        assert_eq!(sql.matches("FORCE ROW LEVEL SECURITY").count(), 17);
        assert_eq!(sql.matches("CREATE POLICY").count(), 17);
        for table in WHATSAPP_OWNER_RLS_TABLES_V1 {
            assert!(sql.contains(&format!(
                "ALTER TABLE makosh_data.{table} ENABLE ROW LEVEL SECURITY"
            )));
            assert!(sql.contains(&format!(
                "ALTER TABLE makosh_data.{table} FORCE ROW LEVEL SECURITY"
            )));
        }
    }
}

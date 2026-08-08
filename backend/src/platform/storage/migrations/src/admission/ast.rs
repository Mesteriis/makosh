//! Fail-closed PostgreSQL AST admission for owner-local additive DDL.

use pg_query::{
    NodeEnum,
    protobuf::{AlterTableType, ConstrType, Node},
};

use super::{
    error::MigrationAdmissionErrorV1,
    owner::{is_owned_relation, valid_owner},
};

pub fn admit_owner_local_additive_sql(
    owner: &str,
    sql: &str,
) -> Result<(), MigrationAdmissionErrorV1> {
    if !valid_owner(owner) {
        return Err(MigrationAdmissionErrorV1::Owner);
    }
    let parsed = pg_query::parse(sql).map_err(|_| MigrationAdmissionErrorV1::Syntax)?;
    if parsed.protobuf.stmts.is_empty() {
        return Err(MigrationAdmissionErrorV1::Syntax);
    }
    for statement in &parsed.protobuf.stmts {
        let node = statement
            .stmt
            .as_ref()
            .and_then(|statement| statement.node.as_ref())
            .ok_or(MigrationAdmissionErrorV1::Syntax)?;
        admit_statement(node, owner)?;
    }
    Ok(())
}

fn admit_statement(node: &NodeEnum, owner: &str) -> Result<(), MigrationAdmissionErrorV1> {
    match node {
        NodeEnum::CreateStmt(statement) if create_table_is_additive(statement, owner) => Ok(()),
        NodeEnum::IndexStmt(statement) if create_index_is_additive(statement, owner) => Ok(()),
        NodeEnum::AlterTableStmt(statement) if alter_table_is_additive(statement, owner) => Ok(()),
        _ => Err(MigrationAdmissionErrorV1::Forbidden),
    }
}

fn create_table_is_additive(statement: &pg_query::protobuf::CreateStmt, owner: &str) -> bool {
    is_owned_relation(statement.relation.as_ref(), owner)
        && !statement.table_elts.is_empty()
        && statement.table_elts.iter().all(|element| {
            is_column_definition(element) || is_owner_local_table_constraint(element, owner)
        })
        && statement
            .constraints
            .iter()
            .all(|constraint| is_owner_local_table_constraint(constraint, owner))
        && statement.inh_relations.is_empty()
        && statement.partbound.is_none()
        && statement.partspec.is_none()
        && statement.of_typename.is_none()
        && statement.tablespacename.is_empty()
        && statement.access_method.is_empty()
}

fn create_index_is_additive(statement: &pg_query::protobuf::IndexStmt, owner: &str) -> bool {
    is_owned_relation(statement.relation.as_ref(), owner)
}

fn alter_table_is_additive(statement: &pg_query::protobuf::AlterTableStmt, owner: &str) -> bool {
    is_owned_relation(statement.relation.as_ref(), owner)
        && !statement.cmds.is_empty()
        && statement.cmds.iter().all(is_additive_alter)
}

fn is_column_definition(node: &Node) -> bool {
    matches!(node.node.as_ref(), Some(NodeEnum::ColumnDef(_)))
}

fn is_additive_alter(node: &Node) -> bool {
    let Some(NodeEnum::AlterTableCmd(command)) = node.node.as_ref() else {
        eprintln!("developer_storage_migration_alter_rejected=missing_command");
        return false;
    };
    let subtype = AlterTableType::try_from(command.subtype);
    let admitted = match subtype {
        Ok(AlterTableType::AtAddColumn) => command
            .def
            .as_ref()
            .is_some_and(|definition| is_column_definition(definition)),
        Ok(AlterTableType::AtAddConstraint) => command
            .def
            .as_ref()
            .is_some_and(|definition| is_check_constraint(definition)),
        _ => false,
    };
    if !admitted {
        eprintln!("developer_storage_migration_alter_rejected=subtype_{subtype:?}");
    }
    admitted
}

fn is_check_constraint(node: &Node) -> bool {
    matches!(node.node.as_ref(), Some(NodeEnum::Constraint(constraint))
        if !constraint.conname.is_empty()
            && matches!(ConstrType::try_from(constraint.contype), Ok(ConstrType::ConstrCheck))
            && !constraint.deferrable
            && !constraint.initdeferred
            && !constraint.skip_validation
            && constraint.pktable.is_none()
            && constraint.fk_attrs.is_empty()
            && constraint.pk_attrs.is_empty())
}

fn is_owner_local_table_constraint(node: &Node, owner: &str) -> bool {
    let Some(NodeEnum::Constraint(constraint)) = node.node.as_ref() else {
        return false;
    };
    match ConstrType::try_from(constraint.contype) {
        Ok(ConstrType::ConstrPrimary | ConstrType::ConstrUnique | ConstrType::ConstrCheck) => true,
        Ok(ConstrType::ConstrForeign) => is_owned_relation(constraint.pktable.as_ref(), owner),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::admit_owner_local_additive_sql;

    #[test]
    fn admits_owner_local_boolean_marker_column() {
        admit_owner_local_additive_sql(
            "mail",
            r#"
ALTER TABLE hermes_data.mail_sync_runs
    ADD COLUMN deadline_exceeded BOOLEAN NOT NULL DEFAULT FALSE;
"#,
        )
        .expect("owner-local additive marker column");
    }
}

//! Owner-local relation naming rules.

use pg_query::protobuf::RangeVar;

pub(super) fn valid_owner(owner: &str) -> bool {
    !owner.is_empty()
        && owner.len() <= 96
        && owner
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

pub(super) fn is_owned_relation(relation: Option<&RangeVar>, owner: &str) -> bool {
    let Some(relation) = relation else {
        return false;
    };
    let schema = if owner == "scheduler" {
        "makosh_platform"
    } else {
        "makosh_data"
    };
    relation.catalogname.is_empty()
        && relation.schemaname == schema
        && relation.relname.starts_with(&format!("{owner}_"))
}

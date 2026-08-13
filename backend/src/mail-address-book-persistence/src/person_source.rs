use std::collections::BTreeSet;

use crate::MailAddressBookPersistenceErrorV1;

pub fn terminal_snapshot_tombstones_v1(
    terminal_snapshot_succeeded: bool,
    active_public_source_ids: &[[u8; 16]],
    seen_public_source_ids: &[[u8; 16]],
) -> Result<Vec<[u8; 16]>, MailAddressBookPersistenceErrorV1> {
    let active = active_public_source_ids
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let seen = seen_public_source_ids
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    if active.len() != active_public_source_ids.len()
        || seen.len() != seen_public_source_ids.len()
        || active.iter().any(|id| id.iter().all(|byte| *byte == 0))
        || seen.iter().any(|id| id.iter().all(|byte| *byte == 0))
    {
        return Err(MailAddressBookPersistenceErrorV1::InvalidInput);
    }
    if !terminal_snapshot_succeeded {
        return Ok(Vec::new());
    }
    Ok(active.difference(&seen).copied().collect())
}

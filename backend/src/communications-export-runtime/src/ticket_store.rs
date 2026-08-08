//! Runtime-local, owner-bound, one-use artifact read authority.

use std::sync::Mutex;

use makosh_communications_export_api::{
    COMMUNICATIONS_EXPORT_MAX_ARTIFACT_BYTES_V1, COMMUNICATIONS_EXPORT_READ_TICKET_BYTES_V1,
    COMMUNICATIONS_EXPORT_READ_TICKET_TTL_SECONDS_V1,
};
use makosh_communications_export_persistence::CommunicationsExportArtifactReceiptV1;

const TOKEN_GENERATION_ATTEMPTS: usize = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommunicationsExportTicketStoreErrorV1 {
    InvalidRequest,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssuedCommunicationsExportTicketV1 {
    pub opaque_read_capability: [u8; COMMUNICATIONS_EXPORT_READ_TICKET_BYTES_V1],
    pub declared_bytes: u64,
    pub expires_at_unix_seconds: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConsumedCommunicationsExportTicketV1 {
    pub export_id: [u8; 16],
    pub artifact: CommunicationsExportArtifactReceiptV1,
}

#[derive(Debug)]
pub struct CommunicationsExportTicketStoreV1 {
    runtime_generation: u64,
    grant_epoch: u64,
    tickets: Mutex<Vec<TicketAuthorityV1>>,
}

impl CommunicationsExportTicketStoreV1 {
    pub fn new(
        runtime_generation: u64,
        grant_epoch: u64,
    ) -> Result<Self, CommunicationsExportTicketStoreErrorV1> {
        if runtime_generation == 0 || grant_epoch == 0 {
            return Err(CommunicationsExportTicketStoreErrorV1::InvalidRequest);
        }
        Ok(Self {
            runtime_generation,
            grant_epoch,
            tickets: Mutex::new(Vec::new()),
        })
    }

    pub fn issue(
        &self,
        logical_owner_id: &str,
        export_id: [u8; 16],
        artifact: CommunicationsExportArtifactReceiptV1,
        now_unix_seconds: i64,
    ) -> Result<IssuedCommunicationsExportTicketV1, CommunicationsExportTicketStoreErrorV1> {
        if !valid_owner(logical_owner_id)
            || !valid_id(&export_id)
            || !valid_artifact(&artifact)
            || now_unix_seconds <= 0
        {
            return Err(CommunicationsExportTicketStoreErrorV1::InvalidRequest);
        }
        let expires_at_unix_seconds = now_unix_seconds
            .checked_add(COMMUNICATIONS_EXPORT_READ_TICKET_TTL_SECONDS_V1)
            .ok_or(CommunicationsExportTicketStoreErrorV1::Unavailable)?;
        let mut tickets = self
            .tickets
            .lock()
            .map_err(|_| CommunicationsExportTicketStoreErrorV1::Unavailable)?;
        tickets.retain(|ticket| ticket.expires_at_unix_seconds > now_unix_seconds);
        for _ in 0..TOKEN_GENERATION_ATTEMPTS {
            let mut capability = [0_u8; COMMUNICATIONS_EXPORT_READ_TICKET_BYTES_V1];
            getrandom::fill(&mut capability)
                .map_err(|_| CommunicationsExportTicketStoreErrorV1::Unavailable)?;
            if capability.iter().all(|byte| *byte == 0)
                || tickets
                    .iter()
                    .any(|ticket| ticket.opaque_read_capability == capability)
            {
                continue;
            }
            tickets.push(TicketAuthorityV1 {
                opaque_read_capability: capability,
                logical_owner_id: logical_owner_id.to_owned(),
                export_id,
                artifact,
                runtime_generation: self.runtime_generation,
                grant_epoch: self.grant_epoch,
                expires_at_unix_seconds,
            });
            return Ok(IssuedCommunicationsExportTicketV1 {
                opaque_read_capability: capability,
                declared_bytes: artifact.declared_bytes,
                expires_at_unix_seconds,
            });
        }
        Err(CommunicationsExportTicketStoreErrorV1::Unavailable)
    }

    pub fn consume(
        &self,
        capability: [u8; COMMUNICATIONS_EXPORT_READ_TICKET_BYTES_V1],
        logical_owner_id: &str,
        runtime_generation: u64,
        grant_epoch: u64,
        now_unix_seconds: i64,
    ) -> Result<Option<ConsumedCommunicationsExportTicketV1>, CommunicationsExportTicketStoreErrorV1>
    {
        if capability.iter().all(|byte| *byte == 0)
            || !valid_owner(logical_owner_id)
            || runtime_generation != self.runtime_generation
            || grant_epoch != self.grant_epoch
            || now_unix_seconds <= 0
        {
            return Err(CommunicationsExportTicketStoreErrorV1::InvalidRequest);
        }
        let mut tickets = self
            .tickets
            .lock()
            .map_err(|_| CommunicationsExportTicketStoreErrorV1::Unavailable)?;
        tickets.retain(|ticket| ticket.expires_at_unix_seconds > now_unix_seconds);
        let Some(position) = tickets
            .iter()
            .position(|ticket| ticket.opaque_read_capability == capability)
        else {
            return Ok(None);
        };
        let ticket = &tickets[position];
        if ticket.logical_owner_id != logical_owner_id
            || ticket.runtime_generation != runtime_generation
            || ticket.grant_epoch != grant_epoch
            || ticket.expires_at_unix_seconds <= now_unix_seconds
        {
            return Ok(None);
        }
        let ticket = tickets.remove(position);
        Ok(Some(ConsumedCommunicationsExportTicketV1 {
            export_id: ticket.export_id,
            artifact: ticket.artifact,
        }))
    }
}

#[derive(Debug)]
struct TicketAuthorityV1 {
    opaque_read_capability: [u8; COMMUNICATIONS_EXPORT_READ_TICKET_BYTES_V1],
    logical_owner_id: String,
    export_id: [u8; 16],
    artifact: CommunicationsExportArtifactReceiptV1,
    runtime_generation: u64,
    grant_epoch: u64,
    expires_at_unix_seconds: i64,
}

fn valid_owner(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128 && value.is_ascii()
}

fn valid_id(value: &[u8; 16]) -> bool {
    value.iter().any(|byte| *byte != 0)
}

fn valid_artifact(value: &CommunicationsExportArtifactReceiptV1) -> bool {
    valid_id(&value.reference_id)
        && value.declared_bytes > 0
        && value.declared_bytes <= COMMUNICATIONS_EXPORT_MAX_ARTIFACT_BYTES_V1
        && value.sha256.iter().any(|byte| *byte != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn artifact() -> CommunicationsExportArtifactReceiptV1 {
        CommunicationsExportArtifactReceiptV1 {
            reference_id: [2; 16],
            declared_bytes: 42,
            sha256: [3; 32],
        }
    }

    #[test]
    fn ticket_is_one_use_owner_and_generation_bound() {
        let store = CommunicationsExportTicketStoreV1::new(7, 9).expect("store");
        let issued = store
            .issue("owner-1", [1; 16], artifact(), 100)
            .expect("issue");
        assert_eq!(
            store
                .consume(issued.opaque_read_capability, "owner-2", 7, 9, 101)
                .expect("consume"),
            None
        );
        assert_eq!(
            store
                .consume(issued.opaque_read_capability, "owner-1", 7, 9, 102)
                .expect("consume")
                .expect("ticket")
                .artifact,
            artifact()
        );
        let issued = store
            .issue("owner-1", [1; 16], artifact(), 200)
            .expect("issue");
        assert_eq!(
            store.consume(issued.opaque_read_capability, "owner-1", 8, 9, 201),
            Err(CommunicationsExportTicketStoreErrorV1::InvalidRequest)
        );
        assert!(
            store
                .consume(issued.opaque_read_capability, "owner-1", 7, 9, 201)
                .expect("consume after rejected generation")
                .is_some()
        );
        assert_eq!(
            store
                .consume(issued.opaque_read_capability, "owner-1", 7, 9, 201)
                .expect("replay"),
            None
        );
    }
}

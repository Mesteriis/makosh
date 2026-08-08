//! Runtime-local, one-use authorization tickets for canonical body content.

use std::sync::Mutex;

use makosh_communications_api::CommunicationMessageIdV1;
use makosh_communications_persistence::CommunicationsBodyContentReceiptV1;

const TICKET_BYTES: usize = 32;
const TICKET_TTL_SECONDS: i64 = 30;
const TOKEN_GENERATION_ATTEMPTS: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommunicationsContentTicketStoreErrorV1 {
    InvalidRequest,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssuedCommunicationsContentTicketV1 {
    pub opaque_read_capability: [u8; TICKET_BYTES],
    pub declared_bytes: u64,
    pub expires_at_unix_seconds: i64,
    pub media_type: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsumedCommunicationsContentTicketV1 {
    pub message_id: CommunicationMessageIdV1,
    pub receipt: CommunicationsBodyContentReceiptV1,
}

#[derive(Debug)]
pub struct CommunicationsContentTicketStoreV1 {
    tickets: Mutex<Vec<ContentTicketAuthorityV1>>,
}

impl CommunicationsContentTicketStoreV1 {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tickets: Mutex::new(Vec::new()),
        }
    }

    pub fn issue(
        &self,
        logical_owner_id: &str,
        message_id: CommunicationMessageIdV1,
        receipt: CommunicationsBodyContentReceiptV1,
        now_unix_seconds: i64,
    ) -> Result<IssuedCommunicationsContentTicketV1, CommunicationsContentTicketStoreErrorV1> {
        if logical_owner_id.is_empty()
            || logical_owner_id.len() > 128
            || !logical_owner_id.is_ascii()
        {
            return Err(CommunicationsContentTicketStoreErrorV1::InvalidRequest);
        }
        let expires_at_unix_seconds = now_unix_seconds
            .checked_add(TICKET_TTL_SECONDS)
            .ok_or(CommunicationsContentTicketStoreErrorV1::Unavailable)?;
        let mut tickets = self
            .tickets
            .lock()
            .map_err(|_| CommunicationsContentTicketStoreErrorV1::Unavailable)?;
        tickets.retain(|ticket| ticket.expires_at_unix_seconds > now_unix_seconds);

        for _ in 0..TOKEN_GENERATION_ATTEMPTS {
            let mut capability = [0_u8; TICKET_BYTES];
            getrandom::fill(&mut capability)
                .map_err(|_| CommunicationsContentTicketStoreErrorV1::Unavailable)?;
            if capability.iter().all(|byte| *byte == 0)
                || tickets
                    .iter()
                    .any(|ticket| ticket.opaque_read_capability == capability)
            {
                continue;
            }
            let declared_bytes = receipt.declared_bytes;
            let media_type = receipt.media_type.clone();
            tickets.push(ContentTicketAuthorityV1 {
                opaque_read_capability: capability,
                logical_owner_id: logical_owner_id.to_owned(),
                message_id,
                receipt,
                expires_at_unix_seconds,
            });
            return Ok(IssuedCommunicationsContentTicketV1 {
                opaque_read_capability: capability,
                declared_bytes,
                expires_at_unix_seconds,
                media_type,
            });
        }
        Err(CommunicationsContentTicketStoreErrorV1::Unavailable)
    }

    pub fn consume(
        &self,
        opaque_read_capability: [u8; TICKET_BYTES],
        logical_owner_id: &str,
        now_unix_seconds: i64,
    ) -> Result<
        Option<ConsumedCommunicationsContentTicketV1>,
        CommunicationsContentTicketStoreErrorV1,
    > {
        let mut tickets = self
            .tickets
            .lock()
            .map_err(|_| CommunicationsContentTicketStoreErrorV1::Unavailable)?;
        tickets.retain(|ticket| ticket.expires_at_unix_seconds > now_unix_seconds);
        let Some(position) = tickets
            .iter()
            .position(|ticket| ticket.opaque_read_capability == opaque_read_capability)
        else {
            return Ok(None);
        };
        let ticket = tickets.remove(position);
        if ticket.logical_owner_id != logical_owner_id
            || ticket.expires_at_unix_seconds <= now_unix_seconds
        {
            return Ok(None);
        }
        Ok(Some(ConsumedCommunicationsContentTicketV1 {
            message_id: ticket.message_id,
            receipt: ticket.receipt,
        }))
    }
}

impl Default for CommunicationsContentTicketStoreV1 {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct ContentTicketAuthorityV1 {
    opaque_read_capability: [u8; TICKET_BYTES],
    logical_owner_id: String,
    message_id: CommunicationMessageIdV1,
    receipt: CommunicationsBodyContentReceiptV1,
    expires_at_unix_seconds: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticket_is_owner_bound_one_use_and_expires() {
        let store = CommunicationsContentTicketStoreV1::new();
        let issued = store
            .issue("owner-1", message_id(), receipt(), 100)
            .expect("issue");
        assert_eq!(
            store
                .consume(issued.opaque_read_capability, "owner-1", 101)
                .expect("consume")
                .expect("ticket")
                .receipt,
            receipt()
        );
        assert_eq!(
            store
                .consume(issued.opaque_read_capability, "owner-1", 101)
                .expect("replay"),
            None
        );

        let wrong_owner = store
            .issue("owner-1", message_id(), receipt(), 200)
            .expect("issue");
        assert_eq!(
            store
                .consume(wrong_owner.opaque_read_capability, "owner-2", 201)
                .expect("wrong owner"),
            None
        );
        assert_eq!(
            store
                .consume(wrong_owner.opaque_read_capability, "owner-1", 201)
                .expect("burned ticket"),
            None
        );

        let expired = store
            .issue("owner-1", message_id(), receipt(), 300)
            .expect("issue");
        assert_eq!(
            store
                .consume(expired.opaque_read_capability, "owner-1", 330)
                .expect("expired"),
            None
        );
    }

    fn message_id() -> CommunicationMessageIdV1 {
        CommunicationMessageIdV1::new([1; 16])
    }

    fn receipt() -> CommunicationsBodyContentReceiptV1 {
        CommunicationsBodyContentReceiptV1 {
            reference_id: [2; 16],
            declared_bytes: 64,
            plaintext_sha256: [3; 32],
            backup_class: 1,
            media_type: "text/plain".to_owned(),
        }
    }
}

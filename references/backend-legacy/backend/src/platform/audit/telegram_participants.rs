use crate::platform::capabilities::{CapabilityActionClass, CapabilityDecision};

use super::helpers::insert_non_empty;
use super::models::NewApiAuditRecord;

impl NewApiAuditRecord {
    pub fn telegram_participants_sync(
        actor_id: impl Into<String>,
        telegram_chat_id: impl Into<String>,
        account_id: impl Into<String>,
        provider_chat_id: impl Into<String>,
        synced_count: i64,
    ) -> Self {
        let telegram_chat_id = telegram_chat_id.into();
        let mut metadata = CapabilityDecision::explicit_user_allowed(
            CapabilityActionClass::Read,
            "telegram.participants.sync",
            "explicit_user_confirmation",
        )
        .audit_metadata();
        let metadata_object = metadata
            .as_object_mut()
            .expect("capability decision metadata must be an object");
        insert_non_empty(metadata_object, "account_id", account_id.into());
        insert_non_empty(metadata_object, "provider_chat_id", provider_chat_id.into());
        insert_non_empty(
            metadata_object,
            "synced_count",
            synced_count.max(0).to_string(),
        );

        Self::new(
            actor_id,
            "telegram.participants.sync",
            "POST",
            "/api/v1/integrations/telegram/provider-sync/conversations/{chat_id}/members",
            "telegram_chat",
            Some(telegram_chat_id),
            metadata,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::platform::audit::models::NewApiAuditRecord;

    #[test]
    fn telegram_participants_sync_audit_preserves_capability_metadata() {
        let record = NewApiAuditRecord::telegram_participants_sync(
            "makosh-frontend",
            "telegram-chat-1",
            "account-1",
            "provider-chat-1",
            42,
        );

        assert_eq!(record.operation, "telegram.participants.sync");
        assert_eq!(record.method, "POST");
        assert_eq!(
            record.path_template,
            "/api/v1/integrations/telegram/provider-sync/conversations/{chat_id}/members"
        );
        assert_eq!(record.target_kind, "telegram_chat");
        assert_eq!(record.target_id.as_deref(), Some("telegram-chat-1"));
        assert_eq!(record.metadata["capability"], "telegram.participants.sync");
        assert_eq!(record.metadata["decision"], "allowed");
        assert_eq!(record.metadata["reason"], "explicit_user_confirmation");
        assert_eq!(record.metadata["account_id"], "account-1");
        assert_eq!(record.metadata["provider_chat_id"], "provider-chat-1");
        assert_eq!(record.metadata["synced_count"], "42");
    }
}

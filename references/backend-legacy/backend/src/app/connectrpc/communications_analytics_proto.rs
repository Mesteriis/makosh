use makosh_connectrpc_contracts::makosh::communications::v1::{
    MailboxHealth as ProtoMailboxHealth, SenderStats as ProtoSenderStats,
    SubscriptionSource as ProtoSubscriptionSource,
};

use crate::domains::communications::analytics::{MailboxHealth, SenderStats};
use crate::domains::communications::subscriptions::SubscriptionSource;

pub(super) fn subscription_source(item: SubscriptionSource) -> ProtoSubscriptionSource {
    ProtoSubscriptionSource {
        sender: item.sender,
        message_count: item.message_count,
        first_seen: item.first_seen,
        last_seen: item.last_seen,
        is_newsletter: item.is_newsletter,
        has_unsubscribe: item.has_unsubscribe,
        ..Default::default()
    }
}
pub(super) fn mailbox_health(item: MailboxHealth) -> ProtoMailboxHealth {
    ProtoMailboxHealth {
        total_messages: item.total_messages,
        unread: item.unread,
        needs_action: item.needs_action,
        waiting: item.waiting,
        done: item.done,
        archived: item.archived,
        spam: item.spam,
        important: item.important,
        with_attachments: item.with_attachments,
        average_importance: item.average_importance,
        oldest_message_days: item.oldest_message_days,
        ..Default::default()
    }
}
pub(super) fn sender_stats(item: SenderStats) -> ProtoSenderStats {
    ProtoSenderStats {
        sender: item.sender,
        message_count: item.message_count,
        avg_importance: item.avg_importance,
        last_message_days: item.last_message_days,
        ..Default::default()
    }
}

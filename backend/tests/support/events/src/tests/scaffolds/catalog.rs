//! Exact delivery namespaces reserved for the first owner-local outbox/inbox slice.

use super::owners;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OwnerDeliveryScaffoldV1 {
    owner_id: &'static str,
    schema_name: &'static str,
}

impl OwnerDeliveryScaffoldV1 {
    pub(crate) const fn new(owner_id: &'static str, schema_name: &'static str) -> Self {
        Self {
            owner_id,
            schema_name,
        }
    }

    pub(crate) const ALL: [Self; 7] = [
        owners::communications::SCAFFOLD,
        owners::contacts::SCAFFOLD,
        owners::organizations::SCAFFOLD,
        owners::tasks::SCAFFOLD,
        owners::calendar::SCAFFOLD,
        owners::documents::SCAFFOLD,
        owners::ai::SCAFFOLD,
    ];

    #[must_use]
    pub(crate) const fn owner_id(self) -> &'static str {
        self.owner_id
    }

    #[must_use]
    pub(crate) const fn schema_name(self) -> &'static str {
        self.schema_name
    }

    #[must_use]
    pub(crate) const fn outbox_table(self) -> &'static str {
        "durable_outbox_v1"
    }

    #[must_use]
    pub(crate) const fn inbox_table(self) -> &'static str {
        "durable_inbox_v1"
    }
}

#[test]
fn reserves_exact_owner_local_delivery_namespaces_without_creating_domain_behavior() {
    let actual = OwnerDeliveryScaffoldV1::ALL.map(|scaffold| {
        (
            scaffold.owner_id(),
            scaffold.schema_name(),
            scaffold.outbox_table(),
            scaffold.inbox_table(),
        )
    });
    assert_eq!(
        actual,
        [
            (
                "communications",
                "makosh_communications",
                "durable_outbox_v1",
                "durable_inbox_v1"
            ),
            (
                "contacts",
                "makosh_contacts",
                "durable_outbox_v1",
                "durable_inbox_v1"
            ),
            (
                "organizations",
                "makosh_organizations",
                "durable_outbox_v1",
                "durable_inbox_v1"
            ),
            (
                "tasks",
                "makosh_tasks",
                "durable_outbox_v1",
                "durable_inbox_v1"
            ),
            (
                "calendar",
                "makosh_calendar",
                "durable_outbox_v1",
                "durable_inbox_v1"
            ),
            (
                "documents",
                "makosh_documents",
                "durable_outbox_v1",
                "durable_inbox_v1"
            ),
            ("ai", "makosh_ai", "durable_outbox_v1", "durable_inbox_v1"),
        ]
    );
}

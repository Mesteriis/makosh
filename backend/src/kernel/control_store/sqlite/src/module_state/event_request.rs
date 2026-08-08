//! SQLite persistence for descriptor-declared Event Hub route requests.

use std::collections::BTreeSet;

use makosh_kernel_control_store::{
    ModuleEventDeliveryPolicyV1, ModuleEventEnvelopeKindV1, ModuleEventRouteDirectionV1,
    ModuleEventRouteRequestInputV1, ModuleEventRouteRequestV1,
    ModuleEventSubscriptionRequirementV1, ModuleRegistration,
};
use rusqlite::{Connection, params};

use crate::{SqliteControlStore, StoreError, valid_capability_ids, valid_identity_token};

impl SqliteControlStore {
    pub fn module_event_route_requests(
        &self,
        registration_id: &str,
        capability_id: &str,
    ) -> Result<Vec<ModuleEventRouteRequestV1>, StoreError> {
        let registration_id = registration_id.to_owned();
        let capability_id = capability_id.to_owned();
        self.with_connection(move |connection| {
            read_event_route_requests(connection, &registration_id, &capability_id)
        })
    }
}

pub(crate) fn validate_event_route_requests(
    registration: &ModuleRegistration,
    capabilities: &[String],
    requests: &[ModuleEventRouteRequestV1],
) -> Result<(), StoreError> {
    let requested = capabilities
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let valid = requests
        .iter()
        .all(|request| valid_event_route_request(registration, &requested, &mut seen, request));
    valid
        .then_some(())
        .ok_or(StoreError::InvalidModuleEventRouteRequest)
}

fn valid_event_route_request(
    registration: &ModuleRegistration,
    capabilities: &BTreeSet<&str>,
    seen: &mut BTreeSet<(String, i64, String, String, u32, i64)>,
    request: &ModuleEventRouteRequestV1,
) -> bool {
    request.registration_id() == registration.registration_id()
        && capabilities.contains(request.capability_id())
        && valid_capability_ids(&[request.capability_id().to_owned()])
        && valid_identity_token(request.contract_owner())
        && valid_identity_token(request.contract_name())
        && request.contract_major() > 0
        && request.contract_revision() > 0
        && (1..=4_096).contains(&request.max_in_flight())
        && valid_delivery_policy(request)
        && seen.insert(route_key(request))
}

fn valid_delivery_policy(request: &ModuleEventRouteRequestV1) -> bool {
    match request.direction() {
        ModuleEventRouteDirectionV1::Publish => request.delivery_policy().is_none(),
        ModuleEventRouteDirectionV1::Consume => request.delivery_policy().is_some_and(|policy| {
            matches!(
                policy.requirement(),
                ModuleEventSubscriptionRequirementV1::Required
                    | ModuleEventSubscriptionRequirementV1::Optional
            ) && (1..=32).contains(&policy.max_deliver())
                && (1..=600_000).contains(&policy.ack_wait_millis())
        }),
    }
}

fn route_key(request: &ModuleEventRouteRequestV1) -> (String, i64, String, String, u32, i64) {
    (
        request.capability_id().to_owned(),
        request.envelope_kind().as_i64(),
        request.contract_owner().to_owned(),
        request.contract_name().to_owned(),
        request.contract_major(),
        request.direction().as_i64(),
    )
}

pub(crate) fn insert_event_route_requests(
    connection: &Connection,
    requests: &[ModuleEventRouteRequestV1],
) -> Result<(), StoreError> {
    for request in requests {
        connection.execute(
            "INSERT INTO makosh_kernel_module_event_route_request
             (registration_id, capability_id, envelope_kind, contract_owner, contract_name,
              contract_major, contract_revision, contract_schema_sha256, direction, max_in_flight)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                request.registration_id(),
                request.capability_id(),
                request.envelope_kind().as_i64(),
                request.contract_owner(),
                request.contract_name(),
                i64::from(request.contract_major()),
                i64::from(request.contract_revision()),
                request.contract_schema_sha256().as_slice(),
                request.direction().as_i64(),
                i64::from(request.max_in_flight()),
            ],
        )?;
        if let Some(policy) = request.delivery_policy() {
            connection.execute(
                "INSERT INTO makosh_kernel_module_event_delivery_policy
                 (registration_id, capability_id, envelope_kind, contract_owner, contract_name,
                  contract_major, direction, subscription_requirement, max_deliver, ack_wait_millis)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    request.registration_id(),
                    request.capability_id(),
                    request.envelope_kind().as_i64(),
                    request.contract_owner(),
                    request.contract_name(),
                    i64::from(request.contract_major()),
                    request.direction().as_i64(),
                    policy.requirement().as_i64(),
                    i64::from(policy.max_deliver()),
                    i64::from(policy.ack_wait_millis()),
                ],
            )?;
        }
    }
    Ok(())
}

fn read_event_route_requests(
    connection: &Connection,
    registration_id: &str,
    capability_id: &str,
) -> Result<Vec<ModuleEventRouteRequestV1>, StoreError> {
    let mut statement = connection.prepare(
        "SELECT route.envelope_kind, route.contract_owner, route.contract_name, route.contract_major,
                route.contract_revision, route.contract_schema_sha256, route.direction,
                route.max_in_flight, policy.subscription_requirement, policy.max_deliver,
                policy.ack_wait_millis
         FROM makosh_kernel_module_event_route_request route
         LEFT JOIN makosh_kernel_module_event_delivery_policy policy
           ON policy.registration_id = route.registration_id
          AND policy.capability_id = route.capability_id
          AND policy.envelope_kind = route.envelope_kind
          AND policy.contract_owner = route.contract_owner
          AND policy.contract_name = route.contract_name
          AND policy.contract_major = route.contract_major
          AND policy.direction = route.direction
         WHERE route.registration_id = ?1 AND route.capability_id = ?2
         ORDER BY route.envelope_kind, route.contract_owner, route.contract_name,
                  route.contract_major, route.direction",
    )?;
    let rows = statement.query_map(params![registration_id, capability_id], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, Vec<u8>>(5)?,
            row.get::<_, i64>(6)?,
            row.get::<_, i64>(7)?,
            row.get::<_, Option<i64>>(8)?,
            row.get::<_, Option<i64>>(9)?,
            row.get::<_, Option<i64>>(10)?,
        ))
    })?;
    rows.map(|row| decode_event_route_request(row?, registration_id, capability_id))
        .collect()
}

type EventRouteRow = (
    i64,
    String,
    String,
    i64,
    i64,
    Vec<u8>,
    i64,
    i64,
    Option<i64>,
    Option<i64>,
    Option<i64>,
);

fn decode_event_route_request(
    row: EventRouteRow,
    registration_id: &str,
    capability_id: &str,
) -> Result<ModuleEventRouteRequestV1, StoreError> {
    let (
        kind,
        owner,
        name,
        major,
        revision,
        digest,
        direction,
        max_in_flight,
        requirement,
        max_deliver,
        ack_wait_millis,
    ) = row;
    let kind = ModuleEventEnvelopeKindV1::from_i64(kind)
        .ok_or(StoreError::InvalidModuleEventRouteRequest)?;
    let direction = ModuleEventRouteDirectionV1::from_i64(direction)
        .ok_or(StoreError::InvalidModuleEventRouteRequest)?;
    let digest: [u8; 32] = digest
        .try_into()
        .map_err(|_| StoreError::InvalidModuleEventRouteRequest)?;
    let major = u32::try_from(major).map_err(|_| StoreError::InvalidModuleEventRouteRequest)?;
    let revision =
        u32::try_from(revision).map_err(|_| StoreError::InvalidModuleEventRouteRequest)?;
    let max_in_flight =
        u16::try_from(max_in_flight).map_err(|_| StoreError::InvalidModuleEventRouteRequest)?;
    let delivery_policy =
        decode_delivery_policy(direction, requirement, max_deliver, ack_wait_millis)?;
    Ok(ModuleEventRouteRequestV1::new(
        ModuleEventRouteRequestInputV1 {
            registration_id: registration_id.to_owned(),
            capability_id: capability_id.to_owned(),
            envelope_kind: kind,
            contract_owner: owner,
            contract_name: name,
            contract_major: major,
            contract_revision: revision,
            contract_schema_sha256: digest,
            direction,
            max_in_flight,
            delivery_policy,
        },
    ))
}

fn decode_delivery_policy(
    direction: ModuleEventRouteDirectionV1,
    requirement: Option<i64>,
    max_deliver: Option<i64>,
    ack_wait_millis: Option<i64>,
) -> Result<Option<ModuleEventDeliveryPolicyV1>, StoreError> {
    match direction {
        ModuleEventRouteDirectionV1::Publish => {
            if requirement.is_none() && max_deliver.is_none() && ack_wait_millis.is_none() {
                Ok(None)
            } else {
                Err(StoreError::InvalidModuleEventRouteRequest)
            }
        }
        ModuleEventRouteDirectionV1::Consume => {
            let requirement = ModuleEventSubscriptionRequirementV1::from_i64(
                requirement.ok_or(StoreError::InvalidModuleEventRouteRequest)?,
            )
            .ok_or(StoreError::InvalidModuleEventRouteRequest)?;
            let max_deliver =
                u8::try_from(max_deliver.ok_or(StoreError::InvalidModuleEventRouteRequest)?)
                    .map_err(|_| StoreError::InvalidModuleEventRouteRequest)?;
            let ack_wait_millis =
                u32::try_from(ack_wait_millis.ok_or(StoreError::InvalidModuleEventRouteRequest)?)
                    .map_err(|_| StoreError::InvalidModuleEventRouteRequest)?;
            Ok(Some(ModuleEventDeliveryPolicyV1::new(
                requirement,
                max_deliver,
                ack_wait_millis,
            )))
        }
    }
}

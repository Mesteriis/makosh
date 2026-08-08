use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::domains::personas::api::models::Persona;

#[derive(Serialize)]
pub(crate) struct EnrichedPersonaListResponse {
    pub(super) items: Vec<crate::domains::personas::enrichment::models::EnrichedPersona>,
}

#[derive(Serialize)]
pub(crate) struct PersonaListResponse {
    pub(super) items: Vec<PersonaReadModel>,
}

#[derive(Serialize)]
pub(crate) struct PersonaReadModel {
    persona_id: String,
    persona_type: crate::domains::personas::api::models::PersonaType,
    is_self: bool,
    is_address_book: bool,
    identity: PersonaIdentityReadModel,
    communication: PersonaCommunicationReadModel,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub(crate) struct PersonaIdentityReadModel {
    display_name: String,
    email_address: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct PersonaCommunicationReadModel {
    primary_email: Option<String>,
}

pub(super) fn persona_read_model(person: Persona) -> PersonaReadModel {
    PersonaReadModel {
        persona_id: person.persona_id.clone(),
        persona_type: person.persona_type,
        is_self: person.is_self,
        is_address_book: person.is_address_book,
        identity: PersonaIdentityReadModel {
            display_name: person.display_name,
            email_address: person.email_address.clone(),
        },
        communication: PersonaCommunicationReadModel {
            primary_email: person.email_address,
        },
        created_at: person.created_at,
        updated_at: person.updated_at,
    }
}

pub(super) fn persona_read_model_from_api(
    person: makosh_personas_api::PersonaRead,
) -> Result<PersonaReadModel, crate::domains::personas::api::errors::PersonaProjectionError> {
    let persona_type =
        crate::domains::personas::api::models::PersonaType::try_from(person.persona_type.as_str())?;
    Ok(PersonaReadModel {
        persona_id: person.persona_id,
        persona_type,
        is_self: person.is_self,
        is_address_book: person.is_address_book,
        identity: PersonaIdentityReadModel {
            display_name: person.display_name,
            email_address: person.email_address.clone(),
        },
        communication: PersonaCommunicationReadModel {
            primary_email: person.email_address,
        },
        created_at: person.created_at,
        updated_at: person.updated_at,
    })
}

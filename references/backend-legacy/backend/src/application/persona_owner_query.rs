use makosh_personas_api::{
    PersonaOwnerQuery, PersonaOwnerQueryError, PersonaOwnerQueryFuture, PersonaRead,
    PersonaUpdateCommand, PersonaWriteError, PersonaWriteFuture, PersonaWritePort,
};
use sqlx::PgPool;

use crate::domains::personas::api::store::PersonaProjectionStore;

#[derive(Clone)]
pub(crate) struct PostgresPersonaOwnerQuery {
    store: PersonaProjectionStore,
}

impl PostgresPersonaOwnerQuery {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self {
            store: PersonaProjectionStore::new(pool),
        }
    }
}

impl PersonaOwnerQuery for PostgresPersonaOwnerQuery {
    fn owner_language<'a>(&'a self) -> PersonaOwnerQueryFuture<'a> {
        Box::pin(async move {
            self.store
                .owner_language()
                .await
                .map_err(|error| PersonaOwnerQueryError(error.to_string()))
        })
    }
}

impl PersonaWritePort for PostgresPersonaOwnerQuery {
    fn update<'a>(&'a self, command: PersonaUpdateCommand) -> PersonaWriteFuture<'a> {
        Box::pin(async move {
            crate::domains::personas::command_service::PersonaCommandService::new(
                self.store.pool().clone(),
            )
            .update_persona_manual(
                &command.persona_id,
                command.display_name.as_deref(),
                command.assign_owner,
            )
            .await
            .map(persona_read)
            .map_err(|error| PersonaWriteError(error.to_string()))
        })
    }
}

fn persona_read(persona: crate::domains::personas::api::models::Persona) -> PersonaRead {
    PersonaRead {
        persona_id: persona.persona_id,
        display_name: persona.display_name,
        email_address: persona.email_address,
        persona_type: persona.persona_type.as_str().to_owned(),
        is_self: persona.is_self,
        is_address_book: persona.is_address_book,
        created_at: persona.created_at,
        updated_at: persona.updated_at,
    }
}

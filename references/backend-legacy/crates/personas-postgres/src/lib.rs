use chrono::{DateTime, Utc};
use makosh_personas_api::{
    PersonaIdentityProjection, PersonaIdentityProjectionListFuture, PersonaIdentityProjectionPort,
    PersonaListFuture, PersonaQueryError, PersonaRead, PersonaReadFuture, PersonaReadPort,
};
use sqlx::{PgPool, Row};

#[derive(Clone)]
pub struct PersonaPostgresReadQuery {
    pool: PgPool,
}

impl PersonaPostgresReadQuery {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl PersonaReadPort for PersonaPostgresReadQuery {
    fn list<'a>(&'a self, limit: i64) -> PersonaListFuture<'a> {
        Box::pin(async move {
            sqlx::query(SELECT_SQL)
                .bind(limit.clamp(1, 100))
                .fetch_all(&self.pool)
                .await
                .map_err(query_error)?
                .into_iter()
                .map(to_read)
                .collect()
        })
    }

    fn get<'a>(&'a self, persona_id: &'a str) -> PersonaReadFuture<'a> {
        Box::pin(async move {
            sqlx::query(GET_SQL)
                .bind(persona_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(query_error)?
                .map(to_read)
                .transpose()
        })
    }
}

impl PersonaIdentityProjectionPort for PersonaPostgresReadQuery {
    fn list_for_values<'a>(
        &'a self,
        identity_type: &'a str,
        identity_values: &'a [String],
    ) -> PersonaIdentityProjectionListFuture<'a> {
        Box::pin(async move {
            sqlx::query(
                "SELECT identity_type, identity_value, metadata FROM persona_identities WHERE source = 'communication_projection' AND status = 'active' AND identity_type = $1 AND identity_value = ANY($2)",
            )
            .bind(identity_type)
            .bind(identity_values)
            .fetch_all(&self.pool)
            .await
            .map_err(query_error)?
            .into_iter()
            .map(|row| {
                Ok(PersonaIdentityProjection {
                    identity_type: row.try_get("identity_type").map_err(query_error)?,
                    identity_value: row.try_get("identity_value").map_err(query_error)?,
                    metadata: row.try_get("metadata").map_err(query_error)?,
                })
            })
            .collect()
        })
    }
}

const SELECT_SQL: &str = "SELECT persona_id, display_name, email_address, person_type, is_self, is_address_book, created_at, updated_at FROM personas ORDER BY updated_at DESC, created_at DESC, persona_id LIMIT $1";
const GET_SQL: &str = "SELECT persona_id, display_name, email_address, person_type, is_self, is_address_book, created_at, updated_at FROM personas WHERE persona_id = $1";

fn to_read(row: sqlx::postgres::PgRow) -> Result<PersonaRead, PersonaQueryError> {
    Ok(PersonaRead {
        persona_id: row.try_get("persona_id").map_err(query_error)?,
        display_name: row.try_get("display_name").map_err(query_error)?,
        email_address: row.try_get("email_address").map_err(query_error)?,
        persona_type: row.try_get("person_type").map_err(query_error)?,
        is_self: row.try_get("is_self").map_err(query_error)?,
        is_address_book: row.try_get("is_address_book").map_err(query_error)?,
        created_at: row
            .try_get::<DateTime<Utc>, _>("created_at")
            .map_err(query_error)?,
        updated_at: row
            .try_get::<DateTime<Utc>, _>("updated_at")
            .map_err(query_error)?,
    })
}

fn query_error(error: sqlx::Error) -> PersonaQueryError {
    PersonaQueryError(error.to_string())
}

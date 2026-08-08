use chrono::Utc;
use serde_json::json;
use sqlx::{Postgres, Transaction};

use super::super::constants::AI_EMBEDDING_DIMENSION;
use super::super::errors::AiError;
use super::super::evidence::link_ai_entity_in_transaction;
use super::super::helpers::{
    content_hash, halfvec_literal, semantic_embedding_id, validate_non_empty,
};
use super::models::{NewSemanticEmbedding, SemanticEmbedding, SemanticSourceKind};
use super::rows::row_to_semantic_embedding;
use super::store::SemanticEmbeddingStore;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;

impl SemanticEmbeddingStore {
    pub async fn upsert_embedding(
        &self,
        embedding: NewSemanticEmbedding<'_>,
    ) -> Result<SemanticEmbedding, AiError> {
        let source_id = validate_non_empty("source_id", embedding.source_id)?;
        let title = validate_non_empty("title", embedding.title)?;
        let source_text = validate_non_empty("source_text", embedding.source_text)?;
        let embedding_model = validate_non_empty("embedding_model", embedding.embedding_model)?;
        let observation_id = embedding
            .observation_id
            .map(|value| validate_non_empty("observation_id", value))
            .transpose()?;
        if embedding.source_kind == SemanticSourceKind::Message && observation_id.is_none() {
            return Err(AiError::InvalidRequest(
                "message semantic embeddings require observation_id",
            ));
        }
        let content_hash = content_hash(&source_text);
        let vector_literal = halfvec_literal(embedding.embedding)?;
        let semantic_embedding_id =
            semantic_embedding_id(embedding.source_kind.as_str(), &source_id, &embedding_model);

        let mut transaction = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO semantic_embeddings (
                semantic_embedding_id,
                source_kind,
                source_id,
                observation_id,
                title,
                source_text,
                content_hash,
                embedding_model,
                embedding_dimension,
                embedding,
                graph_node_id,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::halfvec, $11, now())
            ON CONFLICT (source_kind, source_id, embedding_model)
            DO UPDATE SET
                observation_id = COALESCE(EXCLUDED.observation_id, semantic_embeddings.observation_id),
                title = EXCLUDED.title,
                source_text = EXCLUDED.source_text,
                content_hash = EXCLUDED.content_hash,
                embedding_dimension = EXCLUDED.embedding_dimension,
                embedding = EXCLUDED.embedding,
                graph_node_id = EXCLUDED.graph_node_id,
                updated_at = now()
            RETURNING
                semantic_embedding_id,
                source_kind,
                source_id,
                observation_id,
                title,
                source_text,
                content_hash,
                embedding_model,
                embedding_dimension,
                graph_node_id,
                created_at,
                updated_at
            "#,
        )
        .bind(semantic_embedding_id)
        .bind(embedding.source_kind.as_str())
        .bind(&source_id)
        .bind(observation_id.as_deref())
        .bind(&title)
        .bind(&source_text)
        .bind(&content_hash)
        .bind(&embedding_model)
        .bind(AI_EMBEDDING_DIMENSION as i32)
        .bind(vector_literal)
        .bind(embedding.graph_node_id)
        .fetch_one(&mut *transaction)
        .await?;

        let stored = row_to_semantic_embedding(row)?;
        capture_semantic_embedding_observation(
            &mut transaction,
            &stored,
            embedding.source_kind,
            "upsert",
            "ai.core.semantic.upsert_embedding",
        )
        .await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub(super) async fn is_current(
        &self,
        source_kind: SemanticSourceKind,
        source_id: &str,
        embedding_model: &str,
        content_hash: &str,
    ) -> Result<bool, AiError> {
        let current_hash = sqlx::query_scalar::<_, String>(
            r#"
            SELECT content_hash
            FROM semantic_embeddings
            WHERE source_kind = $1
              AND source_id = $2
              AND embedding_model = $3
            "#,
        )
        .bind(source_kind.as_str())
        .bind(source_id)
        .bind(embedding_model)
        .fetch_optional(&self.pool)
        .await?;

        Ok(current_hash.as_deref() == Some(content_hash))
    }
}

async fn capture_semantic_embedding_observation(
    transaction: &mut Transaction<'_, Postgres>,
    embedding: &SemanticEmbedding,
    source_kind: SemanticSourceKind,
    relationship_kind: &str,
    actor: &str,
) -> Result<(), AiError> {
    let observation = ObservationStore::capture_in_transaction(
        transaction,
        &NewObservation::new(
            "AI_SEMANTIC_EMBEDDING",
            ObservationOriginKind::LocalRuntime,
            Utc::now(),
            json!({
                "semantic_embedding_id": embedding.semantic_embedding_id,
                "source_kind": embedding.source_kind,
                "source_kind_canonical": source_kind.as_str(),
                "source_id": embedding.source_id,
                "observation_id": embedding.observation_id,
                "title": embedding.title,
                "source_text": embedding.source_text,
                "content_hash": embedding.content_hash,
                "embedding_model": embedding.embedding_model,
                "embedding_dimension": embedding.embedding_dimension,
                "graph_node_id": embedding.graph_node_id,
                "operation": relationship_kind,
            }),
            format!(
                "ai-semantic-embedding://{}/{}",
                embedding.semantic_embedding_id, relationship_kind
            ),
        )
        .provenance(json!({
            "captured_by": actor,
            "operation": relationship_kind,
        })),
    )
    .await?;
    link_ai_entity_in_transaction(
        transaction,
        &observation.observation_id,
        "semantic_embedding",
        embedding.semantic_embedding_id.clone(),
        relationship_kind,
        json!({
            "source_kind": embedding.source_kind,
            "source_id": embedding.source_id,
            "embedding_model": embedding.embedding_model,
        }),
    )
    .await?;
    Ok(())
}

use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};

use super::errors::{DocumentImportError, DocumentImportWithProcessingError};
use super::evidence::link_document_entity_in_transaction;
use super::models::{ImportedDocument, ImportedDocumentWithProcessing, NewDocumentImport};
use super::rows::row_to_imported_document;
use crate::domains::documents::processing::store::DocumentProcessingStore;
use chrono::Utc;
use makosh_observations_api::models::{NewObservation, ObservationOriginKind};
use makosh_observations_postgres::store::ObservationStore;

#[derive(Clone)]
pub struct DocumentImportStore {
    pool: PgPool,
}

impl DocumentImportStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn import_document_and_enqueue_processing(
        &self,
        document: &NewDocumentImport,
        processing_store: &DocumentProcessingStore,
    ) -> Result<ImportedDocumentWithProcessing, DocumentImportWithProcessingError> {
        let imported = self.import_document(document).await?;
        let jobs = processing_store
            .enqueue_for_document(&imported.document_id)
            .await?;

        Ok(ImportedDocumentWithProcessing { imported, jobs })
    }

    pub async fn import_document(
        &self,
        document: &NewDocumentImport,
    ) -> Result<ImportedDocument, DocumentImportError> {
        document.validate()?;
        let mut transaction = self.pool.begin().await?;
        let imported = Self::import_document_in_transaction(&mut transaction, document).await?;
        transaction.commit().await?;
        Ok(imported)
    }

    pub(crate) async fn import_document_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        document: &NewDocumentImport,
    ) -> Result<ImportedDocument, DocumentImportError> {
        Self::import_document_with_origin_in_transaction(
            transaction,
            document,
            ObservationOriginKind::FileImport,
            format!("document://{}", document.document_id),
            json!({
                "ingested_by": "documents_domain"
            }),
            None,
            None,
            None,
        )
        .await
    }

    pub(crate) async fn import_document_manual_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        document: &NewDocumentImport,
        source_ref: String,
        provenance: serde_json::Value,
    ) -> Result<ImportedDocument, DocumentImportError> {
        Self::import_document_manual_with_observation_in_transaction(
            transaction,
            document,
            source_ref,
            provenance,
            None,
            None,
            None,
        )
        .await
    }

    pub(crate) async fn import_document_manual_with_observation_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        document: &NewDocumentImport,
        source_ref: String,
        provenance: serde_json::Value,
        source_observation_id: Option<&str>,
        relationship_kind: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<ImportedDocument, DocumentImportError> {
        Self::import_document_with_origin_in_transaction(
            transaction,
            document,
            ObservationOriginKind::Manual,
            source_ref,
            provenance,
            source_observation_id,
            relationship_kind,
            metadata,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn import_document_with_origin_in_transaction(
        transaction: &mut Transaction<'_, Postgres>,
        document: &NewDocumentImport,
        origin_kind: ObservationOriginKind,
        source_ref: String,
        provenance: serde_json::Value,
        source_observation_id: Option<&str>,
        relationship_kind: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> Result<ImportedDocument, DocumentImportError> {
        let document = document.validate()?;
        let observation = NewObservation::new(
            "DOCUMENT",
            origin_kind,
            Utc::now(),
            json!({
                "document_id": document.document_id,
                "document_kind": document.document_kind,
                "title": document.title,
                "source_fingerprint": document.source_fingerprint,
                "extracted_text": document.extracted_text,
            }),
            source_ref,
        )
        .provenance(provenance);

        let observation = ObservationStore::capture_in_transaction(transaction, &observation)
            .await
            .map_err(DocumentImportError::from)?;

        let row = sqlx::query(
            r#"
            INSERT INTO documents (
                document_id,
                document_kind,
                observation_id,
                title,
                source_fingerprint,
                extracted_text
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (document_id)
            DO UPDATE SET
                observation_id = EXCLUDED.observation_id,
                title = EXCLUDED.title,
                source_fingerprint = EXCLUDED.source_fingerprint,
                extracted_text = EXCLUDED.extracted_text
            WHERE documents.document_kind = EXCLUDED.document_kind
            RETURNING
                document_id,
                document_kind,
                observation_id,
                title,
                source_fingerprint,
                extracted_text,
                imported_at
            "#,
        )
        .bind(&document.document_id)
        .bind(&document.document_kind)
        .bind(&observation.observation_id)
        .bind(&document.title)
        .bind(&document.source_fingerprint)
        .bind(&document.extracted_text)
        .fetch_optional(&mut **transaction)
        .await?;

        if let Some(row) = row {
            let imported = row_to_imported_document(row)?;
            link_document_entity_in_transaction(
                transaction,
                &imported.observation_id,
                imported.document_id.clone(),
                Some("import"),
                Some(json!({
                    "document_kind": imported.document_kind,
                    "source_fingerprint": imported.source_fingerprint,
                })),
            )
            .await
            .map_err(DocumentImportError::from)?;
            if let Some(source_observation_id) =
                source_observation_id.filter(|value| !value.trim().is_empty())
            {
                let metadata = match metadata {
                    Some(extra)
                        if json!({
                            "document_kind": imported.document_kind,
                            "source_document_observation_id": imported.observation_id,
                        })
                        .is_object()
                            && extra.is_object() =>
                    {
                        let mut merged = json!({
                            "document_kind": imported.document_kind,
                            "source_document_observation_id": imported.observation_id,
                        });
                        if let (Some(base), Some(extra)) =
                            (merged.as_object_mut(), extra.as_object())
                        {
                            for (key, value) in extra {
                                base.insert(key.clone(), value.clone());
                            }
                        }
                        merged
                    }
                    Some(extra) => extra,
                    None => json!({
                        "document_kind": imported.document_kind,
                        "source_document_observation_id": imported.observation_id,
                    }),
                };
                link_document_entity_in_transaction(
                    transaction,
                    source_observation_id,
                    imported.document_id.clone(),
                    Some(
                        relationship_kind
                            .filter(|value| !value.trim().is_empty())
                            .unwrap_or("workflow_action_projection"),
                    ),
                    Some(metadata),
                )
                .await
                .map_err(DocumentImportError::from)?;
            }
            return Ok(imported);
        }

        let existing_kind = sqlx::query_scalar::<_, String>(
            "SELECT document_kind FROM documents WHERE document_id = $1",
        )
        .bind(&document.document_id)
        .fetch_optional(&mut **transaction)
        .await?;

        match existing_kind {
            Some(existing_kind) => Err(DocumentImportError::DocumentKindChange {
                document_id: document.document_id,
                existing_kind,
                new_kind: document.document_kind,
            }),
            None => Err(DocumentImportError::UpsertSkipped(document.document_id)),
        }
    }
}

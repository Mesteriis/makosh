use makosh_hub_backend::domains::documents::core::errors::DocumentImportError;
use makosh_hub_backend::domains::documents::core::models::{ImportedDocument, NewDocumentImport};
use makosh_hub_backend::domains::documents::core::store::DocumentImportStore;
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPool;
use uuid::Uuid;

pub struct DocumentFactory<'a> {
    pool: &'a PgPool,
    title: String,
    doc_kind: String,
    text: String,
}

impl<'a> DocumentFactory<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            pool,
            title: "Test Document".into(),
            doc_kind: "markdown".into(),
            text: "# Test Document\n\nAuto-generated for integration testing.".into(),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_kind(mut self, kind: impl Into<String>) -> Self {
        self.doc_kind = kind.into();
        self
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    pub async fn create(self) -> Result<ImportedDocument, DocumentImportError> {
        let store = DocumentImportStore::new(self.pool.clone());
        let fingerprint = format!("{:x}", Sha256::digest(self.text.as_bytes()));
        let new_doc = NewDocumentImport {
            document_id: format!("doc:{}", Uuid::new_v4()),
            document_kind: self.doc_kind,
            title: self.title,
            source_fingerprint: fingerprint,
            extracted_text: self.text,
        };
        store.import_document(&new_doc).await
    }
}

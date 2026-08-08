use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

use makosh_hub_backend::domains::documents::core::errors::DocumentImportError;
use makosh_hub_backend::domains::documents::core::models::NewDocumentImport;
use makosh_hub_backend::domains::documents::core::store::DocumentImportStore;
use makosh_hub_backend::platform::storage::database::Database;

#[tokio::test]
async fn document_import_stores_markdown_text_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = DocumentImportStore::new(database.pool().expect("configured pool").clone());
    let suffix = unique_suffix();

    let imported = store
        .import_document(&NewDocumentImport::markdown(
            format!("doc_markdown_{suffix}"),
            "notes.md",
            "# Notes\n\nBudget review notes.",
        ))
        .await
        .expect("import markdown");

    assert_eq!(imported.document_kind, "markdown");
    assert_eq!(imported.title, "notes.md");
    assert_eq!(imported.extracted_text, "Notes\n\nBudget review notes.");

    let observation_link_count: i64 = sqlx::query_scalar(
        r#"
        SELECT count(*)::BIGINT
        FROM observation_links
        WHERE observation_id = $1
          AND domain = 'documents'
          AND entity_kind = 'document'
          AND entity_id = $2
          AND relationship_kind = 'import'
        "#,
    )
    .bind(&imported.observation_id)
    .bind(&imported.document_id)
    .fetch_one(database.pool().expect("configured pool"))
    .await
    .expect("document import observation links");
    assert_eq!(observation_link_count, 1);
}

#[tokio::test]
async fn document_import_stores_pdf_metadata_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let store = DocumentImportStore::new(database.pool().expect("configured pool").clone());
    let suffix = unique_suffix();

    let imported = store
        .import_document(&NewDocumentImport::pdf_metadata(
            format!("doc_pdf_{suffix}"),
            "contract.pdf",
            "sha256:contract",
        ))
        .await
        .expect("import pdf metadata");

    assert_eq!(imported.document_kind, "pdf");
    assert_eq!(imported.title, "contract.pdf");
    assert_eq!(imported.extracted_text, "");
    assert_eq!(imported.source_fingerprint, "sha256:contract");
}

#[tokio::test]
async fn document_import_rejects_blank_required_fields() {
    let store = disconnected_document_store();

    for (field_name, document) in [
        (
            "document_id",
            NewDocumentImport {
                document_id: "   ".to_owned(),
                document_kind: "markdown".to_owned(),
                title: "notes.md".to_owned(),
                source_fingerprint: "sha256:notes".to_owned(),
                extracted_text: "Notes".to_owned(),
            },
        ),
        (
            "document_kind",
            NewDocumentImport {
                document_id: "doc_blank_kind".to_owned(),
                document_kind: "   ".to_owned(),
                title: "notes.md".to_owned(),
                source_fingerprint: "sha256:notes".to_owned(),
                extracted_text: "Notes".to_owned(),
            },
        ),
        (
            "title",
            NewDocumentImport {
                document_id: "doc_blank_title".to_owned(),
                document_kind: "markdown".to_owned(),
                title: "   ".to_owned(),
                source_fingerprint: "sha256:notes".to_owned(),
                extracted_text: "Notes".to_owned(),
            },
        ),
        (
            "source_fingerprint",
            NewDocumentImport {
                document_id: "doc_blank_fingerprint".to_owned(),
                document_kind: "pdf".to_owned(),
                title: "contract.pdf".to_owned(),
                source_fingerprint: "   ".to_owned(),
                extracted_text: String::new(),
            },
        ),
        (
            "extracted_text",
            NewDocumentImport {
                document_id: "doc_blank_extracted_text".to_owned(),
                document_kind: "markdown".to_owned(),
                title: "notes.md".to_owned(),
                source_fingerprint: "sha256:notes".to_owned(),
                extracted_text: "   ".to_owned(),
            },
        ),
    ] {
        let error = store
            .import_document(&document)
            .await
            .expect_err("blank document field must fail");

        assert!(
            matches!(error, DocumentImportError::EmptyField(actual) if actual == field_name),
            "expected EmptyField({field_name}), got {error:?}"
        );
    }
}

#[tokio::test]
async fn document_import_rejects_invalid_kind() {
    let store = disconnected_document_store();
    let document = NewDocumentImport {
        document_id: "doc_invalid_kind".to_owned(),
        document_kind: "docx".to_owned(),
        title: "notes.docx".to_owned(),
        source_fingerprint: "sha256:notes".to_owned(),
        extracted_text: "Notes".to_owned(),
    };

    let error = store
        .import_document(&document)
        .await
        .expect_err("invalid document kind must fail");

    assert!(
        matches!(error, DocumentImportError::InvalidDocumentKind(ref kind) if kind == "docx"),
        "expected InvalidDocumentKind(docx), got {error:?}"
    );
}

#[test]
fn markdown_import_helper_derives_deterministic_local_fingerprint() {
    let first = NewDocumentImport::markdown("doc_fingerprint", "notes.md", "# Notes\n\nBody.");
    let second = NewDocumentImport::markdown("doc_fingerprint", "notes.md", "# Notes\n\nBody.");

    assert_eq!(first.source_fingerprint, second.source_fingerprint);
    assert_eq!(first.extracted_text, "Notes\n\nBody.");
    assert!(first.source_fingerprint.starts_with("local-v1:markdown:"));
}

#[tokio::test]
async fn document_import_extracts_multiple_markdown_heading_levels_against_postgres() {
    let Some(store) = live_document_store("markdown heading extraction").await else {
        return;
    };
    let suffix = unique_suffix();

    let imported = store
        .import_document(&NewDocumentImport::markdown(
            format!("doc_headings_{suffix}"),
            "headings.md",
            "# Title\n\n## Section\n\n### Detail\n\nBody.   \n",
        ))
        .await
        .expect("import markdown headings");

    assert_eq!(
        imported.extracted_text,
        "Title\n\nSection\n\nDetail\n\nBody."
    );
    assert!(
        imported
            .source_fingerprint
            .starts_with("local-v1:markdown:")
    );
}

#[tokio::test]
async fn document_import_preserves_hash_prefixed_non_headings_against_postgres() {
    let Some(store) = live_document_store("markdown non-heading hash-prefixed lines").await else {
        return;
    };
    let suffix = unique_suffix();

    let imported = store
        .import_document(&NewDocumentImport::markdown(
            format!("doc_hash_prefixed_{suffix}"),
            "hash-prefixed.md",
            "# Heading\n\n#hashtag\n#include <x>\n###not heading\n####### Too many hashes",
        ))
        .await
        .expect("import markdown hash-prefixed lines");

    assert_eq!(
        imported.extracted_text,
        "Heading\n\n#hashtag\n#include <x>\n###not heading\n####### Too many hashes"
    );
}

#[tokio::test]
async fn document_import_reimports_same_kind_idempotently_against_postgres() {
    let Some((pool, store)) = live_document_context("same-kind idempotent document import").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let document_id = format!("doc_idempotent_{suffix}");

    let first = store
        .import_document(&NewDocumentImport::markdown(
            document_id.clone(),
            "draft.md",
            "# Draft\n\nInitial text.",
        ))
        .await
        .expect("first import");
    let updated = store
        .import_document(&NewDocumentImport::markdown(
            document_id.clone(),
            "draft-renamed.md",
            "# Draft\n\nUpdated text.",
        ))
        .await
        .expect("second import");

    assert_eq!(updated.document_id, document_id);
    assert_eq!(updated.document_kind, "markdown");
    assert_eq!(updated.title, "draft-renamed.md");
    assert_ne!(updated.source_fingerprint, first.source_fingerprint);
    assert_eq!(updated.extracted_text, "Draft\n\nUpdated text.");
    assert_eq!(updated.imported_at, first.imported_at);

    let count =
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM documents WHERE document_id = $1")
            .bind(&updated.document_id)
            .fetch_one(&pool)
            .await
            .expect("idempotent document count");
    assert_eq!(count, 1);
}

#[tokio::test]
async fn document_import_rejects_existing_document_kind_change_against_postgres() {
    let Some((pool, store)) = live_document_context("document kind change rejection").await else {
        return;
    };
    let suffix = unique_suffix();
    let document_id = format!("doc_kind_change_{suffix}");

    let first = store
        .import_document(&NewDocumentImport::markdown(
            document_id.clone(),
            "notes.md",
            "# Notes\n\nInitial text.",
        ))
        .await
        .expect("first import");

    let error = store
        .import_document(&NewDocumentImport::pdf_metadata(
            document_id.clone(),
            "notes.pdf",
            "sha256:notes-pdf",
        ))
        .await
        .expect_err("document kind changes must fail");

    assert!(
        matches!(
            error,
            DocumentImportError::DocumentKindChange {
                ref document_id,
                ref existing_kind,
                ref new_kind,
            } if document_id == &first.document_id
                && existing_kind == "markdown"
                && new_kind == "pdf"
        ),
        "expected DocumentKindChange, got {error:?}"
    );

    let stored = sqlx::query_as::<_, (String, String, String, String)>(
        r#"
        SELECT document_kind, title, source_fingerprint, extracted_text
        FROM documents
        WHERE document_id = $1
        "#,
    )
    .bind(&document_id)
    .fetch_one(&pool)
    .await
    .expect("stored document after rejected kind change");

    assert_eq!(stored.0, "markdown");
    assert_eq!(stored.1, "notes.md");
    assert_eq!(stored.2, first.source_fingerprint);
    assert_eq!(stored.3, "Notes\n\nInitial text.");
}

async fn live_document_context(
    _test_name: &str,
) -> Option<(sqlx::postgres::PgPool, DocumentImportStore)> {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();

    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();

    Some((pool.clone(), DocumentImportStore::new(pool)))
}

async fn live_document_store(test_name: &str) -> Option<DocumentImportStore> {
    live_document_context(test_name)
        .await
        .map(|(_, store)| store)
}

fn disconnected_document_store() -> DocumentImportStore {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://makosh:unused@127.0.0.1:1/makosh_hub")
        .expect("create lazy test pool");
    DocumentImportStore::new(pool)
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos()
}

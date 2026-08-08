use makosh_hub_backend::engines::search::{
    engine::SearchIndex,
    errors::SearchError,
    models::{SearchDocument, SearchResult},
};

#[test]
fn search_index_returns_message_by_body_term() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let index = SearchIndex::open_or_create(temp_dir.path()).expect("open search index");

    index
        .upsert_document(&SearchDocument {
            object_id: "message-1".to_owned(),
            object_kind: "message".to_owned(),
            title: "Budget review".to_owned(),
            body: "Please review the Q2 budget before Monday.".to_owned(),
        })
        .expect("index document");
    index.commit().expect("commit index");

    let results = index.search("budget", 10).expect("search");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].object_id, "message-1");
    assert_eq!(results[0].object_kind, "message");
}

#[test]
fn search_index_rejects_blank_required_document_fields() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let index = SearchIndex::open_or_create(temp_dir.path()).expect("open search index");

    for (field_name, document) in [
        (
            "object_id",
            SearchDocument {
                object_id: " ".to_owned(),
                object_kind: "message".to_owned(),
                title: "Budget review".to_owned(),
                body: "Please review the Q2 budget before Monday.".to_owned(),
            },
        ),
        (
            "object_kind",
            SearchDocument {
                object_id: "message-1".to_owned(),
                object_kind: " ".to_owned(),
                title: "Budget review".to_owned(),
                body: "Please review the Q2 budget before Monday.".to_owned(),
            },
        ),
        (
            "title",
            SearchDocument {
                object_id: "message-1".to_owned(),
                object_kind: "message".to_owned(),
                title: " ".to_owned(),
                body: "Please review the Q2 budget before Monday.".to_owned(),
            },
        ),
    ] {
        let error = index
            .upsert_document(&document)
            .expect_err("blank document field must fail");

        assert!(matches!(error, SearchError::EmptyField(actual) if actual == field_name));
    }
}

#[test]
fn search_index_rejects_blank_query() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let index = SearchIndex::open_or_create(temp_dir.path()).expect("open search index");

    let error = index.search(" \t", 10).expect_err("blank query must fail");

    assert!(matches!(error, SearchError::EmptyField("query")));
}

#[test]
fn search_index_rejects_zero_limit() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let index = SearchIndex::open_or_create(temp_dir.path()).expect("open search index");

    let error = index.search("budget", 0).expect_err("zero limit must fail");

    assert!(matches!(error, SearchError::InvalidLimit));
}

#[test]
fn search_index_replaces_existing_document_identity() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let index = SearchIndex::open_or_create(temp_dir.path()).expect("open search index");

    index
        .upsert_document(&SearchDocument {
            object_id: "message-1".to_owned(),
            object_kind: "message".to_owned(),
            title: "Budget review".to_owned(),
            body: "Please review the Q2 budget before Monday.".to_owned(),
        })
        .expect("index first document");
    index
        .upsert_document(&SearchDocument {
            object_id: "message-1".to_owned(),
            object_kind: "message".to_owned(),
            title: "Roadmap review".to_owned(),
            body: "Please review the implementation roadmap before Monday.".to_owned(),
        })
        .expect("replace document");
    index.commit().expect("commit index");

    let old_results = index.search("budget", 10).expect("search old term");
    let new_results = index.search("roadmap", 10).expect("search new term");

    assert_eq!(old_results, Vec::new());
    assert_eq!(
        new_results,
        vec![SearchResult {
            object_id: "message-1".to_owned(),
            object_kind: "message".to_owned(),
            title: "Roadmap review".to_owned(),
        }]
    );
}

#[test]
fn search_index_replaces_committed_document_identity() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let index = SearchIndex::open_or_create(temp_dir.path()).expect("open search index");

    index
        .upsert_document(&SearchDocument {
            object_id: "message-1".to_owned(),
            object_kind: "message".to_owned(),
            title: "Budget review".to_owned(),
            body: "Please review the Q2 budget before Monday.".to_owned(),
        })
        .expect("index first document");
    index.commit().expect("commit first version");

    let old_results = index.search("budget", 10).expect("search old term");
    assert_eq!(old_results.len(), 1);

    index
        .upsert_document(&SearchDocument {
            object_id: "message-1".to_owned(),
            object_kind: "message".to_owned(),
            title: "Roadmap review".to_owned(),
            body: "Please review the implementation roadmap before Monday.".to_owned(),
        })
        .expect("replace committed document");
    index.commit().expect("commit second version");

    let old_results = index.search("budget", 10).expect("search old term");
    let new_results = index.search("roadmap", 10).expect("search new term");

    assert_eq!(old_results, Vec::new());
    assert_eq!(
        new_results,
        vec![SearchResult {
            object_id: "message-1".to_owned(),
            object_kind: "message".to_owned(),
            title: "Roadmap review".to_owned(),
        }]
    );
}

#[test]
fn search_index_accepts_blank_body_for_title_only_documents() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let index = SearchIndex::open_or_create(temp_dir.path()).expect("open search index");

    index
        .upsert_document(&SearchDocument {
            object_id: "document-1".to_owned(),
            object_kind: "document".to_owned(),
            title: "PDF metadata overview".to_owned(),
            body: " ".to_owned(),
        })
        .expect("index title-only document");
    index.commit().expect("commit index");

    let results = index.search("metadata", 10).expect("search title term");

    assert_eq!(
        results,
        vec![SearchResult {
            object_id: "document-1".to_owned(),
            object_kind: "document".to_owned(),
            title: "PDF metadata overview".to_owned(),
        }]
    );
}

#[test]
fn search_index_distinguishes_delimiter_bearing_document_identities() {
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let index = SearchIndex::open_or_create(temp_dir.path()).expect("open search index");

    index
        .upsert_document(&SearchDocument {
            object_id: "c".to_owned(),
            object_kind: "a:b".to_owned(),
            title: "First identity".to_owned(),
            body: "sharedterm".to_owned(),
        })
        .expect("index first document");
    index
        .upsert_document(&SearchDocument {
            object_id: "b:c".to_owned(),
            object_kind: "a".to_owned(),
            title: "Second identity".to_owned(),
            body: "sharedterm".to_owned(),
        })
        .expect("index second document");
    index.commit().expect("commit index");

    let results = index.search("sharedterm", 10).expect("search");

    assert_eq!(results.len(), 2);
}

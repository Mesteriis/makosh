use makosh_backend_testkit::context::TestContext;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use makosh_backend_testkit::factories::persona::PersonaFactory;
use serde_json::{Value, json};
use sqlx::Row;
use tower::ServiceExt;

use makosh_hub_backend::app::router::{build_router, build_router_with_database};
use makosh_hub_backend::domains::organizations::enrichment::OrgEnrichmentStore;
use makosh_hub_backend::platform::config::app_config::AppConfig;
use makosh_hub_backend::platform::storage::database::Database;

const T: &str = "orgs-test-token";

fn cfg() -> AppConfig {
    makosh_backend_testkit::app::config_with_secret(T)
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .header("x-makosh-secret", T)
        .body(Body::empty())
        .expect("req")
}
fn post(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", T)
        .body(Body::from(body.to_string()))
        .expect("req")
}
fn put(uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(Method::PUT)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-makosh-secret", T)
        .body(Body::from(body.to_string()))
        .expect("req")
}

async fn jb(r: axum::response::Response) -> Value {
    let b = to_bytes(r.into_body(), usize::MAX).await.expect("b");
    serde_json::from_slice(&b).expect("json")
}

fn uid() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("t")
        .as_nanos()
}

fn enc(v: &str) -> String {
    url::form_urlencoded::byte_serialize(v.as_bytes()).collect()
}

async fn router(db: &str) -> axum::Router {
    let database = Database::connect(Some(db)).await.expect("db");
    build_router_with_database(
        makosh_backend_testkit::app::config_with_secret_and_database_url(T, db),
        database,
    )
}

// Helper: create org, return org_id
async fn mkorg(app: &axum::Router, s: u128) -> String {
    let r = app
        .clone()
        .oneshot(post(
            "/api/v1/organizations",
            json!({"display_name": format!("T{s}"), "org_type": "technology"}),
        ))
        .await
        .expect("r");
    if r.status().is_success() {
        jb(r).await["organization_id"]
            .as_str()
            .unwrap_or("org:unknown")
            .to_owned()
    } else {
        format!("org:bad:{s}")
    }
}

// ── Auth ───────────────────────────────────────────────────────────────────
#[tokio::test]
async fn orgs_auth_reject() {
    let r = build_router(cfg());
    let resp = r
        .oneshot(
            Request::builder()
                .uri("/api/v1/organizations")
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("r");
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ── CRUD ───────────────────────────────────────────────────────────────────
#[tokio::test]
async fn orgs_crud() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let s = uid();
    let a = router(&db).await;

    // Create
    let r = a
        .clone()
        .oneshot(post(
            "/api/v1/organizations",
            json!({"display_name": format!("A{s}"), "org_type": "technology"}),
        ))
        .await
        .expect("r");
    assert!(r.status().is_success(), "create={}", r.status());
    let oid = jb(r).await["organization_id"].as_str().unwrap().to_owned();

    // Get
    let r = a
        .clone()
        .oneshot(get(&format!("/api/v1/organizations/{}", enc(&oid))))
        .await
        .expect("r");
    assert!(r.status().is_success(), "get={}", r.status());

    // Update
    let r = a
        .clone()
        .oneshot(put(
            &format!("/api/v1/organizations/{}", enc(&oid)),
            json!({"display_name": format!("U{s}")}),
        ))
        .await
        .expect("r");
    assert!(!r.status().is_server_error(), "update={}", r.status());

    // Archive
    let r = a
        .oneshot(post(
            &format!("/api/v1/organizations/{}/archive", enc(&oid)),
            json!({}),
        ))
        .await
        .expect("r");
    assert!(r.status().is_success(), "archive={}", r.status());
}

#[tokio::test]
async fn orgs_list() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let s = uid();
    let a = router(&db).await;
    mkorg(&a, s).await;
    let r = a.oneshot(get("/api/v1/organizations")).await.expect("r");
    assert!(r.status().is_success(), "list={}", r.status());
}

#[tokio::test]
async fn orgs_search() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let a = router(&db).await;
    let r = a
        .oneshot(get("/api/v1/organizations/search?q=test"))
        .await
        .expect("r");
    assert!(r.status().is_success(), "search={}", r.status());
}

#[tokio::test]
async fn orgs_not_found_404() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let s = uid();
    let a = router(&db).await;
    let r = a
        .oneshot(get(&format!("/api/v1/organizations/org:nx{s}")))
        .await
        .expect("r");
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
}

// ── Sub-resource read endpoints ────────────────────────────────────────────
macro_rules! org_test {
    ($name:ident, $path:expr) => {
        #[tokio::test]
        async fn $name() {
            let test_context = TestContext::new().await;
            let db = test_context.connection_string();
            let s = uid();
            let a = router(&db).await;
            let oid = mkorg(&a, s).await;
            let r = a.oneshot(get(&format!($path, enc(&oid)))).await.expect("r");
            assert!(
                !r.status().is_server_error(),
                "{} status={}",
                stringify!($name),
                r.status()
            );
        }
    };
}

org_test!(orgs_identities, "/api/v1/organizations/{}/identities");
org_test!(orgs_aliases, "/api/v1/organizations/{}/aliases");
org_test!(orgs_domains, "/api/v1/organizations/{}/domains");
org_test!(orgs_departments, "/api/v1/organizations/{}/departments");
org_test!(orgs_persona_links, "/api/v1/organizations/{}/persona-links");
org_test!(orgs_related, "/api/v1/organizations/{}/related");
org_test!(orgs_timeline, "/api/v1/organizations/{}/timeline");
org_test!(orgs_portals, "/api/v1/organizations/{}/portals");
org_test!(orgs_procedures, "/api/v1/organizations/{}/procedures");
org_test!(orgs_playbooks, "/api/v1/organizations/{}/playbooks");
org_test!(orgs_templates, "/api/v1/organizations/{}/templates");
org_test!(orgs_financial, "/api/v1/organizations/{}/financial");
org_test!(orgs_contracts, "/api/v1/organizations/{}/contracts");
org_test!(orgs_compliance, "/api/v1/organizations/{}/compliance");
org_test!(orgs_services, "/api/v1/organizations/{}/services");
org_test!(orgs_products, "/api/v1/organizations/{}/products");
org_test!(orgs_enrichment, "/api/v1/organizations/{}/enrichment");
org_test!(orgs_risks, "/api/v1/organizations/{}/risks");
org_test!(orgs_health, "/api/v1/organizations/{}/health");
org_test!(orgs_dossier, "/api/v1/organizations/{}/dossier");
org_test!(orgs_brief, "/api/v1/organizations/{}/brief");
org_test!(orgs_context_pack, "/api/v1/organizations/{}/context-pack");

#[tokio::test]
async fn orgs_enrichment_apply_captures_observation_against_postgres() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let s = uid();
    let a = router(&db).await;
    let oid = mkorg(&a, s).await;
    let database = Database::connect(Some(&db)).await.expect("db");
    let pool = database.pool().expect("configured pool").clone();
    let enrichment = OrgEnrichmentStore::new(pool.clone())
        .upsert(
            &oid,
            "crunchbase",
            json!({
                "fact": "Raised seed round"
            }),
            0.81,
        )
        .await
        .expect("create org enrichment result");

    let response = a
        .oneshot(post(
            &format!(
                "/api/v1/organizations/{}/enrichment/{}/apply",
                enc(&oid),
                enc(&enrichment.id)
            ),
            json!({}),
        ))
        .await
        .expect("response");
    assert!(
        response.status().is_success(),
        "apply={}",
        response.status()
    );

    let link_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM observation_links
         WHERE domain = 'organizations'
           AND entity_kind = 'organization_enrichment_result'
           AND entity_id = $1
           AND relationship_kind = 'review_transition'",
    )
    .bind(&enrichment.id)
    .fetch_one(&pool)
    .await
    .expect("organization enrichment observation link count");
    assert_eq!(link_count, 1);
}

// ── Identity / Alias / Department creation ─────────────────────────────────
macro_rules! org_post_test {
    ($name:ident, $path:expr, $body:expr) => {
        #[tokio::test]
        async fn $name() {
            let test_context = TestContext::new().await;
            let db = test_context.connection_string();
            let s = uid();
            let a = router(&db).await;
            let oid = mkorg(&a, s).await;
            let r = a
                .oneshot(post(&format!($path, enc(&oid)), $body))
                .await
                .expect("r");
            assert!(
                !r.status().is_server_error(),
                "{} status={}",
                stringify!($name),
                r.status()
            );
        }
    };
}

org_post_test!(
    orgs_post_identity,
    "/api/v1/organizations/{}/identities",
    json!({"identity_type": "email_domain", "identity_value": "ex.com", "source": "manual"})
);
org_post_test!(
    orgs_post_alias,
    "/api/v1/organizations/{}/aliases",
    json!({"name": "AliasCo", "alias_type": "former_name", "source": "manual"})
);
org_post_test!(
    orgs_post_department,
    "/api/v1/organizations/{}/departments",
    json!({"name": "Engineering", "description": "eng"})
);

// ── Watchlist toggle ───────────────────────────────────────────────────────
#[tokio::test]
async fn orgs_watchlist_toggle() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let s = uid();
    let a = router(&db).await;
    let oid = mkorg(&a, s).await;
    let r = a
        .oneshot(post(
            &format!("/api/v1/organizations/{}/watchlist", enc(&oid)),
            json!({}),
        ))
        .await
        .expect("r");
    assert!(!r.status().is_server_error(), "watchlist={}", r.status());
}

#[tokio::test]
async fn organization_manual_entrypoints_capture_observations_against_postgres() {
    let test_context = TestContext::new().await;
    let db = test_context.connection_string();
    let app = router(&db).await;
    let suffix = uid();
    let oid = mkorg(&app, suffix).await;
    let pool = Database::connect(Some(&db))
        .await
        .expect("db")
        .pool()
        .expect("pool")
        .clone();

    let create_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'organizations'
           AND entity_kind = 'organization'
           AND entity_id = $1
           AND metadata ->> 'action' = 'create'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&oid)
    .fetch_one(&pool)
    .await
    .expect("organization create observation link");

    let identity_response = app
        .clone()
        .oneshot(post(
            &format!("/api/v1/organizations/{}/identities", enc(&oid)),
            json!({
                "identity_type": "email_domain",
                "identity_value": format!("org-{suffix}.example.com"),
                "source": "manual"
            }),
        ))
        .await
        .expect("identity response");
    assert_eq!(identity_response.status(), StatusCode::OK);
    let identity_id = jb(identity_response).await["id"]
        .as_str()
        .expect("identity id")
        .to_owned();
    let identity_source: String =
        sqlx::query_scalar("SELECT source FROM organization_identities WHERE id::text = $1")
            .bind(&identity_id)
            .fetch_one(&pool)
            .await
            .expect("identity source");
    assert!(identity_source.starts_with("observation:"));
    let identity_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'organizations'
           AND entity_kind = 'identity'
           AND entity_id = $1
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&identity_id)
    .fetch_one(&pool)
    .await
    .expect("identity observation link");

    let alias_response = app
        .clone()
        .oneshot(post(
            &format!("/api/v1/organizations/{}/aliases", enc(&oid)),
            json!({
                "name": format!("Alias {suffix}"),
                "alias_type": "former_name",
                "source": "manual"
            }),
        ))
        .await
        .expect("alias response");
    assert_eq!(alias_response.status(), StatusCode::OK);
    let alias_id = jb(alias_response).await["id"]
        .as_str()
        .expect("alias id")
        .to_owned();
    let alias_source: String =
        sqlx::query_scalar("SELECT source FROM organization_aliases WHERE id::text = $1")
            .bind(&alias_id)
            .fetch_one(&pool)
            .await
            .expect("alias source");
    assert!(alias_source.starts_with("observation:"));
    let alias_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'organizations'
           AND entity_kind = 'alias'
           AND entity_id = $1
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&alias_id)
    .fetch_one(&pool)
    .await
    .expect("alias observation link");

    let department_response = app
        .clone()
        .oneshot(post(
            &format!("/api/v1/organizations/{}/departments", enc(&oid)),
            json!({
                "name": format!("Engineering {suffix}"),
                "description": "Core platform"
            }),
        ))
        .await
        .expect("department response");
    assert_eq!(department_response.status(), StatusCode::OK);
    let department_id = jb(department_response).await["id"]
        .as_str()
        .expect("department id")
        .to_owned();
    let department_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'organizations'
           AND entity_kind = 'department'
           AND entity_id = $1
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&department_id)
    .fetch_one(&pool)
    .await
    .expect("department observation link");

    let linked_person_id = PersonaFactory::new(&pool)
        .with_persona_id(format!("persona:organization-person:{suffix}"))
        .with_name(format!("Organization Person {suffix}"))
        .with_email(format!("organization-contact-{suffix}@example.com"))
        .create()
        .await
        .expect("create organization persona link target");

    let persona_link_response = app
        .clone()
        .oneshot(post(
            &format!("/api/v1/organizations/{}/persona-links", enc(&oid)),
            json!({
                "persona_id": linked_person_id,
                "role": "cto",
                "department": "engineering",
                "source": "manual"
            }),
        ))
        .await
        .expect("person link response");
    assert_eq!(persona_link_response.status(), StatusCode::OK);
    let persona_link_id = jb(persona_link_response).await["id"]
        .as_str()
        .expect("person link id")
        .to_owned();
    let persona_link_source: String =
        sqlx::query_scalar("SELECT source FROM organization_persona_links WHERE id::text = $1")
            .bind(&persona_link_id)
            .fetch_one(&pool)
            .await
            .expect("person link source");
    assert!(persona_link_source.starts_with("observation:"));
    let persona_link_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'organizations'
           AND entity_kind = 'persona_link'
           AND entity_id = $1
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&persona_link_id)
    .fetch_one(&pool)
    .await
    .expect("person link observation link");

    let update_response = app
        .clone()
        .oneshot(put(
            &format!("/api/v1/organizations/{}", enc(&oid)),
            json!({
                "display_name": format!("Updated Org {suffix}"),
                "description": "Observation-backed update"
            }),
        ))
        .await
        .expect("update response");
    assert_eq!(update_response.status(), StatusCode::OK);
    let update_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'organizations'
           AND entity_kind = 'organization'
           AND entity_id = $1
           AND metadata ->> 'action' = 'update'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&oid)
    .fetch_one(&pool)
    .await
    .expect("organization update observation link");

    let watchlist_response = app
        .clone()
        .oneshot(post(
            &format!("/api/v1/organizations/{}/watchlist", enc(&oid)),
            json!({}),
        ))
        .await
        .expect("watchlist response");
    assert_eq!(watchlist_response.status(), StatusCode::OK);
    let watchlist_source: String = sqlx::query_scalar(
        "SELECT source FROM organization_preferences WHERE organization_id = $1 AND preference_type = 'ui:watchlist'",
    )
    .bind(&oid)
    .fetch_one(&pool)
    .await
    .expect("watchlist source");
    assert!(watchlist_source.starts_with("observation:"));
    let watchlist_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'organizations'
           AND entity_kind = 'watchlist_toggle'
           AND entity_id = $1
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&oid)
    .fetch_one(&pool)
    .await
    .expect("watchlist observation link");

    let archive_response = app
        .oneshot(post(
            &format!("/api/v1/organizations/{}/archive", enc(&oid)),
            json!({}),
        ))
        .await
        .expect("archive response");
    assert_eq!(archive_response.status(), StatusCode::OK);
    let archive_observation_id: String = sqlx::query_scalar(
        "SELECT observation_id
         FROM observation_links
         WHERE domain = 'organizations'
           AND entity_kind = 'organization'
           AND entity_id = $1
           AND metadata ->> 'action' = 'archive'
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(&oid)
    .fetch_one(&pool)
    .await
    .expect("organization archive observation link");

    for observation_id in [
        create_observation_id.clone(),
        identity_observation_id.clone(),
        alias_observation_id.clone(),
        department_observation_id.clone(),
        persona_link_observation_id.clone(),
        update_observation_id.clone(),
        watchlist_observation_id.clone(),
        archive_observation_id.clone(),
    ] {
        let row = sqlx::query(
            "SELECT observation.observation_id, observation.origin_kind, kind.code AS kind_code
             FROM observations observation
             JOIN observation_kind_definitions kind
               ON kind.kind_definition_id = observation.kind_definition_id
             WHERE observation.observation_id = $1",
        )
        .bind(&observation_id)
        .fetch_one(&pool)
        .await
        .expect("stored observation");
        assert_eq!(
            row.try_get::<String, _>("origin_kind")
                .expect("origin kind"),
            "manual"
        );
    }

    for observation_id in [
        create_observation_id,
        watchlist_observation_id,
        update_observation_id,
        archive_observation_id,
    ] {
        let kind_code: String = sqlx::query_scalar(
            "SELECT kind.code AS kind_code
                 FROM observations observation
                 JOIN observation_kind_definitions kind
                   ON kind.kind_definition_id = observation.kind_definition_id
                 WHERE observation.observation_id = $1",
        )
        .bind(&observation_id)
        .fetch_one(&pool)
        .await
        .expect("organization mutation kind code");
        assert_eq!(kind_code, "ORGANIZATION_MUTATION");
    }

    for observation_id in [
        identity_observation_id,
        alias_observation_id,
        department_observation_id,
        persona_link_observation_id,
    ] {
        let kind_code: String = sqlx::query_scalar(
            "SELECT kind.code AS kind_code
                 FROM observations observation
                 JOIN observation_kind_definitions kind
                   ON kind.kind_definition_id = observation.kind_definition_id
                 WHERE observation.observation_id = $1",
        )
        .bind(&observation_id)
        .fetch_one(&pool)
        .await
        .expect("organization record mutation kind code");
        assert_eq!(kind_code, "ORGANIZATION_RECORD_MUTATION");
    }
}

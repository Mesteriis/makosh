use makosh_backend_testkit::context::TestContext;

use serde_json::json;
use sqlx::Row;
use tower::ServiceExt;

use super::support::{
    LOCAL_API_TOKEN, build_cal_app, delete_request_with_token, get_request_with_token, json_body,
    post_request_with_token, put_request_with_token, unique_suffix, urlencoding_percent_encode,
};
use makosh_hub_backend::platform::storage::database::Database;

#[tokio::test]
async fn calendar_accounts_crud_against_postgres() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let database = Database::connect(Some(&database_url))
        .await
        .expect("database connection");
    let pool = database.pool().expect("configured pool").clone();
    let app = build_cal_app(&database_url).await;
    let acct_name = format!("API Cal Acct {suffix}");

    let response = app.clone().oneshot(post_request_with_token(
        "/api/v1/calendar/accounts",
        json!({"provider": "google", "account_name": &acct_name, "email": format!("cal-{suffix}@example.com")}),
        LOCAL_API_TOKEN,
    )).await.expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let created = json_body(response).await;
    let account_id = created["account_id"]
        .as_str()
        .expect("account_id")
        .to_owned();
    assert_eq!(created["provider"], json!("google"));
    let created_observation_id: String = sqlx::query_scalar(
        r#"
        SELECT observation_id
        FROM observation_links
        WHERE entity_kind = 'calendar_account'
          AND entity_id = $1
          AND relationship_kind = 'create'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("calendar account create observation");

    let response = app
        .clone()
        .oneshot(get_request_with_token(
            &format!(
                "/api/v1/calendar/accounts/{}",
                urlencoding_percent_encode(&account_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );

    let response = app
        .clone()
        .oneshot(put_request_with_token(
            &format!(
                "/api/v1/calendar/accounts/{}",
                urlencoding_percent_encode(&account_id)
            ),
            json!({"account_name": format!("Updated {acct_name}")}),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let updated = json_body(response).await;
    assert_eq!(
        updated["account_name"],
        json!(format!("Updated {acct_name}"))
    );
    let updated_observation_id: String = sqlx::query_scalar(
        r#"
        SELECT observation_id
        FROM observation_links
        WHERE entity_kind = 'calendar_account'
          AND entity_id = $1
          AND relationship_kind = 'update'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("calendar account update observation");
    assert_ne!(updated_observation_id, created_observation_id);

    let response = app
        .clone()
        .oneshot(delete_request_with_token(
            &format!(
                "/api/v1/calendar/accounts/{}",
                urlencoding_percent_encode(&account_id)
            ),
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let deleted = json_body(response).await;
    assert_eq!(deleted["deleted"], json!(true));
    let deleted_observation_id: String = sqlx::query_scalar(
        r#"
        SELECT observation_id
        FROM observation_links
        WHERE entity_kind = 'calendar_account'
          AND entity_id = $1
          AND relationship_kind = 'delete'
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(&account_id)
    .fetch_one(&pool)
    .await
    .expect("calendar account delete observation");
    assert_ne!(deleted_observation_id, updated_observation_id);
    let delete_observation = sqlx::query(
        "SELECT observation.origin_kind, kind.code AS kind_code
         FROM observations observation
         JOIN observation_kind_definitions kind
           ON kind.kind_definition_id = observation.kind_definition_id
         WHERE observation.observation_id = $1",
    )
    .bind(&deleted_observation_id)
    .fetch_one(&pool)
    .await
    .expect("calendar account delete observation row");
    assert_eq!(
        delete_observation
            .try_get::<String, _>("origin_kind")
            .expect("origin kind"),
        "manual"
    );
    assert_eq!(
        delete_observation
            .try_get::<String, _>("kind_code")
            .expect("kind code"),
        "CALENDAR_ACCOUNT_MUTATION"
    );
}

#[tokio::test]
async fn calendar_accounts_list_returns_items() {
    let test_context = TestContext::new().await;
    let database_url = test_context.connection_string();
    let suffix = unique_suffix();
    let app = build_cal_app(&database_url).await;

    let response = app.clone().oneshot(post_request_with_token(
        "/api/v1/calendar/accounts",
        json!({"provider": "google", "account_name": format!("List Acct {suffix}"), "email": format!("list-{suffix}@example.com")}),
        LOCAL_API_TOKEN,
    )).await.expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );

    let response = app
        .oneshot(get_request_with_token(
            "/api/v1/calendar/accounts",
            LOCAL_API_TOKEN,
        ))
        .await
        .expect("response");
    assert!(
        !response.status().is_server_error(),
        "status={}",
        response.status()
    );
    let body = json_body(response).await;
    assert!(!body["items"].as_array().expect("items").is_empty());
}

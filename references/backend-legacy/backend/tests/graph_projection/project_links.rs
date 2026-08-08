use makosh_hub_backend::domains::documents::core::models::NewDocumentImport;
use makosh_hub_backend::domains::projects::core::{ids::project_graph_node_id, models::NewProject};
use makosh_hub_backend::domains::projects::link_reviews::models::{
    ProjectLinkReviewCommand, ProjectLinkReviewState, ProjectLinkTargetKind,
};
use makosh_hub_backend::platform::graph::{GraphNodeKind, node_id};

use super::support::{
    ExpectedProjectEdge, assert_project_edge_with_evidence, cleanup_project_graph_fixture,
    live_projection_context, project_graph_counts, seed_message, unique_suffix,
};

#[tokio::test]
async fn graph_projection_links_projects_to_keyword_messages_documents_and_people_against_postgres()
{
    let Some(context) = live_projection_context("project graph projection").await else {
        return;
    };
    let suffix = unique_suffix();
    let keyword = format!("GraphProject{suffix}");
    let project_id = format!("project:v1:graph:{suffix}");
    context
        .project_store
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Graph Project {suffix}"),
                "Product Development",
                "Graph project projection test",
                "Alex Morgan",
                vec![keyword.clone()],
            )
            .progress(55),
        )
        .await
        .expect("upsert graph project");
    let owner = context
        .person_store
        .upsert_email_persona(&format!("graph-project-owner-{suffix}@example.com"))
        .await
        .expect("upsert graph project owner");
    let projected = seed_message(
        &context,
        suffix,
        owner
            .email_address
            .as_deref()
            .expect("graph project owner fixture must have email"),
        &[format!("graph-project-reviewer-{suffix}@example.com")],
        &format!("provider-graph-project-{suffix}"),
        &format!("{keyword} kickoff"),
    )
    .await;
    let document_id = format!("doc_graph_project_{suffix}");
    context
        .document_store
        .import_document(&NewDocumentImport::markdown(
            &document_id,
            format!("{keyword} notes.md"),
            "# Notes\n\nProject graph content.",
        ))
        .await
        .expect("import graph project document");

    context
        .graph_projection
        .project_from_v1()
        .await
        .expect("first graph projection");
    let counts_after_first = project_graph_counts(&context.pool, &project_id).await;
    context
        .graph_projection
        .project_from_v1()
        .await
        .expect("second graph projection");
    let counts_after_second = project_graph_counts(&context.pool, &project_id).await;
    assert_eq!(counts_after_first, counts_after_second);

    let project_node_id = project_graph_node_id(&project_id);
    let owner_node_id = node_id(GraphNodeKind::Persona, &owner.persona_id);
    let reviewer_node_id = node_id(
        GraphNodeKind::EmailAddress,
        &format!("graph-project-reviewer-{suffix}@example.com"),
    );
    assert_project_edge_with_evidence(
        &context.pool,
        ExpectedProjectEdge {
            source_node_id: &project_node_id,
            target_node_id: &node_id(GraphNodeKind::Message, &projected.message_id),
            relationship_type: "project_has_message",
            source_kind: "message",
            source_id: &projected.message_id,
            observation_id: Some(projected.observation_id.as_str()),
            review_state: "suggested",
            confidence: 0.75,
        },
    )
    .await;
    assert_project_edge_with_evidence(
        &context.pool,
        ExpectedProjectEdge {
            source_node_id: &project_node_id,
            target_node_id: &node_id(GraphNodeKind::Document, &document_id),
            relationship_type: "project_has_document",
            source_kind: "document",
            source_id: &document_id,
            observation_id: None,
            review_state: "suggested",
            confidence: 0.75,
        },
    )
    .await;
    assert_project_edge_with_evidence(
        &context.pool,
        ExpectedProjectEdge {
            source_node_id: &project_node_id,
            target_node_id: &owner_node_id,
            relationship_type: "project_involves_persona",
            source_kind: "message",
            source_id: &projected.message_id,
            observation_id: Some(projected.observation_id.as_str()),
            review_state: "suggested",
            confidence: 0.75,
        },
    )
    .await;
    assert_project_edge_with_evidence(
        &context.pool,
        ExpectedProjectEdge {
            source_node_id: &project_node_id,
            target_node_id: &reviewer_node_id,
            relationship_type: "project_involves_email_address",
            source_kind: "message",
            source_id: &projected.message_id,
            observation_id: Some(projected.observation_id.as_str()),
            review_state: "suggested",
            confidence: 0.75,
        },
    )
    .await;

    cleanup_project_graph_fixture(&context.pool, &project_id).await;
}

#[tokio::test]
async fn graph_projection_omits_rejected_project_link_against_postgres() {
    let Some(context) = live_projection_context("project graph projection rejected link").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let keyword = format!("GraphReject{suffix}");
    let project_id = format!("project:v1:graph-reject:{suffix}");

    context
        .project_store
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Graph Reject Project {suffix}"),
                "Product Development",
                "Graph project rejected link test",
                "Alex Morgan",
                vec![keyword.clone()],
            )
            .progress(50),
        )
        .await
        .expect("upsert graph reject project");

    let projected = seed_message(
        &context,
        suffix,
        &format!("owner-reject-{suffix}@example.com"),
        &[format!("reviewer-reject-{suffix}@example.com")],
        &format!("provider-graph-reject-{suffix}"),
        &format!("{keyword} kickoff"),
    )
    .await;

    context
        .project_link_review_store
        .set_review_state(&ProjectLinkReviewCommand {
            command_id: format!("graph-reject-{suffix}"),
            project_id: project_id.clone(),
            target_kind: ProjectLinkTargetKind::Message,
            target_id: projected.message_id.clone(),
            review_state: ProjectLinkReviewState::UserRejected,
            actor_id: format!("reviewer-actor-{suffix}"),
        })
        .await
        .expect("set rejected link review");

    context
        .graph_projection
        .project_from_v1()
        .await
        .expect("project projection for rejected link");

    let project_node_id = project_graph_node_id(&project_id);
    let message_node_id = node_id(GraphNodeKind::Message, &projected.message_id);
    let link_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT count(*)
        FROM graph_edges
        WHERE source_node_id = $1
          AND target_node_id = $2
          AND relationship_type = 'project_has_message'
        "#,
    )
    .bind(&project_node_id)
    .bind(&message_node_id)
    .fetch_one(&context.pool)
    .await
    .expect("rejected project link count");
    assert_eq!(link_count, 0);

    cleanup_project_graph_fixture(&context.pool, &project_id).await;
}

#[tokio::test]
async fn graph_projection_marks_confirmed_project_link_user_confirmed_against_postgres() {
    let Some(context) = live_projection_context("project graph projection confirmed link").await
    else {
        return;
    };
    let suffix = unique_suffix();
    let keyword = format!("GraphConfirm{suffix}");
    let project_id = format!("project:v1:graph-confirm:{suffix}");

    context
        .project_store
        .upsert_project(
            &NewProject::active(
                &project_id,
                format!("Graph Confirm Project {suffix}"),
                "Product Development",
                "Graph project confirmed link test",
                "Alex Morgan",
                vec![keyword.clone()],
            )
            .progress(50),
        )
        .await
        .expect("upsert graph confirm project");

    let projected = seed_message(
        &context,
        suffix,
        &format!("owner-confirm-{suffix}@example.com"),
        &[format!("reviewer-confirm-{suffix}@example.com")],
        &format!("provider-graph-confirm-{suffix}"),
        &format!("{keyword} kickoff"),
    )
    .await;

    context
        .project_link_review_store
        .set_review_state(&ProjectLinkReviewCommand {
            command_id: format!("graph-confirm-{suffix}"),
            project_id: project_id.clone(),
            target_kind: ProjectLinkTargetKind::Message,
            target_id: projected.message_id.clone(),
            review_state: ProjectLinkReviewState::UserConfirmed,
            actor_id: format!("reviewer-actor-{suffix}"),
        })
        .await
        .expect("set confirmed link review");

    context
        .graph_projection
        .project_from_v1()
        .await
        .expect("project projection for confirmed link");

    let project_node_id = project_graph_node_id(&project_id);
    assert_project_edge_with_evidence(
        &context.pool,
        ExpectedProjectEdge {
            source_node_id: &project_node_id,
            target_node_id: &node_id(GraphNodeKind::Message, &projected.message_id),
            relationship_type: "project_has_message",
            source_kind: "message",
            source_id: &projected.message_id,
            observation_id: Some(projected.observation_id.as_str()),
            review_state: "user_confirmed",
            confidence: 1.0,
        },
    )
    .await;

    cleanup_project_graph_fixture(&context.pool, &project_id).await;
}

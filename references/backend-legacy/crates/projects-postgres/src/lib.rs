use chrono::{DateTime, NaiveDate, Utc};
use makosh_projects_api::{
    Project, ProjectCandidateReadPort, ProjectCandidatesFuture, ProjectDetail, ProjectDetailFuture,
    ProjectDocumentSummary, ProjectDocumentsFuture, ProjectGraphReadPort, ProjectLinkCandidate,
    ProjectLinkReviewState, ProjectListFuture, ProjectListResponse, ProjectMatchedDocument,
    ProjectMatchedMessage, ProjectMessageSummary, ProjectMessagesFuture, ProjectPersonaSummary,
    ProjectProjectionFuture, ProjectProjectionSource, ProjectQueryError, ProjectReadPort,
    ProjectStats, ProjectSummary, ProjectTimelineItem, ProjectUpsert, ProjectWriteFuture,
    ProjectWritePort,
};
use sqlx::{PgPool, Row, postgres::PgRow};

const DEFAULT_LIMIT: i64 = 25;
const DETAIL_LIMIT: i64 = 8;

#[derive(Clone)]
pub struct ProjectPostgresReadQuery {
    pool: PgPool,
}

impl ProjectPostgresReadQuery {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn parse_review_state(value: String) -> Result<ProjectLinkReviewState, ProjectQueryError> {
    match value.as_str() {
        "suggested" => Ok(ProjectLinkReviewState::Suggested),
        "user_confirmed" => Ok(ProjectLinkReviewState::UserConfirmed),
        "user_rejected" => Ok(ProjectLinkReviewState::UserRejected),
        other => Err(ProjectQueryError(format!(
            "invalid project review state: {other}"
        ))),
    }
}

fn recipients_from_value(value: serde_json::Value) -> Result<Vec<String>, ProjectQueryError> {
    value
        .as_array()
        .ok_or_else(|| ProjectQueryError("project message recipients must be an array".into()))?
        .iter()
        .map(|value| {
            value.as_str().map(ToOwned::to_owned).ok_or_else(|| {
                ProjectQueryError("project message recipient must be a string".into())
            })
        })
        .collect()
}

impl ProjectReadPort for ProjectPostgresReadQuery {
    fn list<'a>(&'a self, limit: Option<i64>) -> ProjectListFuture<'a> {
        Box::pin(async move {
            let limit = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, 100);
            let rows = sqlx::query(PROJECT_SQL)
                .bind(limit)
                .fetch_all(&self.pool)
                .await
                .map_err(error)?;
            let mut items = Vec::with_capacity(rows.len());
            for row in rows {
                let project = map_project(row)?;
                items.push(ProjectSummary {
                    graph_node_id: graph_node_id(&project.project_id),
                    stats: self.stats(&project.project_id).await?,
                    project,
                });
            }
            Ok(ProjectListResponse { items })
        })
    }

    fn detail<'a>(&'a self, project_id: &'a str) -> ProjectDetailFuture<'a> {
        Box::pin(async move {
            let row = sqlx::query(PROJECT_BY_ID_SQL)
                .bind(project_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(error)?;
            let Some(row) = row else {
                return Ok(None);
            };
            let project = map_project(row)?;
            let messages = self.target_ids(project_id, "message").await?;
            let documents = self.target_ids(project_id, "document").await?;
            let stats = self
                .stats_with_ids(&project.project_id, &messages, &documents)
                .await?;
            let timeline = self.timeline(&messages, &documents).await?;
            let key_personas = self.personas(&messages).await?;
            let recent_messages = self.messages(&messages).await?;
            let document_items = self.documents(&documents).await?;
            Ok(Some(ProjectDetail {
                graph_node_id: graph_node_id(&project.project_id),
                stats,
                timeline,
                key_personas: key_personas.clone(),
                key_people: key_personas,
                recent_messages,
                documents: document_items,
                project,
            }))
        })
    }
}

impl ProjectCandidateReadPort for ProjectPostgresReadQuery {
    fn candidates<'a>(&'a self, project_id: &'a str) -> ProjectCandidatesFuture<'a> {
        Box::pin(async move {
            let messages = self.target_ids(project_id, "message").await?;
            let documents = self.target_ids(project_id, "document").await?;
            let mut result = Vec::new();
            if !messages.is_empty() {
                let rows = sqlx::query("SELECT message_id, observation_id, account_id, subject, sender, COALESCE(occurred_at, projected_at) occurred_at FROM communication_messages WHERE message_id=ANY($1)")
                    .bind(&messages).fetch_all(&self.pool).await.map_err(error)?;
                for row in rows {
                    result.push(ProjectLinkCandidate {
                        target_kind: "message".into(),
                        target_id: row.try_get("message_id").map_err(error)?,
                        observation_id: row.try_get("observation_id").map_err(error)?,
                        account_id: row.try_get("account_id").map_err(error)?,
                        source_fingerprint: None,
                        title: row.try_get("subject").map_err(error)?,
                        subtitle: row.try_get("sender").map_err(error)?,
                        occurred_at: row.try_get("occurred_at").map_err(error)?,
                    });
                }
            }
            if !documents.is_empty() {
                let rows = sqlx::query("SELECT document_id, observation_id, source_fingerprint, document_kind, title, imported_at FROM documents WHERE document_id=ANY($1)")
                    .bind(&documents).fetch_all(&self.pool).await.map_err(error)?;
                for row in rows {
                    result.push(ProjectLinkCandidate {
                        target_kind: "document".into(),
                        target_id: row.try_get("document_id").map_err(error)?,
                        observation_id: row.try_get("observation_id").map_err(error)?,
                        account_id: None,
                        source_fingerprint: row.try_get("source_fingerprint").map_err(error)?,
                        title: row.try_get("title").map_err(error)?,
                        subtitle: row.try_get("document_kind").map_err(error)?,
                        occurred_at: row.try_get("imported_at").map_err(error)?,
                    });
                }
            }
            result.sort_by(|a, b| b.occurred_at.cmp(&a.occurred_at));
            Ok(result)
        })
    }
}

impl ProjectGraphReadPort for ProjectPostgresReadQuery {
    fn projection_projects<'a>(&'a self) -> ProjectProjectionFuture<'a> {
        Box::pin(async move {
            let rows = sqlx::query("SELECT project_id,name,kind,status,description,owner_display_name,progress_percent,start_date,target_date,created_at,updated_at FROM projects ORDER BY project_id")
                .fetch_all(&self.pool).await.map_err(error)?;
            let mut projects = Vec::with_capacity(rows.len());
            for row in rows {
                let project = map_project(row)?;
                let keywords = sqlx::query_scalar(
                    "SELECT keyword FROM project_keywords WHERE project_id=$1 ORDER BY keyword",
                )
                .bind(&project.project_id)
                .fetch_all(&self.pool)
                .await
                .map_err(error)?;
                projects.push(ProjectProjectionSource { project, keywords });
            }
            Ok(projects)
        })
    }

    fn matching_messages<'a>(&'a self, project_id: &'a str) -> ProjectMessagesFuture<'a> {
        Box::pin(async move {
            let ids = self.target_ids(project_id, "message").await?;
            if ids.is_empty() {
                return Ok(Vec::new());
            }
            let rows = sqlx::query("SELECT message.message_id,message.raw_record_id,message.observation_id,message.account_id,message.provider_record_id,message.subject,message.sender,message.recipients,message.occurred_at,message.projected_at,COALESCE(review.review_state,'suggested') AS review_state FROM communication_messages message LEFT JOIN project_link_reviews review ON review.project_id=$2 AND review.target_kind='message' AND review.target_id=message.message_id WHERE message.message_id=ANY($1) ORDER BY COALESCE(message.occurred_at,message.projected_at) DESC,message.message_id")
                .bind(&ids).bind(project_id).fetch_all(&self.pool).await.map_err(error)?;
            rows.into_iter()
                .map(|row| {
                    Ok(ProjectMatchedMessage {
                        message_id: row.try_get("message_id").map_err(error)?,
                        raw_record_id: row.try_get("raw_record_id").map_err(error)?,
                        observation_id: row.try_get("observation_id").map_err(error)?,
                        account_id: row.try_get("account_id").map_err(error)?,
                        provider_record_id: row.try_get("provider_record_id").map_err(error)?,
                        subject: row.try_get("subject").map_err(error)?,
                        sender: row.try_get("sender").map_err(error)?,
                        recipients: recipients_from_value(
                            row.try_get("recipients").map_err(error)?,
                        )?,
                        occurred_at: row.try_get("occurred_at").map_err(error)?,
                        projected_at: row.try_get("projected_at").map_err(error)?,
                        review_state: parse_review_state(
                            row.try_get("review_state").map_err(error)?,
                        )?,
                    })
                })
                .collect()
        })
    }

    fn matching_documents<'a>(&'a self, project_id: &'a str) -> ProjectDocumentsFuture<'a> {
        Box::pin(async move {
            let ids = self.target_ids(project_id, "document").await?;
            if ids.is_empty() {
                return Ok(Vec::new());
            }
            let rows = sqlx::query("SELECT document.document_id,document.document_kind,document.title,document.observation_id,document.source_fingerprint,document.imported_at,COALESCE(review.review_state,'suggested') AS review_state FROM documents document LEFT JOIN project_link_reviews review ON review.project_id=$2 AND review.target_kind='document' AND review.target_id=document.document_id WHERE document.document_id=ANY($1) ORDER BY document.imported_at DESC,document.document_id")
                .bind(&ids).bind(project_id).fetch_all(&self.pool).await.map_err(error)?;
            rows.into_iter()
                .map(|row| {
                    Ok(ProjectMatchedDocument {
                        document_id: row.try_get("document_id").map_err(error)?,
                        document_kind: row.try_get("document_kind").map_err(error)?,
                        title: row.try_get("title").map_err(error)?,
                        observation_id: row.try_get("observation_id").map_err(error)?,
                        source_fingerprint: row.try_get("source_fingerprint").map_err(error)?,
                        imported_at: row.try_get("imported_at").map_err(error)?,
                        review_state: parse_review_state(
                            row.try_get("review_state").map_err(error)?,
                        )?,
                    })
                })
                .collect()
        })
    }
}

impl ProjectWritePort for ProjectPostgresReadQuery {
    fn upsert<'a>(&'a self, project: &'a ProjectUpsert) -> ProjectWriteFuture<'a> {
        Box::pin(async move {
            let mut tx = self.pool.begin().await.map_err(error)?;
            let row = sqlx::query("INSERT INTO projects (project_id,name,kind,status,description,owner_display_name,progress_percent,start_date,target_date) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9) ON CONFLICT (project_id) DO UPDATE SET name=EXCLUDED.name,kind=EXCLUDED.kind,status=EXCLUDED.status,description=EXCLUDED.description,owner_display_name=EXCLUDED.owner_display_name,progress_percent=EXCLUDED.progress_percent,start_date=EXCLUDED.start_date,target_date=EXCLUDED.target_date,updated_at=now() RETURNING project_id,name,kind,status,description,owner_display_name,progress_percent,start_date,target_date,created_at,updated_at")
                .bind(&project.project_id).bind(&project.name).bind(&project.kind).bind(&project.status).bind(&project.description).bind(&project.owner_display_name).bind(project.progress_percent).bind(project.start_date).bind(project.target_date).fetch_one(&mut *tx).await.map_err(error)?;
            sqlx::query("DELETE FROM project_keywords WHERE project_id=$1")
                .bind(&project.project_id)
                .execute(&mut *tx)
                .await
                .map_err(error)?;
            for keyword in &project.keywords {
                sqlx::query("INSERT INTO project_keywords (project_id,keyword) VALUES ($1,$2) ON CONFLICT (project_id,keyword) DO NOTHING").bind(&project.project_id).bind(keyword).execute(&mut *tx).await.map_err(error)?;
            }
            tx.commit().await.map_err(error)?;
            map_project(row)
        })
    }
}

impl ProjectPostgresReadQuery {
    async fn target_ids(
        &self,
        project_id: &str,
        kind: &str,
    ) -> Result<Vec<String>, ProjectQueryError> {
        let (table, text_column, id_column) = match kind {
            "message" => ("communication_messages", "body_text", "message_id"),
            "document" => ("documents", "extracted_text", "document_id"),
            _ => return Err(ProjectQueryError("invalid project target kind".into())),
        };
        let title_column = if kind == "message" {
            "subject"
        } else {
            "title"
        };
        let sql = format!(
            "WITH keyword_matches AS (SELECT {id_column} AS target_id FROM {table} target WHERE EXISTS (SELECT 1 FROM project_keywords keyword WHERE keyword.project_id = $1 AND (position(lower(keyword.keyword) in lower(target.{title_column})) > 0 OR position(lower(keyword.keyword) in lower(target.{text_column})) > 0))), confirmed AS (SELECT target_id FROM project_link_reviews WHERE project_id=$1 AND target_kind=$2 AND review_state='user_confirmed'), rejected AS (SELECT target_id FROM project_link_reviews WHERE project_id=$1 AND target_kind=$2 AND review_state='user_rejected'), active AS (SELECT target_id FROM keyword_matches UNION SELECT target_id FROM confirmed) SELECT active.target_id FROM active WHERE NOT EXISTS (SELECT 1 FROM rejected WHERE rejected.target_id=active.target_id) ORDER BY active.target_id"
        );
        let rows = sqlx::query(&sql)
            .bind(project_id)
            .bind(kind)
            .fetch_all(&self.pool)
            .await
            .map_err(error)?;
        rows.into_iter()
            .map(|r| r.try_get("target_id").map_err(error))
            .collect()
    }

    async fn stats(&self, project_id: &str) -> Result<ProjectStats, ProjectQueryError> {
        let messages = self.target_ids(project_id, "message").await?;
        let documents = self.target_ids(project_id, "document").await?;
        self.stats_with_ids(project_id, &messages, &documents).await
    }

    async fn stats_with_ids(
        &self,
        project_id: &str,
        messages: &[String],
        documents: &[String],
    ) -> Result<ProjectStats, ProjectQueryError> {
        let message_count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM communication_messages WHERE message_id = ANY($1)",
        )
        .bind(messages)
        .fetch_one(&self.pool)
        .await
        .map_err(error)?;
        let document_count: i64 =
            sqlx::query_scalar("SELECT count(*) FROM documents WHERE document_id = ANY($1)")
                .bind(documents)
                .fetch_one(&self.pool)
                .await
                .map_err(error)?;
        let persona_count: i64 = sqlx::query_scalar("WITH m AS (SELECT sender, recipients FROM communication_messages WHERE message_id=ANY($1)), p AS (SELECT lower(trim(sender)) email FROM m UNION ALL SELECT lower(trim(x.value)) FROM m, jsonb_array_elements_text(m.recipients) x) SELECT count(DISTINCT email) FROM p WHERE email <> ''").bind(messages).fetch_one(&self.pool).await.map_err(error)?;
        let graph_connection_count: i64 = sqlx::query_scalar("SELECT count(*) FROM graph_edges WHERE valid_to IS NULL AND (source_node_id=$1 OR target_node_id=$1)").bind(graph_node_id(project_id)).fetch_one(&self.pool).await.map_err(error)?;
        let latest_activity_at: Option<DateTime<Utc>> = sqlx::query_scalar("WITH a AS (SELECT COALESCE(occurred_at, projected_at) at FROM communication_messages WHERE message_id=ANY($1) UNION ALL SELECT imported_at FROM documents WHERE document_id=ANY($2)) SELECT max(at) FROM a").bind(messages).bind(documents).fetch_one(&self.pool).await.map_err(error)?;
        Ok(ProjectStats {
            message_count,
            document_count,
            persona_count,
            people_count: persona_count,
            graph_connection_count,
            latest_activity_at,
        })
    }

    async fn timeline(
        &self,
        messages: &[String],
        documents: &[String],
    ) -> Result<Vec<ProjectTimelineItem>, ProjectQueryError> {
        let rows = sqlx::query("WITH a AS (SELECT 'message' item_kind, message_id item_id, subject title, sender subtitle, COALESCE(occurred_at, projected_at) occurred_at FROM communication_messages WHERE message_id=ANY($1) UNION ALL SELECT 'document', document_id, title, document_kind, imported_at FROM documents WHERE document_id=ANY($2)) SELECT * FROM a ORDER BY occurred_at DESC, item_kind, item_id LIMIT $3").bind(messages).bind(documents).bind(DETAIL_LIMIT).fetch_all(&self.pool).await.map_err(error)?;
        rows.into_iter()
            .map(|r| {
                Ok(ProjectTimelineItem {
                    item_kind: r.try_get("item_kind").map_err(error)?,
                    item_id: r.try_get("item_id").map_err(error)?,
                    title: r.try_get("title").map_err(error)?,
                    subtitle: r.try_get("subtitle").map_err(error)?,
                    occurred_at: r.try_get("occurred_at").map_err(error)?,
                })
            })
            .collect()
    }

    async fn personas(
        &self,
        messages: &[String],
    ) -> Result<Vec<ProjectPersonaSummary>, ProjectQueryError> {
        if messages.is_empty() {
            return Ok(vec![]);
        }
        let rows=sqlx::query("WITH m AS (SELECT sender, recipients, COALESCE(occurred_at, projected_at) occurred_at FROM communication_messages WHERE message_id=ANY($1)), p AS (SELECT lower(trim(sender)) email, occurred_at FROM m UNION ALL SELECT lower(trim(x.value)), m.occurred_at FROM m, jsonb_array_elements_text(m.recipients) x) SELECT COALESCE(person.display_name,p.email) display_name,p.email email_address,count(*)::BIGINT interaction_count,max(p.occurred_at) last_interaction_at FROM p LEFT JOIN personas person ON person.email_address=p.email WHERE p.email<>'' GROUP BY p.email,person.display_name ORDER BY interaction_count DESC,last_interaction_at DESC NULLS LAST,display_name LIMIT $2").bind(messages).bind(DETAIL_LIMIT).fetch_all(&self.pool).await.map_err(error)?;
        rows.into_iter()
            .map(|r| {
                Ok(ProjectPersonaSummary {
                    display_name: r.try_get("display_name").map_err(error)?,
                    email_address: r.try_get("email_address").map_err(error)?,
                    interaction_count: r.try_get("interaction_count").map_err(error)?,
                    last_interaction_at: r.try_get("last_interaction_at").map_err(error)?,
                })
            })
            .collect()
    }

    async fn messages(
        &self,
        ids: &[String],
    ) -> Result<Vec<ProjectMessageSummary>, ProjectQueryError> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let rows=sqlx::query("SELECT message_id,subject,sender,COALESCE(occurred_at,projected_at) occurred_at FROM communication_messages WHERE message_id=ANY($1) ORDER BY occurred_at DESC,message_id LIMIT $2").bind(ids).bind(DETAIL_LIMIT).fetch_all(&self.pool).await.map_err(error)?;
        rows.into_iter()
            .map(|r| {
                Ok(ProjectMessageSummary {
                    message_id: r.try_get("message_id").map_err(error)?,
                    subject: r.try_get("subject").map_err(error)?,
                    sender: r.try_get("sender").map_err(error)?,
                    occurred_at: r.try_get("occurred_at").map_err(error)?,
                })
            })
            .collect()
    }
    async fn documents(
        &self,
        ids: &[String],
    ) -> Result<Vec<ProjectDocumentSummary>, ProjectQueryError> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        let rows=sqlx::query("SELECT document_id,document_kind,title,observation_id,imported_at FROM documents WHERE document_id=ANY($1) ORDER BY imported_at DESC,document_id LIMIT $2").bind(ids).bind(DETAIL_LIMIT).fetch_all(&self.pool).await.map_err(error)?;
        rows.into_iter()
            .map(|r| {
                Ok(ProjectDocumentSummary {
                    document_id: r.try_get("document_id").map_err(error)?,
                    document_kind: r.try_get("document_kind").map_err(error)?,
                    title: r.try_get("title").map_err(error)?,
                    observation_id: r.try_get("observation_id").map_err(error)?,
                    imported_at: r.try_get("imported_at").map_err(error)?,
                })
            })
            .collect()
    }
}

const PROJECT_SQL: &str = "SELECT project_id,name,kind,status,description,owner_display_name,progress_percent,start_date,target_date,created_at,updated_at FROM projects ORDER BY updated_at DESC,name,project_id LIMIT $1";
const PROJECT_BY_ID_SQL: &str = "SELECT project_id,name,kind,status,description,owner_display_name,progress_percent,start_date,target_date,created_at,updated_at FROM projects WHERE project_id=$1";
fn graph_node_id(id: &str) -> String {
    format!("graph:node:v1:project:{id}")
}
fn error(e: sqlx::Error) -> ProjectQueryError {
    ProjectQueryError(e.to_string())
}
fn map_project(row: PgRow) -> Result<Project, ProjectQueryError> {
    Ok(Project {
        project_id: row.try_get("project_id").map_err(error)?,
        name: row.try_get("name").map_err(error)?,
        kind: row.try_get("kind").map_err(error)?,
        status: row.try_get("status").map_err(error)?,
        description: row.try_get("description").map_err(error)?,
        owner_display_name: row.try_get("owner_display_name").map_err(error)?,
        progress_percent: row.try_get("progress_percent").map_err(error)?,
        start_date: row
            .try_get::<Option<NaiveDate>, _>("start_date")
            .map_err(error)?,
        target_date: row.try_get("target_date").map_err(error)?,
        created_at: row.try_get("created_at").map_err(error)?,
        updated_at: row.try_get("updated_at").map_err(error)?,
    })
}

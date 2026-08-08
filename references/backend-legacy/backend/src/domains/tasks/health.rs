use chrono::{DateTime, Duration, Utc};
use serde_json::{Value, json};
use sqlx::Row;
use sqlx::postgres::PgPool;
use thiserror::Error;

use crate::engines::context_packs::{
    errors::ContextPackStoreError, models::ContextPackKind, store::ContextPackStore,
};

pub struct TaskWatchtowerService;

impl TaskWatchtowerService {
    pub async fn overdue(pool: &PgPool) -> Result<Value, TaskHealthError> {
        let now = Utc::now();
        let rows = sqlx::query("SELECT task_id, title, makosh_status, priority_score, due_at FROM tasks WHERE due_at < $1 AND makosh_status NOT IN ('done','cancelled','archived') ORDER BY priority_score DESC NULLS LAST LIMIT 30")
            .bind(now).fetch_all(pool).await?;
        let items: Vec<Value> = rows
            .iter()
            .map(|r| {
                json!({
                    "task_id": r.try_get::<String,_>("task_id").unwrap_or_default(),
                    "title": r.try_get::<String,_>("title").unwrap_or_default(),
                    "status": r.try_get::<String,_>("makosh_status").unwrap_or_default(),
                    "priority": r.try_get::<Option<f64>,_>("priority_score").unwrap_or(None),
                    "due_at": r.try_get::<Option<DateTime<Utc>>,_>("due_at").unwrap_or(None),
                })
            })
            .collect();
        Ok(json!({"overdue": items}))
    }

    pub async fn waiting_too_long(pool: &PgPool, days: i64) -> Result<Value, TaskHealthError> {
        let threshold = Utc::now() - Duration::days(days);
        let rows = sqlx::query("SELECT task_id, title, waiting_reason, updated_at FROM tasks WHERE makosh_status='waiting' AND updated_at < $1 ORDER BY updated_at ASC LIMIT 20")
            .bind(threshold).fetch_all(pool).await?;
        let items: Vec<Value> = rows.iter().map(|r| json!({
            "task_id": r.try_get::<String,_>("task_id").unwrap_or_default(),
            "title": r.try_get::<String,_>("title").unwrap_or_default(),
            "waiting_reason": r.try_get::<Option<String>,_>("waiting_reason").unwrap_or(None),
            "since": r.try_get::<DateTime<Utc>,_>("updated_at").ok(),
        })).collect();
        Ok(json!({"waiting_too_long": items}))
    }

    pub async fn without_context(pool: &PgPool) -> Result<Value, TaskHealthError> {
        let rows = sqlx::query("SELECT task_id, title, makosh_status FROM tasks WHERE makosh_status NOT IN ('done','cancelled','archived') ORDER BY priority_score DESC NULLS LAST LIMIT 50")
            .fetch_all(pool)
            .await?;
        let context_store = ContextPackStore::new(pool.clone());
        let mut items = Vec::new();
        for row in rows {
            let task_id = row.try_get::<String, _>("task_id").unwrap_or_default();
            let has_context = context_store
                .exists(ContextPackKind::Task, &task_id)
                .await?;
            if has_context {
                continue;
            }
            items.push(json!({
                "task_id": task_id,
                "title": row.try_get::<String,_>("title").unwrap_or_default(),
                "status": row.try_get::<String,_>("makosh_status").unwrap_or_default(),
            }));
            if items.len() >= 20 {
                break;
            }
        }
        Ok(json!({"tasks_without_context": items}))
    }

    pub async fn stale_tasks(pool: &PgPool, days: i64) -> Result<Value, TaskHealthError> {
        let threshold = Utc::now() - Duration::days(days);
        let rows = sqlx::query("SELECT task_id, title, makosh_status, updated_at FROM tasks WHERE makosh_status NOT IN ('done','cancelled','archived') AND updated_at < $1 ORDER BY updated_at ASC LIMIT 20")
            .bind(threshold).fetch_all(pool).await?;
        let items: Vec<Value> = rows
            .iter()
            .map(|r| {
                json!({
                    "task_id": r.try_get::<String,_>("task_id").unwrap_or_default(),
                    "title": r.try_get::<String,_>("title").unwrap_or_default(),
                    "status": r.try_get::<String,_>("makosh_status").unwrap_or_default(),
                    "since": r.try_get::<DateTime<Utc>,_>("updated_at").ok(),
                })
            })
            .collect();
        Ok(json!({"stale_tasks": items}))
    }

    pub async fn cycle_time(pool: &PgPool) -> Result<Value, TaskHealthError> {
        let rows = sqlx::query("SELECT EXTRACT(EPOCH FROM (COALESCE(completed_at, now()) - created_at))/3600 as hours, makosh_status FROM tasks WHERE completed_at IS NOT NULL ORDER BY completed_at DESC LIMIT 50")
            .fetch_all(pool).await?;
        let hours: Vec<f64> = rows
            .iter()
            .filter_map(|r| r.try_get::<Option<f64>, _>("hours").unwrap_or(None))
            .collect();
        let avg = if hours.is_empty() {
            0.0
        } else {
            hours.iter().sum::<f64>() / hours.len() as f64
        };
        Ok(json!({"average_cycle_hours": avg, "completed_count": hours.len()}))
    }

    pub async fn workload(pool: &PgPool) -> Result<Value, TaskHealthError> {
        let active = sqlx::query("SELECT COUNT(*) as cnt FROM tasks WHERE makosh_status IN ('new','triaged','ready','in_progress','waiting','blocked','review')")
            .fetch_one(pool).await?;
        let overdue = sqlx::query("SELECT COUNT(*) as cnt FROM tasks WHERE due_at < $1 AND makosh_status NOT IN ('done','cancelled','archived')")
            .bind(Utc::now()).fetch_one(pool).await?;
        Ok(json!({
            "active_count": active.try_get::<Option<i64>,_>("cnt").unwrap_or(Some(0)),
            "overdue_count": overdue.try_get::<Option<i64>,_>("cnt").unwrap_or(Some(0)),
        }))
    }
}

#[derive(Debug, Error)]
pub enum TaskHealthError {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    ContextPack(#[from] ContextPackStoreError),
}

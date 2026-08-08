use makosh_graph_api::{GraphCount, GraphSummary, GraphSummaryQueryPort};
use makosh_graph_postgres::GraphPostgresSummaryQuery;
use makosh_hub_backend::platform::config::app_config::AppConfig;
use makosh_hub_backend::platform::storage::database::Database;
use makosh_hub_backend::workflows::graph_projection::models::GraphProjectionReport;
use makosh_hub_backend::workflows::graph_projection::service::GraphProjectionService;
use serde::Serialize;
use thiserror::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    makosh_hub_backend::app::router::init_tracing();

    let config = AppConfig::from_env()?;
    let database_url = config
        .database_url()
        .ok_or(GraphProjectCommandError::MissingDatabaseUrl)?;
    let database = Database::connect(Some(database_url)).await?;
    let pool = database
        .pool()
        .ok_or(GraphProjectCommandError::MissingDatabaseUrl)?
        .clone();

    let projection = GraphProjectionService::new(pool.clone())
        .project_from_v1()
        .await?;
    let summary = GraphPostgresSummaryQuery::new(pool)
        .summary()
        .await
        .map_err(|error| std::io::Error::other(error.to_string()))?;

    println!(
        "{}",
        serde_json::to_string_pretty(&GraphProjectCommandReport::new(projection, summary))?
    );

    Ok(())
}

#[derive(Debug, Error)]
enum GraphProjectCommandError {
    #[error("DATABASE_URL is required for graph projection")]
    MissingDatabaseUrl,
}

#[derive(Debug, Serialize)]
struct GraphProjectCommandReport {
    projection: ProjectionCounts,
    summary: GraphSummary,
    total_nodes: i64,
    total_edges: i64,
}

impl GraphProjectCommandReport {
    fn new(projection: GraphProjectionReport, summary: GraphSummary) -> Self {
        Self {
            projection: ProjectionCounts::from(projection),
            total_nodes: total_count(&summary.node_counts),
            total_edges: total_count(&summary.edge_counts),
            summary,
        }
    }
}

#[derive(Debug, Serialize)]
struct ProjectionCounts {
    nodes_upserted: usize,
    edges_upserted: usize,
    evidence_upserted: usize,
}

impl From<GraphProjectionReport> for ProjectionCounts {
    fn from(report: GraphProjectionReport) -> Self {
        Self {
            nodes_upserted: report.nodes_upserted,
            edges_upserted: report.edges_upserted,
            evidence_upserted: report.evidence_upserted,
        }
    }
}

fn total_count(counts: &[GraphCount]) -> i64 {
    counts.iter().map(|count| count.count).sum()
}

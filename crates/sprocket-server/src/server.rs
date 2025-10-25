//! Server setup and routing.

use axum::routing::get;
use axum::routing::post;
use axum::Router;
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api::workflows::*;
use crate::api::AppState;
use crate::config::Config;
use crate::db::Database;
use crate::manager::spawn_manager;

/// OpenAPI documentation.
#[derive(OpenApi)]
#[openapi(
    paths(
        submit_workflow,
        get_workflow,
        list_workflows,
        cancel_workflow,
        get_workflow_outputs,
        get_workflow_logs,
    ),
    components(schemas(
        crate::api::models::SubmitWorkflowRequest,
        crate::api::models::SubmitWorkflowResponse,
        crate::api::models::WdlSourceRequest,
        crate::api::models::GetWorkflowResponse,
        crate::api::models::ListWorkflowsQuery,
        crate::api::models::ListWorkflowsResponse,
        crate::api::models::CancelWorkflowResponse,
        crate::api::models::GetWorkflowOutputsResponse,
        crate::api::models::GetWorkflowLogsQuery,
        crate::api::models::GetWorkflowLogsResponse,
        crate::db::WorkflowRow,
        crate::db::WorkflowStatus,
        crate::db::WdlSourceType,
    )),
    tags(
        (name = "workflows", description = "Workflow management endpoints")
    )
)]
struct ApiDoc;

/// Create the application router.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/workflows", post(submit_workflow).get(list_workflows))
        .route("/workflows/{id}", get(get_workflow))
        .route("/workflows/{id}/cancel", post(cancel_workflow))
        .route("/workflows/{id}/outputs", get(get_workflow_outputs))
        .route("/workflows/{id}/logs", get(get_workflow_logs))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Run the server.
///
/// # Errors
///
/// Returns an error if the server fails to start or bind to the address.
pub async fn run(config: Config) -> anyhow::Result<()> {
    let db = Database::new(
        config.database.url.as_str(),
        config.database.max_connections,
    )
    .await?;

    let manager = spawn_manager(config.clone(), db);

    let state = AppState { manager };

    let app = create_router(state);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("server listening on `{}`", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

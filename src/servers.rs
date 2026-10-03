use axum::{
	Router,
	extract::State,
	http::StatusCode,
	middleware,
	response::{IntoResponse, Json, Redirect},
	routing::get,
};
use serde_json::json;
use tower_http::services::ServeDir;

use crate::{AppState, api, middlewares::owner_authenticate::OwnerUser, session_store::AuthUser};

async fn healthz_handler(State(state): State<AppState>) -> impl IntoResponse {
	match sqlx::query("SELECT 1").execute(&state.model.db).await {
		Ok(_) => (
			StatusCode::OK,
			Json(json!({ "status": "ok", "database": "connected" })),
		),
		Err(e) => {
			tracing::error!(error = %e, "health check failed: database ping error");
			(
				StatusCode::SERVICE_UNAVAILABLE,
				Json(json!({ "status": "error", "message": "database unavailable" })),
			)
		}
	}
}

/// Constructs the Admin server router.
/// Listens on `CONFIG.admin_port`.
/// Exposes `/admin` (protected by Admin auth) and `/auth` (admin login/logout).
/// Does NOT expose public or owner routes.
pub fn admin_router(state: AppState) -> Router {
	let admin_routes = Router::new()
		.nest("/admin", api::admin::router())
		.layer(middleware::from_extractor_with_state::<AuthUser, AppState>(
			state.clone(),
		))
		.nest("/auth", api::auth::admin_auth_router())
		.route("/", get(|| async { Redirect::to("/admin") }));

	let static_asset_server = ServeDir::new("static");

	Router::new()
		.route("/healthz", get(healthz_handler))
		.route("/health", get(healthz_handler))
		.merge(admin_routes)
		.with_state(state)
		.fallback_service(static_asset_server)
}

/// Constructs the Project Owner server router.
/// Listens on `CONFIG.owner_port`.
/// Exposes `/owner` (protected by Owner auth) and `/auth` (owner login/logout).
/// Does NOT expose public or admin routes.
pub fn owner_router(state: AppState) -> Router {
	let owner_routes = Router::new()
		.nest("/owner", api::owners::router())
		.layer(middleware::from_extractor_with_state::<OwnerUser, AppState>(state.clone()))
		.nest("/auth", api::auth::owner_auth_router())
		.route("/", get(|| async { Redirect::to("/owner") }));

	let static_asset_server = ServeDir::new("static");

	Router::new()
		.route("/healthz", get(healthz_handler))
		.route("/health", get(healthz_handler))
		.merge(owner_routes)
		.with_state(state)
		.fallback_service(static_asset_server)
}

/// Constructs the Public server router.
/// Listens on `CONFIG.public_port`.
/// Exposes public pages: `/`, `/projects`, `/owners`, `/properties`, `/sitemap.xml`, etc.
/// Does NOT expose any admin or owner management routes.
pub fn public_router(state: AppState) -> Router {
	let public_routes = api::public::router();
	let static_asset_server = ServeDir::new("static");

	Router::new()
		.route("/healthz", get(healthz_handler))
		.route("/health", get(healthz_handler))
		.merge(public_routes)
		.with_state(state)
		.fallback_service(static_asset_server)
}

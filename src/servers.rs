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

/// Constructs the unified multiplexed router.
/// Serves all 3 front-ends on a single domain / port:
/// - Public site at `/`
/// - Admin portal at `/admin` (protected by Admin auth)
/// - Owner portal at `/owner` (protected by Owner auth)
/// - Unified Auth at `/auth` (admin & owner login/logout)
pub fn multiplexed_router(state: AppState) -> Router {
	let admin_routes = Router::new()
		.nest("/admin", api::admin::router())
		.layer(middleware::from_extractor_with_state::<AuthUser, AppState>(
			state.clone(),
		))
		.route("/admin/login", get(|| async { Redirect::to("/auth/admin-login") }));

	let owner_routes = Router::new()
		.nest("/owner", api::owners::router())
		.layer(middleware::from_extractor_with_state::<OwnerUser, AppState>(
			state.clone(),
		))
		.route("/owner/login", get(|| async { Redirect::to("/auth/owner-login") }));

	let auth_routes = Router::new()
		.nest("/auth", api::auth::router())
		.route("/login", get(|| async { Redirect::to("/auth/admin-login") }));

	let public_routes = api::public::router();
	let static_asset_server = ServeDir::new("static");

	Router::new()
		.route("/healthz", get(healthz_handler))
		.route("/health", get(healthz_handler))
		.merge(admin_routes)
		.merge(owner_routes)
		.merge(auth_routes)
		.merge(public_routes)
		.with_state(state)
		.fallback_service(static_asset_server)
}

/// Constructs the Admin server router for port-isolated mode.
/// Listens on `CONFIG.admin_port`.
/// Exposes admin routes directly at `/` without requiring the `/admin` prefix,
/// while also supporting `/admin/*` for backwards compatibility.
pub fn admin_router(state: AppState) -> Router {
	let admin_core = api::admin::router()
		.layer(middleware::from_extractor_with_state::<AuthUser, AppState>(
			state.clone(),
		));

	let admin_routes = Router::new()
		.merge(admin_core.clone())
		.nest("/admin", admin_core)
		.nest("/auth", api::auth::admin_auth_router())
		.route("/login", get(|| async { Redirect::to("/auth/admin-login") }));

	let static_asset_server = ServeDir::new("static");

	Router::new()
		.route("/healthz", get(healthz_handler))
		.route("/health", get(healthz_handler))
		.merge(admin_routes)
		.with_state(state)
		.fallback_service(static_asset_server)
}

/// Constructs the Project Owner server router for port-isolated mode.
/// Listens on `CONFIG.owner_port`.
/// Exposes owner routes directly at `/` without requiring the `/owner` prefix,
/// while also supporting `/owner/*` for backwards compatibility.
pub fn owner_router(state: AppState) -> Router {
	let owner_core = api::owners::router()
		.layer(middleware::from_extractor_with_state::<OwnerUser, AppState>(
			state.clone(),
		));

	let owner_routes = Router::new()
		.merge(owner_core.clone())
		.nest("/owner", owner_core)
		.nest("/auth", api::auth::owner_auth_router())
		.route("/login", get(|| async { Redirect::to("/auth/owner-login") }));

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

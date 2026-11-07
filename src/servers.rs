use axum::{
	Router,
	extract::{DefaultBodyLimit, State},
	http::{Method, StatusCode, header},
	middleware,
	response::{IntoResponse, Json},
	routing::get,
};
use serde_json::json;
use tower_http::{
	cors::{AllowOrigin, CorsLayer},
	services::ServeDir,
};

use crate::{
	AppState, api, config::CONFIG, middlewares::owner_authenticate::OwnerUser,
	session_store::AuthUser,
};

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

/// Constructs the single unified router for MyPlace:
/// - Public site served at `/`
/// - Unconditional JSON API served at `/api`:
///     - `/api/auth`: Login, logout, /me
///     - `/api/admin`: Admin JSON API (protected by Admin auth)
///     - `/api/owner`: Owner JSON API (protected by Owner auth)
/// - Static asset fallback
pub fn main_router(state: AppState) -> Router {
	let cors = CorsLayer::new()
		.allow_origin(AllowOrigin::predicate({
			let allowed = CONFIG.cors_allowed_origins.clone();
			move |origin: &axum::http::HeaderValue, _| {
				if let Ok(origin_str) = origin.to_str() {
					let clean = origin_str.trim_end_matches('/');
					allowed.iter().any(|a| a.trim_end_matches('/') == clean)
						|| clean.starts_with("http://localhost:")
						|| clean.starts_with("http://127.0.0.1:")
						|| clean.ends_with(".github.io")
						|| clean.ends_with(".pages.dev")
						|| clean.ends_with(".workers.dev")
				} else {
					false
				}
			}
		}))
		.allow_methods([
			Method::GET,
			Method::POST,
			Method::PUT,
			Method::DELETE,
			Method::OPTIONS,
		])
		.allow_headers([
			header::AUTHORIZATION,
			header::CONTENT_TYPE,
			header::ACCEPT,
			header::COOKIE,
		])
		.allow_credentials(true);

	let admin_api = api::admin::router().layer(
		middleware::from_extractor_with_state::<AuthUser, AppState>(state.clone()),
	);

	let owner_api = api::owners::router().layer(
		middleware::from_extractor_with_state::<OwnerUser, AppState>(state.clone()),
	);

	let api_router = Router::new()
		.nest("/auth", api::auth::router())
		.nest("/admin", admin_api)
		.nest("/owner", owner_api)
		.layer(DefaultBodyLimit::max(100 * 1024 * 1024))
		.layer(cors);

	let public_routes = api::public::router();
	let static_asset_server = ServeDir::new("static");

	Router::new()
		.route("/healthz", get(healthz_handler))
		.route("/health", get(healthz_handler))
		.nest("/api", api_router)
		.merge(public_routes)
		.with_state(state)
		.fallback_service(static_asset_server)
}

/// Backwards compatibility alias for tests
pub fn public_router(state: AppState) -> Router {
	main_router(state)
}

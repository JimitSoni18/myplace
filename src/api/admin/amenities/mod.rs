use axum::{
	Json, Router,
	extract::{Path, State},
	http::StatusCode,
	routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{AppState, session_store::AuthUser, utils::sql::slugify};

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/", get(list_amenities).post(create_amenity))
		.route("/{id}/toggle", post(toggle_amenity))
		.route("/{id}", axum::routing::delete(delete_amenity))
}

#[derive(Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "admin/")]
pub struct AmenityItem {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub is_active: bool,
	pub projects_count: i64,
}

#[derive(Deserialize, ts_rs::TS)]
#[ts(export, export_to = "admin/")]
pub struct CreateAmenityPayload {
	pub name: String,
}

async fn list_amenities(
	State(state): State<AppState>,
	_auth_user: AuthUser,
) -> Result<Json<Vec<AmenityItem>>, (StatusCode, Json<serde_json::Value>)> {
	struct DbRow {
		id: i32,
		name: String,
		slug: String,
		is_active: bool,
		projects_count: i64,
	}

	let rows = sqlx::query_file_as!(DbRow, "queries/amenities/list_amenities.sql")
		.fetch_all(&state.model.db)
		.await
		.map_err(|e| {
			tracing::error!(error = ?e, "failed to fetch amenities");
			(
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(json!({ "code": "INTERNAL_ERROR", "message": "Failed to fetch amenities" })),
			)
		})?;

	let amenities = rows
		.into_iter()
		.map(|r| AmenityItem {
			id: r.id,
			name: r.name,
			slug: r.slug,
			is_active: r.is_active,
			projects_count: r.projects_count,
		})
		.collect();

	Ok(Json(amenities))
}

async fn create_amenity(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Json(payload): Json<CreateAmenityPayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
	let trimmed = payload.name.trim();
	if trimmed.is_empty() {
		return Err((
			StatusCode::BAD_REQUEST,
			Json(json!({ "code": "VALIDATION_ERROR", "message": "Amenity name is required" })),
		));
	}

	let slug = slugify(trimmed);

	sqlx::query_file!("queries/amenities/create_amenity.sql", trimmed, slug)
		.execute(&state.model.db)
		.await
		.map_err(|e| {
			tracing::error!(error = ?e, "failed to insert amenity");
			(
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(json!({ "code": "INTERNAL_ERROR", "message": "Failed to insert amenity" })),
			)
		})?;

	tracing::info!(name = trimmed, slug = %slug, "created amenity");
	Ok((
		StatusCode::CREATED,
		Json(json!({ "success": true, "name": trimmed, "slug": slug })),
	))
}

async fn toggle_amenity(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
	sqlx::query_file!("queries/amenities/toggle_amenity.sql", id)
		.execute(&state.model.db)
		.await
		.map_err(|e| {
			tracing::error!(error = ?e, "failed to toggle amenity");
			(
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(json!({ "code": "INTERNAL_ERROR", "message": "Failed to toggle amenity" })),
			)
		})?;

	Ok(Json(json!({ "success": true, "id": id })))
}

async fn delete_amenity(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
	sqlx::query_file!("queries/amenities/delete_amenity.sql", id)
		.execute(&state.model.db)
		.await
		.map_err(|e| {
			tracing::error!(error = ?e, "failed to delete amenity");
			(
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(json!({ "code": "INTERNAL_ERROR", "message": "Failed to delete amenity" })),
			)
		})?;

	Ok(StatusCode::NO_CONTENT)
}

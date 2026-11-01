use axum::{
	Json, Router,
	extract::{Path, Query, State},
	http::StatusCode,
	routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{AppState, session_store::AuthUser, utils};

#[derive(Serialize, Deserialize, Clone, ts_rs::TS)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct LocationItem {
	pub id: i32,
	pub formatted_address: String,
	pub city: Option<String>,
	pub state_or_province: Option<String>,
	pub area: Option<String>,
}

#[derive(Deserialize, ts_rs::TS)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct CreateOrUpdateLocationPayload {
	pub formatted_address: String,
	pub city: Option<String>,
	pub state_or_province: Option<String>,
	pub area: Option<String>,
}

#[derive(Deserialize, ts_rs::TS)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct LocationQuery {
	#[serde(default)]
	pub q: Option<String>,
}

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/", get(list_locations).post(create_location))
		.route("/{id}", get(get_location).put(update_location).delete(delete_location))
}

async fn list_locations(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Query(query): Query<LocationQuery>,
) -> Result<Json<Vec<LocationItem>>, (StatusCode, Json<serde_json::Value>)> {
	let locations = match query.q.filter(|s| !s.trim().is_empty()) {
		Some(ref q) => {
			let escaped = utils::sql::escape(q.trim());
			let pattern = format!("%{escaped}%");
			sqlx::query_file_as!(
				LocationItem,
				"queries/locations/search_locations.sql",
				pattern
			)
			.fetch_all(&state.model.db)
			.await
		}
		None => {
			sqlx::query_file_as!(LocationItem, "queries/locations/list_locations.sql")
				.fetch_all(&state.model.db)
				.await
		}
	}
	.map_err(|e| {
		tracing::error!(error = ?e, "failed to fetch locations");
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(json!({ "code": "INTERNAL_ERROR", "message": "Failed to fetch locations" })),
		)
	})?;

	Ok(Json(locations))
}

async fn get_location(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
) -> Result<Json<LocationItem>, (StatusCode, Json<serde_json::Value>)> {
	let location = sqlx::query_file_as!(
		LocationItem,
		"queries/locations/get_location_by_id.sql",
		id
	)
	.fetch_one(&state.model.db)
	.await
	.map_err(|e| match e {
		sqlx::Error::RowNotFound => (
			StatusCode::NOT_FOUND,
			Json(json!({ "code": "NOT_FOUND", "message": "Location not found" })),
		),
		_ => {
			tracing::error!(error = ?e, "failed to get location");
			(
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(json!({ "code": "INTERNAL_ERROR", "message": "Database query error" })),
			)
		}
	})?;

	Ok(Json(location))
}

async fn create_location(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Json(payload): Json<CreateOrUpdateLocationPayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
	let address = payload.formatted_address.trim();
	if address.is_empty() {
		return Err((
			StatusCode::BAD_REQUEST,
			Json(json!({ "code": "VALIDATION_ERROR", "message": "Address is required" })),
		));
	}

	let result = sqlx::query_file!(
		"queries/locations/create_location.sql",
		address,
		payload.city.as_deref().map(|s| s.trim()),
		payload.state_or_province.as_deref().map(|s| s.trim()),
		payload.area.as_deref().map(|s| s.trim()),
	)
	.execute(&state.model.db)
	.await
	.map_err(|e| {
		tracing::error!(error = ?e, "failed to create location");
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(json!({ "code": "INTERNAL_ERROR", "message": "Failed to create location" })),
		)
	})?;

	Ok((
		StatusCode::CREATED,
		Json(json!({ "success": true, "rows_affected": result.rows_affected() })),
	))
}

async fn update_location(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
	Json(payload): Json<CreateOrUpdateLocationPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
	let address = payload.formatted_address.trim();
	if address.is_empty() {
		return Err((
			StatusCode::BAD_REQUEST,
			Json(json!({ "code": "VALIDATION_ERROR", "message": "Address is required" })),
		));
	}

	sqlx::query_file!(
		"queries/locations/update_location.sql",
		address,
		payload.city.as_deref().map(|s| s.trim()),
		payload.state_or_province.as_deref().map(|s| s.trim()),
		payload.area.as_deref().map(|s| s.trim()),
		id,
	)
	.execute(&state.model.db)
	.await
	.map_err(|e| {
		tracing::error!(error = ?e, "failed to update location");
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(json!({ "code": "INTERNAL_ERROR", "message": "Failed to update location" })),
		)
	})?;

	Ok(Json(json!({ "success": true, "message": "Location updated" })))
}

async fn delete_location(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
	sqlx::query_file!("queries/locations/delete_location.sql", id)
		.execute(&state.model.db)
		.await
		.map_err(|e| {
			tracing::error!(error = ?e, "failed to delete location");
			(
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(json!({ "code": "INTERNAL_ERROR", "message": "Failed to delete location" })),
			)
		})?;

	Ok(StatusCode::NO_CONTENT)
}

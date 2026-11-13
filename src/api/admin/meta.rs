use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;

use crate::{AppState, session_store::AuthUser};

pub fn router() -> Router<AppState> {
	Router::new().route("/options", get(get_meta_options))
}

#[derive(Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, export_to = "admin/")]
pub struct LookupItem {
	pub id: i32,
	pub name: String,
}

#[derive(Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, export_to = "admin/")]
pub struct MetaOptionsResponse {
	pub owners: Vec<LookupItem>,
	pub project_types: Vec<LookupItem>,
	pub amenities: Vec<LookupItem>,
	pub cities: Vec<String>,
	pub areas: Vec<String>,
	pub states: Vec<String>,
}

async fn get_meta_options(
	State(state): State<AppState>,
	_auth_user: AuthUser,
) -> Json<MetaOptionsResponse> {
	let db = &state.model.db;

	#[derive(sqlx::FromRow)]
	struct IdNameRow {
		id: i32,
		name: String,
	}

	let owners = sqlx::query_as::<_, IdNameRow>(
		"SELECT id, name FROM project_owners WHERE deleted_at IS NULL ORDER BY name ASC",
	)
	.fetch_all(db)
	.await
	.unwrap_or_default()
	.into_iter()
	.map(|o| LookupItem { id: o.id, name: o.name })
	.collect();

	let project_types = sqlx::query_as::<_, IdNameRow>(
		"SELECT id, name FROM project_types WHERE is_active = TRUE ORDER BY id ASC",
	)
	.fetch_all(db)
	.await
	.unwrap_or_default()
	.into_iter()
	.map(|pt| LookupItem { id: pt.id, name: pt.name })
	.collect();

	let amenities = sqlx::query_as::<_, IdNameRow>(
		"SELECT id, name FROM amenities WHERE is_active = TRUE ORDER BY name ASC",
	)
	.fetch_all(db)
	.await
	.unwrap_or_default()
	.into_iter()
	.map(|a| LookupItem { id: a.id, name: a.name })
	.collect();

	#[derive(sqlx::FromRow)]
	struct TextRow {
		val: String,
	}

	let cities = sqlx::query_as::<_, TextRow>(
		"SELECT DISTINCT city AS val FROM projects WHERE city != '' AND deleted_at IS NULL ORDER BY city ASC",
	)
	.fetch_all(db)
	.await
	.unwrap_or_default()
	.into_iter()
	.map(|r| r.val)
	.collect();

	let areas = sqlx::query_as::<_, TextRow>(
		"SELECT DISTINCT area AS val FROM projects WHERE area != '' AND deleted_at IS NULL ORDER BY area ASC",
	)
	.fetch_all(db)
	.await
	.unwrap_or_default()
	.into_iter()
	.map(|r| r.val)
	.collect();

	let states = sqlx::query_as::<_, TextRow>(
		"SELECT DISTINCT state_or_province AS val FROM projects WHERE state_or_province != '' AND deleted_at IS NULL ORDER BY state_or_province ASC",
	)
	.fetch_all(db)
	.await
	.unwrap_or_default()
	.into_iter()
	.map(|r| r.val)
	.collect();

	Json(MetaOptionsResponse {
		owners,
		project_types,
		amenities,
		cities,
		areas,
		states,
	})
}

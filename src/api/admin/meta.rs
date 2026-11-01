use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;

use crate::{AppState, session_store::AuthUser};

pub fn router() -> Router<AppState> {
	Router::new().route("/options", get(get_meta_options))
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "admin/")]
pub struct LookupItem {
	pub id: i32,
	pub name: String,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "admin/")]
pub struct LocationLookupItem {
	pub id: i32,
	pub formatted_address: String,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "admin/")]
pub struct PropertyTypeLookupItem {
	pub id: i32,
	pub name: String,
	pub category: String,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "admin/")]
pub struct MetaOptionsResponse {
	pub owners: Vec<LookupItem>,
	pub locations: Vec<LocationLookupItem>,
	pub amenities: Vec<LookupItem>,
	pub property_types: Vec<PropertyTypeLookupItem>,
}

async fn get_meta_options(
	State(state): State<AppState>,
	_auth_user: AuthUser,
) -> Json<MetaOptionsResponse> {
	let db = &state.model.db;

	#[derive(sqlx::FromRow)]
	struct OwnerRow {
		id: i32,
		name: String,
	}
	let owners = sqlx::query_as::<_, OwnerRow>(
		"SELECT id, name FROM project_owners WHERE deleted_at IS NULL ORDER BY name ASC",
	)
	.fetch_all(db)
	.await
	.unwrap_or_default()
	.into_iter()
	.map(|o| LookupItem { id: o.id, name: o.name })
	.collect();

	#[derive(sqlx::FromRow)]
	struct LocRow {
		id: i32,
		formatted_address: String,
	}
	let locations = sqlx::query_as::<_, LocRow>(
		"SELECT id, formatted_address FROM locations WHERE deleted_at IS NULL ORDER BY formatted_address ASC",
	)
	.fetch_all(db)
	.await
	.unwrap_or_default()
	.into_iter()
	.map(|l| LocationLookupItem { id: l.id, formatted_address: l.formatted_address })
	.collect();

	#[derive(sqlx::FromRow)]
	struct AmenityRow {
		id: i32,
		name: String,
	}
	let amenities = sqlx::query_as::<_, AmenityRow>(
		"SELECT id, name FROM amenities WHERE is_active = true ORDER BY name ASC",
	)
	.fetch_all(db)
	.await
	.unwrap_or_default()
	.into_iter()
	.map(|a| LookupItem { id: a.id, name: a.name })
	.collect();

	#[derive(sqlx::FromRow)]
	struct PropTypeRow {
		id: i32,
		name: String,
		category: String,
	}
	let property_types = sqlx::query_as::<_, PropTypeRow>(
		"SELECT id, name, category FROM property_types ORDER BY category, name ASC",
	)
	.fetch_all(db)
	.await
	.unwrap_or_default()
	.into_iter()
	.map(|pt| PropertyTypeLookupItem { id: pt.id, name: pt.name, category: pt.category })
	.collect();

	Json(MetaOptionsResponse {
		owners,
		locations,
		amenities,
		property_types,
	})
}

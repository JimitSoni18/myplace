use axum::{
	Json, Router,
	extract::State,
	routing::get,
};
use serde::Serialize;

use crate::{AppState, session_store::AuthUser};

pub fn router() -> Router<AppState> {
	Router::new().route("/", get(get_dashboard))
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "admin/")]
pub struct DashboardRecentProject {
	pub id: i32,
	pub name: String,
	pub owner_name: String,
	pub location: String,
	pub category: String,
	pub total_properties: i64,
	pub available_properties: i64,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "admin/")]
pub struct DashboardStats {
	pub total_owners: i64,
	pub total_projects: i64,
	pub total_properties: i64,
	pub total_residential: i64,
	pub total_commercial: i64,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "admin/")]
pub struct DashboardResponse {
	pub admin_name: String,
	pub stats: DashboardStats,
	pub recent_projects: Vec<DashboardRecentProject>,
}

async fn get_dashboard(
	State(state): State<AppState>,
	auth_user: AuthUser,
) -> Json<DashboardResponse> {
	let db = &state.model.db;

	let total_owners: i64 = sqlx::query_file_scalar!("queries/dashboard/count_owners.sql")
		.fetch_one(db)
		.await
		.unwrap_or(Some(0))
		.unwrap_or(0);

	let total_projects: i64 = sqlx::query_file_scalar!("queries/dashboard/count_projects.sql")
		.fetch_one(db)
		.await
		.unwrap_or(Some(0))
		.unwrap_or(0);

	let total_properties: i64 = sqlx::query_file_scalar!("queries/dashboard/count_properties.sql")
		.fetch_one(db)
		.await
		.unwrap_or(Some(0))
		.unwrap_or(0);

	let total_residential: i64 =
		sqlx::query_file_scalar!("queries/dashboard/count_residential.sql")
			.fetch_one(db)
			.await
			.unwrap_or(Some(0))
			.unwrap_or(0);

	let total_commercial: i64 = sqlx::query_file_scalar!("queries/dashboard/count_commercial.sql")
		.fetch_one(db)
		.await
		.unwrap_or(Some(0))
		.unwrap_or(0);

	struct RecentProjectRow {
		id: i32,
		name: String,
		owner_name: String,
		location: String,
		category: String,
		total_properties: Option<i64>,
		available_properties: Option<i64>,
	}

	let recent_raw =
		sqlx::query_file_as!(RecentProjectRow, "queries/dashboard/recent_projects.sql")
			.fetch_all(db)
			.await
			.unwrap_or_default();

	let recent_projects: Vec<DashboardRecentProject> = recent_raw
		.into_iter()
		.map(|r| DashboardRecentProject {
			id: r.id,
			name: r.name,
			owner_name: r.owner_name,
			location: r.location,
			category: r.category,
			total_properties: r.total_properties.unwrap_or(0),
			available_properties: r.available_properties.unwrap_or(0),
		})
		.collect();

	Json(DashboardResponse {
		admin_name: auth_user.username,
		stats: DashboardStats {
			total_owners,
			total_projects,
			total_properties,
			total_residential,
			total_commercial,
		},
		recent_projects,
	})
}

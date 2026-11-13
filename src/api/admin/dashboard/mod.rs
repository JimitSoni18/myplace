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
	pub city: String,
	pub area: String,
	pub project_type_name: String,
	pub configurations_count: i64,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "admin/")]
pub struct DashboardStats {
	pub total_owners: i64,
	pub total_projects: i64,
	pub total_configurations: i64,
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

	let total_configurations: i64 = sqlx::query_file_scalar!("queries/dashboard/count_configurations.sql")
		.fetch_one(db)
		.await
		.unwrap_or(0);

	struct RecentProjectRow {
		id: i32,
		name: String,
		owner_name: String,
		city: String,
		area: String,
		project_type_name: String,
		configurations_count: Option<i64>,
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
			city: r.city,
			area: r.area,
			project_type_name: r.project_type_name,
			configurations_count: r.configurations_count.unwrap_or(0),
		})
		.collect();

	Json(DashboardResponse {
		admin_name: auth_user.username,
		stats: DashboardStats {
			total_owners,
			total_projects,
			total_configurations,
		},
		recent_projects,
	})
}

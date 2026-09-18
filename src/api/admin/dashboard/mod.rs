use askama::Template;
use axum::{
	Extension, Router,
	extract::State,
	response::{Html, IntoResponse},
	routing::get,
};

use crate::{
	AppState,
	session_store::AuthUser,
	templates::admin::{AdminPage, DashboardRecentProject, DashboardTemplate},
};

pub fn router() -> Router<AppState> {
	Router::new().route("/", get(get_dashboard))
}

async fn get_dashboard(
	State(state): State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
) -> impl IntoResponse {
	let db = &state.model.db;

	// --- Aggregate stats --- (single query each, indexed columns)
	let total_owners: i64 = sqlx::query_scalar!(
		"SELECT COUNT(*) FROM project_owners WHERE deleted_at IS NULL"
	)
	.fetch_one(db)
	.await
	.unwrap_or(Some(0))
	.unwrap_or(0);

	let total_projects: i64 =
		sqlx::query_scalar!("SELECT COUNT(*) FROM projects WHERE deleted_at IS NULL")
			.fetch_one(db)
			.await
			.unwrap_or(Some(0))
			.unwrap_or(0);

	let total_properties: i64 =
		sqlx::query_scalar!("SELECT COUNT(*) FROM properties WHERE deleted_at IS NULL")
			.fetch_one(db)
			.await
			.unwrap_or(Some(0))
			.unwrap_or(0);

	// Category breakdown joins property_types to get the category string.
	let total_residential: i64 = sqlx::query_scalar!(
		r#"SELECT COUNT(pr.id)
		   FROM properties pr
		   JOIN property_types pt ON pt.id = pr.property_type_id
		   WHERE pr.deleted_at IS NULL AND pt.category = 'RESIDENTIAL'"#
	)
	.fetch_one(db)
	.await
	.unwrap_or(Some(0))
	.unwrap_or(0);

	let total_commercial: i64 = sqlx::query_scalar!(
		r#"SELECT COUNT(pr.id)
		   FROM properties pr
		   JOIN property_types pt ON pt.id = pr.property_type_id
		   WHERE pr.deleted_at IS NULL AND pt.category = 'COMMERCIAL'"#
	)
	.fetch_one(db)
	.await
	.unwrap_or(Some(0))
	.unwrap_or(0);

	// --- Recent projects (created in last 30 days) ---
	struct RecentProjectRow {
		id: i32,
		name: String,
		owner_name: String,
		location: String,
		category: String,
		total_properties: Option<i64>,
		available_properties: Option<i64>,
	}

	let recent_raw = sqlx::query_as!(
		RecentProjectRow,
		r#"
		SELECT
			proj.id,
			proj.name,
			po.name   AS owner_name,
			loc.formatted_address AS location,
			proj.category,
			COUNT(DISTINCT pr.id)                                                   AS "total_properties: i64",
			COUNT(DISTINCT pl.id) FILTER (WHERE pl.status = 'active')               AS "available_properties: i64"
		FROM projects proj
		JOIN project_owners po ON po.id = proj.project_owner_id
		JOIN locations     loc ON loc.id = proj.location_id
		LEFT JOIN properties pr ON pr.project_id = proj.id AND pr.deleted_at IS NULL
		LEFT JOIN property_listings pl ON pl.property_id = pr.id
		WHERE proj.deleted_at IS NULL
		  AND proj.created_at >= NOW() - INTERVAL '30 days'
		GROUP BY proj.id, po.name, loc.formatted_address
		ORDER BY proj.created_at DESC
		LIMIT 20
		"#
	)
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

	let html = DashboardTemplate {
		admin_name: &auth_user.username,
		page: AdminPage::Dashboard,
		total_owners,
		total_projects,
		total_properties,
		total_residential,
		total_commercial,
		recent_projects,
	}
	.render()
	.unwrap_or_else(|e| {
		tracing::error!(?e, "failed to render dashboard");
		"Internal error".to_string()
	});

	Html(html)
}

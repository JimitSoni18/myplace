use axum::{
	Json, Router,
	extract::{Path, State},
	http::StatusCode,
	routing::get,
};
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

use crate::{
	AppState,
	config::CONFIG,
	middlewares::owner_authenticate::OwnerUser,
};

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/dashboard", get(owner_dashboard))
		.route("/projects", get(owner_projects_list))
		.route("/projects/{id}", get(owner_project_detail))
		.route("/projects/{id}/properties", get(owner_project_properties))
		.nest("/notifications", crate::api::notifications::owner_router())
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "owner/", optional_fields = nullable)]
pub struct OwnerDashboardProject {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub category: String,
	pub location: String,
	pub properties_count: i64,
	pub hero_thumb_url: Option<String>,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "owner/")]
pub struct OwnerDashboardStats {
	pub total_projects: i64,
	pub total_properties: i64,
	pub total_residential: i64,
	pub total_commercial: i64,
	pub total_land: i64,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "owner/")]
pub struct OwnerDashboardResponse {
	pub owner_name: String,
	pub username: String,
	pub stats: OwnerDashboardStats,
	pub projects: Vec<OwnerDashboardProject>,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "owner/")]
pub struct OwnerProjectMediaItem {
	pub id: Uuid,
	pub media_type: String,
	pub url: String,
	pub thumbnail_url: String,
	pub sequence: i16,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "owner/", optional_fields = nullable)]
pub struct OwnerProjectDocumentItem {
	pub id: Uuid,
	pub file_name: String,
	pub url: String,
	pub file_size: Option<i64>,
	pub doc_type: String,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "owner/", optional_fields = nullable)]
pub struct OwnerProjectDetailResponse {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub category: String,
	pub location: String,
	pub description: Option<String>,
	pub start_date: Option<String>,
	pub launch_date: Option<String>,
	pub possession_date: Option<String>,
	pub properties_count: i64,
	pub media: Vec<OwnerProjectMediaItem>,
	pub documents: Vec<OwnerProjectDocumentItem>,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "owner/", optional_fields = nullable)]
pub struct OwnerPropertyItem {
	pub id: i32,
	pub unit_number: Option<String>,
	pub building: Option<String>,
	pub category: String,
	pub type_name: String,
	pub price: f64,
	pub status: String,
	pub built_up_area: Option<f64>,
}

fn format_opt_date(d: Option<time::Date>) -> Option<String> {
	let fmt = time::macros::format_description!("[year]-[month]-[day]");
	d.and_then(|date| date.format(&fmt).ok())
}

async fn owner_dashboard(
	owner: OwnerUser,
	State(state): State<AppState>,
) -> Result<Json<OwnerDashboardResponse>, (StatusCode, Json<serde_json::Value>)> {
	struct DbRow {
		id: i32,
		name: String,
		slug: String,
		category: String,
		location: Option<String>,
		properties_count: i64,
		hero_thumb_key: Option<String>,
	}

	let rows = sqlx::query_file_as!(
		DbRow,
		"queries/projects/list_owner_portal_projects.sql",
		owner.owner_id
	)
	.fetch_all(&state.model.db)
	.await
	.map_err(|e| {
		tracing::error!(error = ?e, "failed to fetch owner projects");
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(json!({ "code": "INTERNAL_ERROR", "message": "Failed to fetch dashboard data" })),
		)
	})?;

	let total_projects = rows.len() as i64;
	let total_properties: i64 = rows.iter().map(|r| r.properties_count).sum();
	let total_residential: i64 = rows
		.iter()
		.filter(|r| r.category.eq_ignore_ascii_case("RESIDENTIAL"))
		.map(|r| r.properties_count)
		.sum();
	let total_commercial: i64 = rows
		.iter()
		.filter(|r| r.category.eq_ignore_ascii_case("COMMERCIAL"))
		.map(|r| r.properties_count)
		.sum();
	let total_land: i64 = rows
		.iter()
		.filter(|r| r.category.eq_ignore_ascii_case("LAND"))
		.map(|r| r.properties_count)
		.sum();

	let projects: Vec<OwnerDashboardProject> = rows
		.into_iter()
		.map(|r| {
			let hero_thumb_url = r.hero_thumb_key.as_ref().map(|k| CONFIG.asset_url(k));
			OwnerDashboardProject {
				id: r.id,
				name: r.name,
				slug: r.slug,
				category: r.category,
				location: r.location.unwrap_or_else(|| "Unspecified".to_string()),
				properties_count: r.properties_count,
				hero_thumb_url,
			}
		})
		.collect();

	Ok(Json(OwnerDashboardResponse {
		owner_name: owner.owner_name,
		username: owner.username,
		stats: OwnerDashboardStats {
			total_projects,
			total_properties,
			total_residential,
			total_commercial,
			total_land,
		},
		projects,
	}))
}

async fn owner_projects_list(
	owner: OwnerUser,
	State(state): State<AppState>,
) -> Result<Json<Vec<OwnerDashboardProject>>, (StatusCode, Json<serde_json::Value>)> {
	struct DbRow {
		id: i32,
		name: String,
		slug: String,
		category: String,
		location: Option<String>,
		properties_count: i64,
		hero_thumb_key: Option<String>,
	}

	let rows = sqlx::query_file_as!(
		DbRow,
		"queries/projects/list_owner_portal_projects.sql",
		owner.owner_id
	)
	.fetch_all(&state.model.db)
	.await
	.map_err(|e| {
		tracing::error!(error = ?e, "failed to fetch owner project list");
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(json!({ "code": "INTERNAL_ERROR", "message": "Failed to fetch projects" })),
		)
	})?;

	let projects = rows
		.into_iter()
		.map(|r| {
			let hero_thumb_url = r.hero_thumb_key.as_ref().map(|k| CONFIG.asset_url(k));
			OwnerDashboardProject {
				id: r.id,
				name: r.name,
				slug: r.slug,
				category: r.category,
				location: r.location.unwrap_or_else(|| "Unspecified".to_string()),
				properties_count: r.properties_count,
				hero_thumb_url,
			}
		})
		.collect();

	Ok(Json(projects))
}

async fn owner_project_detail(
	owner: OwnerUser,
	State(state): State<AppState>,
	Path(id): Path<i32>,
) -> Result<Json<OwnerProjectDetailResponse>, (StatusCode, Json<serde_json::Value>)> {
	struct DbProject {
		id: i32,
		name: String,
		slug: String,
		description: Option<String>,
		category: String,
		location_name: Option<String>,
		start_date: Option<time::Date>,
		launch_date: Option<time::Date>,
		possession_date: Option<time::Date>,
	}

	let p = sqlx::query_file_as!(
		DbProject,
		"queries/projects/get_owner_portal_project_detail.sql",
		id,
		owner.owner_id
	)
	.fetch_optional(&state.model.db)
	.await
	.map_err(|e| {
		tracing::error!(error = ?e, "db error fetching project");
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(json!({ "code": "DATABASE_ERROR", "message": "Database query error" })),
		)
	})?
	.ok_or_else(|| {
		(
			StatusCode::NOT_FOUND,
			Json(json!({ "code": "NOT_FOUND", "message": "Project not found or not owned by you" })),
		)
	})?;

	let count_row = sqlx::query_file!("queries/projects/count_project_properties.sql", id)
		.fetch_one(&state.model.db)
		.await
		.map_err(|_| {
			(
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(json!({ "code": "DATABASE_ERROR", "message": "Failed to count properties" })),
			)
		})?;

	struct DbMedia {
		media_id: Uuid,
		media_type: String,
		s3_key: String,
		thumbnail_key: Option<String>,
		sequence: i16,
	}

	let media_rows = sqlx::query_file_as!(DbMedia, "queries/projects/get_project_media.sql", id)
		.fetch_all(&state.model.db)
		.await
		.unwrap_or_default();

	let media = media_rows
		.into_iter()
		.map(|r| {
			let url = CONFIG.asset_url(&r.s3_key);
			let thumbnail_url = r
				.thumbnail_key
				.as_ref()
				.map(|k| CONFIG.asset_url(k))
				.unwrap_or_else(|| url.clone());
			OwnerProjectMediaItem {
				id: r.media_id,
				media_type: r.media_type,
				url,
				thumbnail_url,
				sequence: r.sequence,
			}
		})
		.collect();

	#[allow(dead_code)]
	struct DbDoc {
		id: i32,
		media_id: Uuid,
		display_name: Option<String>,
		doc_type: Option<String>,
		s3_key: String,
		file_size: Option<i64>,
	}

	let doc_rows = sqlx::query_file_as!(DbDoc, "queries/projects/get_project_documents.sql", id)
		.fetch_all(&state.model.db)
		.await
		.unwrap_or_default();

	let documents = doc_rows
		.into_iter()
		.map(|r| {
			let url = CONFIG.asset_url(&r.s3_key);
			let file_name = r
				.display_name
				.unwrap_or_else(|| r.s3_key.split('/').last().unwrap_or("document").to_string());
			OwnerProjectDocumentItem {
				id: r.media_id,
				file_name,
				url,
				file_size: r.file_size,
				doc_type: r.doc_type.unwrap_or_else(|| "brochure".to_string()),
			}
		})
		.collect();

	Ok(Json(OwnerProjectDetailResponse {
		id: p.id,
		name: p.name,
		slug: p.slug,
		category: p.category,
		location: p.location_name.unwrap_or_else(|| "Unspecified".to_string()),
		description: p.description,
		start_date: format_opt_date(p.start_date),
		launch_date: format_opt_date(p.launch_date),
		possession_date: format_opt_date(p.possession_date),
		properties_count: count_row.count,
		media,
		documents,
	}))
}

async fn owner_project_properties(
	owner: OwnerUser,
	State(state): State<AppState>,
	Path(id): Path<i32>,
) -> Result<Json<Vec<OwnerPropertyItem>>, (StatusCode, Json<serde_json::Value>)> {
	// Verify project ownership
	let is_owned = sqlx::query_scalar::<_, i32>(
		"SELECT id FROM projects WHERE id = $1 AND project_owner_id = $2 AND deleted_at IS NULL",
	)
	.bind(id)
	.bind(owner.owner_id)
	.fetch_optional(&state.model.db)
	.await
	.map_err(|_| {
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(json!({ "code": "DATABASE_ERROR", "message": "Failed to verify project ownership" })),
		)
	})?
	.is_some();

	if !is_owned {
		return Err((
			StatusCode::NOT_FOUND,
			Json(json!({ "code": "NOT_FOUND", "message": "Project not found or not owned by you" })),
		));
	}

	#[derive(sqlx::FromRow)]
	struct DbRow {
		id: i32,
		unit_number: Option<String>,
		building: Option<String>,
		category: String,
		type_name: String,
		price: f64,
		status: Option<String>,
		built_up_area: Option<f64>,
	}

	let rows = sqlx::query_as::<_, DbRow>(
		r#"
		SELECT p.id, p.unit_number, p.building, pt.category, pt.name as type_name,
		       COALESCE(pl.price, 0.0) as price, pl.status, p.built_up_area
		FROM properties p
		JOIN property_types pt ON pt.id = p.property_type_id
		LEFT JOIN property_listings pl ON pl.property_id = p.id
		WHERE p.project_id = $1 AND p.deleted_at IS NULL
		ORDER BY p.id DESC
		"#,
	)
	.bind(id)
	.fetch_all(&state.model.db)
	.await
	.unwrap_or_default();

	let items = rows
		.into_iter()
		.map(|r| OwnerPropertyItem {
			id: r.id,
			unit_number: r.unit_number,
			building: r.building,
			category: r.category,
			type_name: r.type_name,
			price: r.price,
			status: r.status.unwrap_or_else(|| "available".to_string()),
			built_up_area: r.built_up_area,
		})
		.collect();

	Ok(Json(items))
}

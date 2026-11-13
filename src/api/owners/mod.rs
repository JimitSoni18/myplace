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
		.nest("/notifications", crate::api::notifications::owner_router())
}

#[derive(Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, export_to = "owner/", optional_fields = nullable)]
pub struct OwnerDashboardProject {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub project_type_name: String,
	pub city: String,
	pub area: String,
	pub formatted_address: String,
	pub configurations_count: i64,
	pub hero_thumb_url: Option<String>,
}

#[derive(Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, export_to = "owner/")]
pub struct OwnerDashboardStats {
	pub total_projects: i64,
	pub total_configurations: i64,
}

#[derive(Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, export_to = "owner/")]
pub struct OwnerDashboardResponse {
	pub owner_name: String,
	pub username: String,
	pub stats: OwnerDashboardStats,
	pub projects: Vec<OwnerDashboardProject>,
}

#[derive(Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, export_to = "owner/")]
pub struct OwnerProjectMediaItem {
	pub id: Uuid,
	pub media_type: String,
	pub url: String,
	pub thumbnail_url: String,
	pub sequence: i16,
}

#[derive(Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, export_to = "owner/", optional_fields = nullable)]
pub struct OwnerProjectDocumentItem {
	pub id: Uuid,
	pub file_name: String,
	pub url: String,
	pub file_size: Option<i64>,
	pub doc_type: String,
}

#[derive(Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, export_to = "owner/", optional_fields = nullable)]
pub struct OwnerConfigurationItem {
	pub id: i32,
	pub name: String,
	pub bhk: Option<f64>,
	pub carpet_area: Option<f64>,
	pub built_up_area: Option<f64>,
	pub super_built_up_area: Option<f64>,
	pub price: Option<f64>,
	pub price_on_request: bool,
	pub floor_plan_url: Option<String>,
	pub floor_plan_thumb_url: Option<String>,
	pub sequence: i16,
}

#[derive(Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, export_to = "owner/", optional_fields = nullable)]
pub struct OwnerProjectDetailResponse {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub project_type_name: String,
	pub formatted_address: String,
	pub city: String,
	pub area: String,
	pub state_or_province: String,
	pub postal_code: Option<String>,
	pub rera_number: Option<String>,
	pub rera_authority: Option<String>,
	pub total_land_area: Option<f64>,
	pub towers_count: Option<i32>,
	pub floors_count: Option<i32>,
	pub units_count: Option<i32>,
	pub description: Option<String>,
	pub start_date: Option<String>,
	pub launch_date: Option<String>,
	pub possession_date: Option<String>,
	pub configurations_count: i64,
	pub configurations: Vec<OwnerConfigurationItem>,
	pub media: Vec<OwnerProjectMediaItem>,
	pub documents: Vec<OwnerProjectDocumentItem>,
}

fn format_opt_date(d: Option<time::Date>) -> Option<String> {
	let fmt = time::macros::format_description!("[year]-[month]-[day]");
	d.and_then(|date| date.format(&fmt).ok())
}

async fn owner_dashboard(
	State(state): State<AppState>,
	owner_user: OwnerUser,
) -> Result<Json<OwnerDashboardResponse>, (StatusCode, Json<serde_json::Value>)> {
	struct ProjectRow {
		id: i32,
		name: String,
		slug: String,
		project_type_name: String,
		city: String,
		area: String,
		formatted_address: String,
		configurations_count: i64,
		hero_thumb_key: Option<String>,
	}

	let rows = sqlx::query_file_as!(
		ProjectRow,
		"queries/projects/list_owner_portal_projects.sql",
		owner_user.owner_id
	)
	.fetch_all(&state.model.db)
	.await
	.map_err(|e| {
		tracing::error!(error = ?e, "failed to fetch owner dashboard projects");
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(json!({ "code": "INTERNAL_ERROR", "message": "Failed to fetch dashboard" })),
		)
	})?;

	let total_projects = rows.len() as i64;
	let total_configurations: i64 = rows.iter().map(|r| r.configurations_count).sum();

	let projects: Vec<OwnerDashboardProject> = rows
		.into_iter()
		.map(|r| {
			let hero_thumb_url = r.hero_thumb_key.as_ref().map(|k| CONFIG.asset_url(k));
			OwnerDashboardProject {
				id: r.id,
				name: r.name,
				slug: r.slug,
				project_type_name: r.project_type_name,
				city: r.city,
				area: r.area,
				formatted_address: r.formatted_address,
				configurations_count: r.configurations_count,
				hero_thumb_url,
			}
		})
		.collect();

	Ok(Json(OwnerDashboardResponse {
		owner_name: owner_user.owner_name,
		username: owner_user.username,
		stats: OwnerDashboardStats {
			total_projects,
			total_configurations,
		},
		projects,
	}))
}

async fn owner_projects_list(
	State(state): State<AppState>,
	owner_user: OwnerUser,
) -> Result<Json<Vec<OwnerDashboardProject>>, (StatusCode, Json<serde_json::Value>)> {
	struct ProjectRow {
		id: i32,
		name: String,
		slug: String,
		project_type_name: String,
		city: String,
		area: String,
		formatted_address: String,
		configurations_count: i64,
		hero_thumb_key: Option<String>,
	}

	let rows = sqlx::query_file_as!(
		ProjectRow,
		"queries/projects/list_owner_portal_projects.sql",
		owner_user.owner_id
	)
	.fetch_all(&state.model.db)
	.await
	.map_err(|e| {
		tracing::error!(error = ?e, "failed to fetch owner projects");
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
				project_type_name: r.project_type_name,
				city: r.city,
				area: r.area,
				formatted_address: r.formatted_address,
				configurations_count: r.configurations_count,
				hero_thumb_url,
			}
		})
		.collect();

	Ok(Json(projects))
}

async fn owner_project_detail(
	State(state): State<AppState>,
	owner_user: OwnerUser,
	Path(id): Path<i32>,
) -> Result<Json<OwnerProjectDetailResponse>, (StatusCode, Json<serde_json::Value>)> {
	let p = sqlx::query_file!(
		"queries/projects/get_owner_portal_project_detail.sql",
		id,
		owner_user.owner_id
	)
	.fetch_optional(&state.model.db)
	.await
	.map_err(|e| {
		tracing::error!(error = ?e, "failed to get owner project");
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(json!({ "code": "INTERNAL_ERROR", "message": "Failed to get project" })),
		)
	})?
	.ok_or_else(|| {
		(
			StatusCode::NOT_FOUND,
			Json(json!({ "code": "NOT_FOUND", "message": "Project not found" })),
		)
	})?;

	// Configurations
	let config_rows = sqlx::query_file!("queries/configurations/list_project_configurations.sql", id)
		.fetch_all(&state.model.db)
		.await
		.unwrap_or_default();

	let configurations: Vec<OwnerConfigurationItem> = config_rows
		.into_iter()
		.map(|c| {
			let floor_plan_url = c.floor_plan_key.as_deref().map(|k| CONFIG.asset_url(k));
			let floor_plan_thumb_url = c.floor_plan_thumb_key.as_deref().map(|k| CONFIG.asset_url(k));
			OwnerConfigurationItem {
				id: c.id,
				name: c.name,
				bhk: c.bhk,
				carpet_area: c.carpet_area,
				built_up_area: c.built_up_area,
				super_built_up_area: c.super_built_up_area,
				price: c.price,
				price_on_request: c.price_on_request,
				floor_plan_url,
				floor_plan_thumb_url,
				sequence: c.sequence,
			}
		})
		.collect();

	// Media
	struct DbMedia {
		id: Uuid,
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
		.map(|m| {
			let url = CONFIG.asset_url(&m.s3_key);
			let thumbnail_url = m
				.thumbnail_key
				.as_ref()
				.map(|k| CONFIG.asset_url(k))
				.unwrap_or_else(|| url.clone());
			OwnerProjectMediaItem {
				id: m.id,
				media_type: m.media_type,
				url,
				thumbnail_url,
				sequence: m.sequence,
			}
		})
		.collect();

	// Documents
	struct DbDoc {
		id: Uuid,
		s3_key: String,
		display_name: String,
		doc_type: String,
		file_size: Option<i64>,
	}
	let doc_rows = sqlx::query_file_as!(DbDoc, "queries/projects/get_project_documents.sql", id)
		.fetch_all(&state.model.db)
		.await
		.unwrap_or_default();

	let documents = doc_rows
		.into_iter()
		.map(|d| OwnerProjectDocumentItem {
			id: d.id,
			file_name: d.display_name,
			url: CONFIG.asset_url(&d.s3_key),
			file_size: d.file_size,
			doc_type: d.doc_type,
		})
		.collect();

	let configurations_count = configurations.len() as i64;

	Ok(Json(OwnerProjectDetailResponse {
		id: p.id,
		name: p.name,
		slug: p.slug,
		project_type_name: p.project_type_name,
		formatted_address: p.formatted_address,
		city: p.city,
		area: p.area,
		state_or_province: p.state_or_province,
		postal_code: p.postal_code,
		rera_number: p.rera_number,
		rera_authority: p.rera_authority,
		total_land_area: p.total_land_area,
		towers_count: p.towers_count,
		floors_count: p.floors_count,
		units_count: p.units_count,
		description: p.description,
		start_date: format_opt_date(p.start_date),
		launch_date: format_opt_date(p.launch_date),
		possession_date: format_opt_date(p.possession_date),
		configurations_count,
		configurations,
		media,
		documents,
	}))
}

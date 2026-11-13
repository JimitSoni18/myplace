use axum::{
	Json, Router,
	extract::{Multipart, Path, Query, State},
	http::StatusCode,
	routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
	AppState,
	config::CONFIG,
	session_store::AuthUser,
	utils::sql::slugify,
};

pub mod error;
use error::ProjectError;

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/", get(project_list).post(create_project))
		.route(
			"/{id}",
			get(project_detail)
				.put(update_project)
				.delete(delete_project),
		)
		.route(
			"/{id}/media",
			get(project_media_list).post(upload_project_media),
		)
		.route(
			"/{id}/media/reorder",
			axum::routing::post(reorder_project_media),
		)
		.route(
			"/{id}/media/{mid}",
			axum::routing::delete(delete_project_media),
		)
		.route(
			"/{id}/documents",
			get(project_documents_list).post(upload_project_document),
		)
		.route(
			"/{id}/documents/{did}",
			axum::routing::delete(delete_project_document),
		)
		.route(
			"/{id}/configurations",
			get(list_configurations).post(create_configuration),
		)
		.route(
			"/{id}/configurations/{cid}",
			axum::routing::put(update_configuration).delete(delete_configuration),
		)
		.route(
			"/{id}/configurations/{cid}/floor-plan",
			axum::routing::post(upload_configuration_floor_plan),
		)
		.route(
			"/{id}/floor-plans",
			get(list_floor_plans).post(upload_floor_plan),
		)
		.route(
			"/{id}/floor-plans/{fpid}",
			axum::routing::delete(delete_floor_plan),
		)
		.route(
			"/{id}/specifications",
			get(get_specifications).put(save_specifications),
		)
		.route(
			"/{id}/landmarks",
			get(get_landmarks).put(save_landmarks),
		)
}

#[derive(Deserialize, ts_rs::TS, utoipa::ToSchema, utoipa::IntoParams)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct ProjectListQuery {
	#[serde(default)]
	pub q: Option<String>,
	#[serde(default)]
	pub owner_id: Option<i32>,
	#[serde(default)]
	pub project_type_id: Option<i32>,
}

#[derive(Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct ProjectListItem {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub project_type_name: String,
	pub owner_name: String,
	pub city: String,
	pub area: String,
	pub formatted_address: String,
	pub rera_number: Option<String>,
	pub configurations_count: i64,
	pub thumbnail_url: Option<String>,
}

#[derive(Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, export_to = "admin/")]
pub struct ProjectMediaItem {
	pub id: Uuid,
	pub media_type: String,
	pub url: String,
	pub thumbnail_url: String,
	pub sequence: i16,
}

#[derive(Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct ProjectDocumentItem {
	pub id: Uuid,
	pub file_name: String,
	pub url: String,
	pub file_size: Option<i64>,
	pub doc_type: String,
}

#[derive(Serialize, Deserialize, Clone, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct ProjectConfigurationItem {
	pub id: i32,
	pub project_id: i32,
	pub name: String,
	pub bhk: Option<f64>,
	pub carpet_area: Option<f64>,
	pub built_up_area: Option<f64>,
	pub super_built_up_area: Option<f64>,
	pub price: Option<f64>,
	pub price_on_request: bool,
	pub floor_plan_media_id: Option<Uuid>,
	pub floor_plan_url: Option<String>,
	pub floor_plan_thumb_url: Option<String>,
	pub sequence: i16,
	pub is_active: bool,
}

#[derive(Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct ConfigurationFormPayload {
	pub name: String,
	pub bhk: Option<f64>,
	pub carpet_area: Option<f64>,
	pub built_up_area: Option<f64>,
	pub super_built_up_area: Option<f64>,
	pub price: Option<f64>,
	#[serde(default)]
	pub price_on_request: bool,
	pub floor_plan_media_id: Option<Uuid>,
	#[serde(default)]
	pub sequence: i16,
	#[serde(default = "default_true")]
	pub is_active: bool,
}

fn default_true() -> bool {
	true
}

#[derive(Serialize, Deserialize, Clone, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct ProjectFloorPlanItem {
	pub id: Uuid,
	pub project_id: i32,
	pub media_id: Uuid,
	pub title: String,
	pub sequence: i16,
	pub url: String,
	pub thumbnail_url: String,
}

#[derive(Serialize, Deserialize, Clone, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, export_to = "admin/")]
pub struct ProjectSpecificationItem {
	pub id: Option<i32>,
	pub category: String,
	pub items: Vec<String>,
	pub sequence: i16,
}

#[derive(Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, export_to = "admin/")]
pub struct SaveSpecificationsPayload {
	pub specifications: Vec<ProjectSpecificationItem>,
}

#[derive(Serialize, Deserialize, Clone, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct ProjectLandmarkItem {
	pub id: Option<i32>,
	pub category: String,
	pub name: String,
	pub distance: Option<String>,
	pub duration: Option<String>,
	pub latitude: Option<f64>,
	pub longitude: Option<f64>,
	pub source: String,
	pub sequence: i16,
}

#[derive(Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, export_to = "admin/")]
pub struct SaveLandmarksPayload {
	pub landmarks: Vec<ProjectLandmarkItem>,
}

#[derive(Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct ProjectDetailData {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub project_type_id: i32,
	pub project_type_name: String,
	pub owner_id: i32,
	pub owner_name: String,
	pub description: Option<String>,
	pub formatted_address: String,
	pub city: String,
	pub area: String,
	pub state_or_province: String,
	pub postal_code: Option<String>,
	pub latitude: Option<f64>,
	pub longitude: Option<f64>,
	pub rera_number: Option<String>,
	pub rera_authority: Option<String>,
	pub total_land_area: Option<f64>,
	pub towers_count: Option<i32>,
	pub floors_count: Option<i32>,
	pub units_count: Option<i32>,
	pub start_date: Option<String>,
	pub launch_date: Option<String>,
	pub possession_date: Option<String>,
	pub amenities: Vec<String>,
	pub amenity_ids: Vec<i32>,
	pub media: Vec<ProjectMediaItem>,
	pub documents: Vec<ProjectDocumentItem>,
	pub configurations: Vec<ProjectConfigurationItem>,
	pub floor_plans: Vec<ProjectFloorPlanItem>,
	pub specifications: Vec<ProjectSpecificationItem>,
	pub landmarks: Vec<ProjectLandmarkItem>,
}

#[derive(Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct ProjectFormPayload {
	pub name: String,
	pub project_owner_id: i32,
	pub project_type_id: i32,
	pub description: Option<String>,
	pub formatted_address: String,
	pub city: String,
	pub area: String,
	pub state_or_province: String,
	pub postal_code: Option<String>,
	pub latitude: Option<f64>,
	pub longitude: Option<f64>,
	pub rera_number: Option<String>,
	pub rera_authority: Option<String>,
	pub total_land_area: Option<f64>,
	pub towers_count: Option<i32>,
	pub floors_count: Option<i32>,
	pub units_count: Option<i32>,
	pub start_date: Option<String>,
	pub launch_date: Option<String>,
	pub possession_date: Option<String>,
	#[serde(default)]
	pub amenities: Vec<i32>,
}

fn parse_opt_date(s: Option<String>) -> Option<time::Date> {
	let s = s?.trim().to_string();
	if s.is_empty() {
		return None;
	}
	let fmt = time::macros::format_description!("[year]-[month]-[day]");
	time::Date::parse(&s, &fmt).ok()
}

fn format_opt_date(d: Option<time::Date>) -> Option<String> {
	let fmt = time::macros::format_description!("[year]-[month]-[day]");
	d.and_then(|date| date.format(&fmt).ok())
}

// ---------------------------------------------------------------------------
// 1. Projects List
// ---------------------------------------------------------------------------
async fn project_list(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Query(query): Query<ProjectListQuery>,
) -> Result<Json<Vec<ProjectListItem>>, ProjectError> {
	let q_pattern = query.q.as_deref().filter(|s| !s.trim().is_empty()).map(|s| format!("%{}%", s.trim()));

	let rows = sqlx::query_file!(
		"queries/projects/list_projects.sql",
		query.owner_id,
		query.project_type_id,
		q_pattern.as_deref(),
	)
	.fetch_all(&state.model.db)
	.await?;

	let items = rows
		.into_iter()
		.map(|r| {
			let thumbnail_url = r.hero_thumb_key.as_ref().map(|k| CONFIG.asset_url(k));
			ProjectListItem {
				id: r.id,
				name: r.name,
				slug: r.slug,
				project_type_name: r.project_type_name,
				owner_name: r.owner_name,
				city: r.city,
				area: r.area,
				formatted_address: r.formatted_address,
				rera_number: r.rera_number,
				configurations_count: r.configurations_count,
				thumbnail_url,
			}
		})
		.collect();

	Ok(Json(items))
}

// ---------------------------------------------------------------------------
// 2. Project Detail (Full Aggregation)
// ---------------------------------------------------------------------------
async fn project_detail(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
) -> Result<Json<ProjectDetailData>, ProjectError> {
	let p = sqlx::query_file!("queries/projects/get_project_detail.sql", id)
		.fetch_optional(&state.model.db)
		.await?
		.ok_or(ProjectError::NotFound)?;

	// Amenities
	#[derive(sqlx::FromRow)]
	struct AmenityRow {
		id: i32,
		name: String,
	}
	let amenity_rows = sqlx::query_as::<_, AmenityRow>(
		"SELECT a.id, a.name FROM amenities a
		 JOIN project_amenities pa ON pa.amenity_id = a.id
		 WHERE pa.project_id = $1 ORDER BY a.name ASC",
	)
	.bind(id)
	.fetch_all(&state.model.db)
	.await?;

	let amenities = amenity_rows.iter().map(|a| a.name.clone()).collect();
	let amenity_ids = amenity_rows.iter().map(|a| a.id).collect();

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
		.await?;

	let media = media_rows
		.into_iter()
		.map(|m| {
			let url = CONFIG.asset_url(&m.s3_key);
			let thumbnail_url = m
				.thumbnail_key
				.as_ref()
				.map(|k| CONFIG.asset_url(k))
				.unwrap_or_else(|| url.clone());
			ProjectMediaItem {
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
		.await?;

	let documents = doc_rows
		.into_iter()
		.map(|d| ProjectDocumentItem {
			id: d.id,
			file_name: d.display_name,
			url: CONFIG.asset_url(&d.s3_key),
			file_size: d.file_size,
			doc_type: d.doc_type,
		})
		.collect();

	// Configurations
	let config_rows = sqlx::query_file!("queries/configurations/list_project_configurations.sql", id)
		.fetch_all(&state.model.db)
		.await?;

	let configurations = config_rows
		.into_iter()
		.map(|c| {
			let floor_plan_url = c.floor_plan_key.as_deref().map(|k| CONFIG.asset_url(k));
			let floor_plan_thumb_url = c.floor_plan_thumb_key.as_deref().map(|k| CONFIG.asset_url(k));
			ProjectConfigurationItem {
				id: c.id,
				project_id: c.project_id,
				name: c.name,
				bhk: c.bhk,
				carpet_area: c.carpet_area,
				built_up_area: c.built_up_area,
				super_built_up_area: c.super_built_up_area,
				price: c.price,
				price_on_request: c.price_on_request,
				floor_plan_media_id: c.floor_plan_media_id,
				floor_plan_url,
				floor_plan_thumb_url,
				sequence: c.sequence,
				is_active: c.is_active,
			}
		})
		.collect();

	// Project Floor Plans
	let floor_plan_rows = sqlx::query_file!("queries/floor_plans/get_project_floor_plans.sql", id)
		.fetch_all(&state.model.db)
		.await?;

	let floor_plans = floor_plan_rows
		.into_iter()
		.map(|fp| {
			let url = CONFIG.asset_url(&fp.s3_key);
			let thumbnail_url = fp
				.thumbnail_key
				.as_ref()
				.map(|k| CONFIG.asset_url(k))
				.unwrap_or_else(|| url.clone());
			ProjectFloorPlanItem {
				id: fp.id,
				project_id: fp.project_id,
				media_id: fp.media_id,
				title: fp.title,
				sequence: fp.sequence,
				url,
				thumbnail_url,
			}
		})
		.collect();

	// Specifications
	let spec_rows = sqlx::query_file!("queries/specifications/get_project_specifications.sql", id)
		.fetch_all(&state.model.db)
		.await?;

	let specifications = spec_rows
		.into_iter()
		.map(|s| ProjectSpecificationItem {
			id: Some(s.id),
			category: s.category,
			items: s.items,
			sequence: s.sequence,
		})
		.collect();

	// Landmarks
	let landmark_rows = sqlx::query_file!("queries/landmarks/get_project_landmarks.sql", id)
		.fetch_all(&state.model.db)
		.await?;

	let landmarks = landmark_rows
		.into_iter()
		.map(|l| ProjectLandmarkItem {
			id: Some(l.id),
			category: l.category,
			name: l.name,
			distance: l.distance,
			duration: l.duration,
			latitude: l.latitude,
			longitude: l.longitude,
			source: l.source,
			sequence: l.sequence,
		})
		.collect();

	Ok(Json(ProjectDetailData {
		id: p.id,
		name: p.name,
		slug: p.slug,
		project_type_id: p.project_type_id,
		project_type_name: p.project_type_name,
		owner_id: p.project_owner_id,
		owner_name: p.owner_name,
		description: p.description,
		formatted_address: p.formatted_address,
		city: p.city,
		area: p.area,
		state_or_province: p.state_or_province,
		postal_code: p.postal_code,
		latitude: p.latitude,
		longitude: p.longitude,
		rera_number: p.rera_number,
		rera_authority: p.rera_authority,
		total_land_area: p.total_land_area,
		towers_count: p.towers_count,
		floors_count: p.floors_count,
		units_count: p.units_count,
		start_date: format_opt_date(p.start_date),
		launch_date: format_opt_date(p.launch_date),
		possession_date: format_opt_date(p.possession_date),
		amenities,
		amenity_ids,
		media,
		documents,
		configurations,
		floor_plans,
		specifications,
		landmarks,
	}))
}

// ---------------------------------------------------------------------------
// 3. Create Project
// ---------------------------------------------------------------------------
async fn create_project(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Json(payload): Json<ProjectFormPayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), ProjectError> {
	let name = payload.name.trim();
	if name.is_empty() {
		return Err(ProjectError::Validation("Project name cannot be empty".to_string()));
	}

	let base_slug = slugify(name);
	let mut slug = base_slug.clone();
	let mut counter = 1;
	while sqlx::query_scalar!(
		"SELECT id FROM projects WHERE slug = $1 AND deleted_at IS NULL",
		slug
	)
	.fetch_optional(&state.model.db)
	.await?
	.is_some()
	{
		slug = format!("{base_slug}-{counter}");
		counter += 1;
	}

	let start_date = parse_opt_date(payload.start_date);
	let launch_date = parse_opt_date(payload.launch_date);
	let possession_date = parse_opt_date(payload.possession_date);

	let mut txn = state.model.db.begin().await?;

	let rec = sqlx::query_file!(
		"queries/projects/insert_project.sql",
		payload.project_owner_id,
		payload.project_type_id,
		name,
		slug,
		payload.description.as_deref().filter(|s| !s.trim().is_empty()),
		payload.formatted_address.trim(),
		payload.city.trim(),
		payload.area.trim(),
		payload.state_or_province.trim(),
		payload.postal_code.as_deref().filter(|s| !s.trim().is_empty()),
		payload.latitude,
		payload.longitude,
		payload.rera_number.as_deref().filter(|s| !s.trim().is_empty()),
		payload.rera_authority.as_deref().filter(|s| !s.trim().is_empty()),
		payload.total_land_area,
		payload.towers_count,
		payload.floors_count,
		payload.units_count,
		start_date,
		launch_date,
		possession_date,
	)
	.fetch_one(&mut *txn)
	.await?;

	for aid in payload.amenities {
		sqlx::query_file!("queries/projects/insert_project_amenity.sql", rec.id, aid)
			.execute(&mut *txn)
			.await?;
	}

	txn.commit().await?;

	Ok((
		StatusCode::CREATED,
		Json(json!({ "id": rec.id, "slug": slug })),
	))
}

// ---------------------------------------------------------------------------
// 4. Update Project
// ---------------------------------------------------------------------------
async fn update_project(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
	Json(payload): Json<ProjectFormPayload>,
) -> Result<Json<serde_json::Value>, ProjectError> {
	let existing = sqlx::query_file!("queries/projects/get_project_for_edit.sql", id)
		.fetch_optional(&state.model.db)
		.await?
		.ok_or(ProjectError::NotFound)?;

	let name = payload.name.trim();
	if name.is_empty() {
		return Err(ProjectError::Validation("Project name cannot be empty".to_string()));
	}

	let slug = if name == existing.name {
		existing.slug
	} else {
		let base_slug = slugify(name);
		let mut s = base_slug.clone();
		let mut counter = 1;
		while sqlx::query_scalar!(
			"SELECT id FROM projects WHERE slug = $1 AND id != $2 AND deleted_at IS NULL",
			s,
			id
		)
		.fetch_optional(&state.model.db)
		.await?
		.is_some()
		{
			s = format!("{base_slug}-{counter}");
			counter += 1;
		}
		s
	};

	let start_date = parse_opt_date(payload.start_date);
	let launch_date = parse_opt_date(payload.launch_date);
	let possession_date = parse_opt_date(payload.possession_date);

	let mut txn = state.model.db.begin().await?;

	sqlx::query_file!(
		"queries/projects/update_project.sql",
		id,
		payload.project_owner_id,
		payload.project_type_id,
		name,
		slug,
		payload.description.as_deref().filter(|s| !s.trim().is_empty()),
		payload.formatted_address.trim(),
		payload.city.trim(),
		payload.area.trim(),
		payload.state_or_province.trim(),
		payload.postal_code.as_deref().filter(|s| !s.trim().is_empty()),
		payload.latitude,
		payload.longitude,
		payload.rera_number.as_deref().filter(|s| !s.trim().is_empty()),
		payload.rera_authority.as_deref().filter(|s| !s.trim().is_empty()),
		payload.total_land_area,
		payload.towers_count,
		payload.floors_count,
		payload.units_count,
		start_date,
		launch_date,
		possession_date,
	)
	.execute(&mut *txn)
	.await?;

	// Sync amenities
	sqlx::query_file!("queries/projects/delete_project_amenities.sql", id)
		.execute(&mut *txn)
		.await?;

	for aid in payload.amenities {
		sqlx::query_file!("queries/projects/insert_project_amenity.sql", id, aid)
			.execute(&mut *txn)
			.await?;
	}

	txn.commit().await?;

	Ok(Json(json!({ "success": true, "id": id, "slug": slug })))
}

// ---------------------------------------------------------------------------
// 5. Delete Project
// ---------------------------------------------------------------------------
async fn delete_project(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
) -> Result<StatusCode, ProjectError> {
	sqlx::query_file!("queries/projects/delete_project.sql", id)
		.execute(&state.model.db)
		.await?;

	Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// 6. Configurations Handlers
// ---------------------------------------------------------------------------
async fn list_configurations(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
) -> Result<Json<Vec<ProjectConfigurationItem>>, ProjectError> {
	let rows = sqlx::query_file!("queries/configurations/list_project_configurations.sql", id)
		.fetch_all(&state.model.db)
		.await?;

	let items = rows
		.into_iter()
		.map(|c| {
			let floor_plan_url = c.floor_plan_key.as_deref().map(|k| CONFIG.asset_url(k));
			let floor_plan_thumb_url = c.floor_plan_thumb_key.as_deref().map(|k| CONFIG.asset_url(k));
			ProjectConfigurationItem {
				id: c.id,
				project_id: c.project_id,
				name: c.name,
				bhk: c.bhk,
				carpet_area: c.carpet_area,
				built_up_area: c.built_up_area,
				super_built_up_area: c.super_built_up_area,
				price: c.price,
				price_on_request: c.price_on_request,
				floor_plan_media_id: c.floor_plan_media_id,
				floor_plan_url,
				floor_plan_thumb_url,
				sequence: c.sequence,
				is_active: c.is_active,
			}
		})
		.collect();

	Ok(Json(items))
}

async fn create_configuration(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
	Json(payload): Json<ConfigurationFormPayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), ProjectError> {
	let name = payload.name.trim();
	if name.is_empty() {
		return Err(ProjectError::Validation("Configuration name cannot be empty".to_string()));
	}

	let rec = sqlx::query_file!(
		"queries/configurations/insert_configuration.sql",
		id,
		name,
		payload.bhk,
		payload.carpet_area,
		payload.built_up_area,
		payload.super_built_up_area,
		payload.price,
		payload.price_on_request,
		payload.floor_plan_media_id,
		payload.sequence,
	)
	.fetch_one(&state.model.db)
	.await?;

	Ok((StatusCode::CREATED, Json(json!({ "id": rec.id }))))
}

async fn update_configuration(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path((id, cid)): Path<(i32, i32)>,
	Json(payload): Json<ConfigurationFormPayload>,
) -> Result<Json<serde_json::Value>, ProjectError> {
	let name = payload.name.trim();
	if name.is_empty() {
		return Err(ProjectError::Validation("Configuration name cannot be empty".to_string()));
	}

	sqlx::query!(
		"UPDATE project_configurations
		 SET name = $3, bhk = $4, carpet_area = $5, built_up_area = $6, super_built_up_area = $7,
		     price = $8, price_on_request = $9, floor_plan_media_id = $10, sequence = $11, is_active = $12
		 WHERE id = $1 AND project_id = $2",
		cid,
		id,
		name,
		payload.bhk,
		payload.carpet_area,
		payload.built_up_area,
		payload.super_built_up_area,
		payload.price,
		payload.price_on_request,
		payload.floor_plan_media_id,
		payload.sequence,
		payload.is_active,
	)
	.execute(&state.model.db)
	.await?;

	Ok(Json(json!({ "success": true, "id": cid })))
}

async fn delete_configuration(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path((id, cid)): Path<(i32, i32)>,
) -> Result<StatusCode, ProjectError> {
	sqlx::query!(
		"DELETE FROM project_configurations WHERE id = $1 AND project_id = $2",
		cid,
		id
	)
	.execute(&state.model.db)
	.await?;

	Ok(StatusCode::NO_CONTENT)
}

async fn upload_configuration_floor_plan(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path((id, cid)): Path<(i32, i32)>,
	mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, ProjectError> {
	while let Some(field) = multipart.next_field().await.map_err(|e| ProjectError::Internal(e.to_string()))? {
		let bytes = field.bytes().await.map_err(|e| ProjectError::Internal(e.to_string()))?.to_vec();
		if bytes.is_empty() {
			continue;
		}

		let upload_res = crate::media::process_and_upload_image(&state.model.db, &state.s3_client, bytes).await?;

		sqlx::query!(
			"UPDATE project_configurations SET floor_plan_media_id = $1 WHERE id = $2 AND project_id = $3",
			upload_res.media_id,
			cid,
			id
		)
		.execute(&state.model.db)
		.await?;

		return Ok(Json(json!({
			"media_id": upload_res.media_id,
			"url": upload_res.original_url,
			"thumbnail_url": upload_res.thumbnail_url
		})));
	}

	Err(ProjectError::Validation("No image file uploaded".to_string()))
}

// ---------------------------------------------------------------------------
// 7. Project Floor Plans (General Site Layouts / Cluster Plans)
// ---------------------------------------------------------------------------
async fn list_floor_plans(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
) -> Result<Json<Vec<ProjectFloorPlanItem>>, ProjectError> {
	let rows = sqlx::query_file!("queries/floor_plans/get_project_floor_plans.sql", id)
		.fetch_all(&state.model.db)
		.await?;

	let items = rows
		.into_iter()
		.map(|fp| {
			let url = CONFIG.asset_url(&fp.s3_key);
			let thumbnail_url = fp
				.thumbnail_key
				.as_ref()
				.map(|k| CONFIG.asset_url(k))
				.unwrap_or_else(|| url.clone());
			ProjectFloorPlanItem {
				id: fp.id,
				project_id: fp.project_id,
				media_id: fp.media_id,
				title: fp.title,
				sequence: fp.sequence,
				url,
				thumbnail_url,
			}
		})
		.collect();

	Ok(Json(items))
}

async fn upload_floor_plan(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
	mut multipart: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>), ProjectError> {
	let mut title = "Site Layout Plan".to_string();
	let mut file_bytes: Option<Vec<u8>> = None;

	while let Some(field) = multipart.next_field().await.map_err(|e| ProjectError::Internal(e.to_string()))? {
		let name = field.name().unwrap_or("").to_string();
		if name == "title" {
			let text = field.text().await.map_err(|e| ProjectError::Internal(e.to_string()))?;
			if !text.trim().is_empty() {
				title = text.trim().to_string();
			}
		} else if name == "file" || name == "files" {
			let bytes = field.bytes().await.map_err(|e| ProjectError::Internal(e.to_string()))?.to_vec();
			if !bytes.is_empty() {
				file_bytes = Some(bytes);
			}
		}
	}

	let bytes = file_bytes.ok_or_else(|| ProjectError::Validation("No image file provided".to_string()))?;
	let upload_res = crate::media::process_and_upload_image(&state.model.db, &state.s3_client, bytes).await?;

	let next_seq: i16 = sqlx::query_scalar!(
		"SELECT (COALESCE(MAX(sequence), 0) + 1)::SMALLINT AS \"seq!: i16\" FROM project_floor_plans WHERE project_id = $1",
		id
	)
	.fetch_one(&state.model.db)
	.await?;

	let floor_plan_id = sqlx::query_file!(
		"queries/floor_plans/insert_project_floor_plan.sql",
		id,
		upload_res.media_id,
		title,
		next_seq,
	)
	.fetch_one(&state.model.db)
	.await?
	.id;

	Ok((
		StatusCode::CREATED,
		Json(json!({
			"id": floor_plan_id,
			"media_id": upload_res.media_id,
			"title": title,
			"url": upload_res.original_url,
			"thumbnail_url": upload_res.thumbnail_url,
		})),
	))
}

async fn delete_floor_plan(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path((id, fpid)): Path<(i32, Uuid)>,
) -> Result<StatusCode, ProjectError> {
	sqlx::query_file!("queries/floor_plans/delete_project_floor_plan.sql", fpid, id)
		.execute(&state.model.db)
		.await?;

	Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// 8. Specifications Handlers
// ---------------------------------------------------------------------------
async fn get_specifications(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
) -> Result<Json<Vec<ProjectSpecificationItem>>, ProjectError> {
	let rows = sqlx::query_file!("queries/specifications/get_project_specifications.sql", id)
		.fetch_all(&state.model.db)
		.await?;

	let items = rows
		.into_iter()
		.map(|s| ProjectSpecificationItem {
			id: Some(s.id),
			category: s.category,
			items: s.items,
			sequence: s.sequence,
		})
		.collect();

	Ok(Json(items))
}

async fn save_specifications(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
	Json(payload): Json<SaveSpecificationsPayload>,
) -> Result<Json<serde_json::Value>, ProjectError> {
	let mut txn = state.model.db.begin().await?;

	sqlx::query_file!("queries/specifications/delete_project_specifications.sql", id)
		.execute(&mut *txn)
		.await?;

	for (idx, spec) in payload.specifications.iter().enumerate() {
		let cat = spec.category.trim();
		if cat.is_empty() || spec.items.is_empty() {
			continue;
		}
		sqlx::query_file!(
			"queries/specifications/insert_specification.sql",
			id,
			cat,
			&spec.items,
			idx as i16,
		)
		.execute(&mut *txn)
		.await?;
	}

	txn.commit().await?;

	Ok(Json(json!({ "success": true })))
}

// ---------------------------------------------------------------------------
// 9. Landmarks Handlers
// ---------------------------------------------------------------------------
async fn get_landmarks(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
) -> Result<Json<Vec<ProjectLandmarkItem>>, ProjectError> {
	let rows = sqlx::query_file!("queries/landmarks/get_project_landmarks.sql", id)
		.fetch_all(&state.model.db)
		.await?;

	let items = rows
		.into_iter()
		.map(|l| ProjectLandmarkItem {
			id: Some(l.id),
			category: l.category,
			name: l.name,
			distance: l.distance,
			duration: l.duration,
			latitude: l.latitude,
			longitude: l.longitude,
			source: l.source,
			sequence: l.sequence,
		})
		.collect();

	Ok(Json(items))
}

async fn save_landmarks(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
	Json(payload): Json<SaveLandmarksPayload>,
) -> Result<Json<serde_json::Value>, ProjectError> {
	let mut txn = state.model.db.begin().await?;

	sqlx::query_file!("queries/landmarks/delete_project_landmarks.sql", id)
		.execute(&mut *txn)
		.await?;

	for (idx, lm) in payload.landmarks.iter().enumerate() {
		let name = lm.name.trim();
		let cat = lm.category.trim();
		if name.is_empty() {
			continue;
		}
		sqlx::query_file!(
			"queries/landmarks/insert_landmark.sql",
			id,
			cat,
			name,
			lm.distance.as_deref().filter(|s| !s.trim().is_empty()),
			lm.duration.as_deref().filter(|s| !s.trim().is_empty()),
			lm.latitude,
			lm.longitude,
			if lm.source.is_empty() { "manual" } else { &lm.source },
			idx as i16,
		)
		.execute(&mut *txn)
		.await?;
	}

	txn.commit().await?;

	Ok(Json(json!({ "success": true })))
}

// ---------------------------------------------------------------------------
// 10. Project Media Gallery Handlers
// ---------------------------------------------------------------------------
async fn project_media_list(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
) -> Result<Json<Vec<ProjectMediaItem>>, ProjectError> {
	struct DbMedia {
		id: Uuid,
		media_type: String,
		s3_key: String,
		thumbnail_key: Option<String>,
		sequence: i16,
	}

	let rows = sqlx::query_file_as!(DbMedia, "queries/projects/get_project_media.sql", id)
		.fetch_all(&state.model.db)
		.await?;

	let items = rows
		.into_iter()
		.map(|m| {
			let url = CONFIG.asset_url(&m.s3_key);
			let thumbnail_url = m
				.thumbnail_key
				.as_ref()
				.map(|k| CONFIG.asset_url(k))
				.unwrap_or_else(|| url.clone());
			ProjectMediaItem {
				id: m.id,
				media_type: m.media_type,
				url,
				thumbnail_url,
				sequence: m.sequence,
			}
		})
		.collect();

	Ok(Json(items))
}

async fn upload_project_media(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
	mut multipart: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>), ProjectError> {
	let mut uploaded_ids = Vec::new();

	while let Some(field) = multipart.next_field().await.map_err(|e| ProjectError::Internal(e.to_string()))? {
		let field_name = field.name().unwrap_or("").to_string();
		let filename = field.file_name().unwrap_or("").to_string();
		let content_type = field.content_type().map(|s| s.to_string());
		let bytes = field.bytes().await.map_err(|e| ProjectError::Internal(e.to_string()))?.to_vec();

		if bytes.is_empty() {
			continue;
		}

		let is_video = crate::media::is_video_upload(
			&field_name,
			content_type.as_deref(),
			Some(&filename),
		);

		let media_id = if is_video {
			let res = crate::media::process_and_upload_video(
				&state.model.db,
				&state.s3_client,
				bytes,
				&filename,
				content_type.as_deref(),
			)
			.await?;
			res.media_id
		} else {
			let res = crate::media::process_and_upload_image(&state.model.db, &state.s3_client, bytes).await?;
			res.media_id
		};

		let next_seq: i16 = sqlx::query_scalar!(
			"SELECT (COALESCE(MAX(sequence), 0) + 1)::SMALLINT AS \"seq!: i16\" FROM project_media WHERE project_id = $1",
			id
		)
		.fetch_one(&state.model.db)
		.await?;

		sqlx::query_file!(
			"queries/projects/insert_project_media.sql",
			id,
			media_id,
			next_seq,
		)
		.execute(&state.model.db)
		.await?;

		uploaded_ids.push(media_id);
	}

	Ok((StatusCode::CREATED, Json(json!({ "uploaded": uploaded_ids }))))
}

#[derive(Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, export_to = "admin/")]
pub struct ReorderMediaPayload {
	pub order: Vec<Uuid>,
}

async fn reorder_project_media(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
	Json(payload): Json<ReorderMediaPayload>,
) -> Result<Json<serde_json::Value>, ProjectError> {
	let mut txn = state.model.db.begin().await?;

	sqlx::query_file!(
		"queries/projects/shift_project_media_sequences_negative.sql",
		id
	)
	.execute(&mut *txn)
	.await?;

	for (idx, mid) in payload.order.iter().enumerate() {
		sqlx::query_file!(
			"queries/projects/update_project_media_sequence.sql",
			id,
			mid,
			idx as i16,
		)
		.execute(&mut *txn)
		.await?;
	}

	txn.commit().await?;

	Ok(Json(json!({ "success": true })))
}

async fn delete_project_media(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path((id, mid)): Path<(i32, Uuid)>,
) -> Result<StatusCode, ProjectError> {
	sqlx::query_file!("queries/projects/delete_project_media.sql", id, mid)
		.execute(&state.model.db)
		.await?;

	Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// 11. Project Documents Handlers
// ---------------------------------------------------------------------------
async fn project_documents_list(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
) -> Result<Json<Vec<ProjectDocumentItem>>, ProjectError> {
	struct DbDoc {
		id: Uuid,
		s3_key: String,
		display_name: String,
		doc_type: String,
		file_size: Option<i64>,
	}

	let rows = sqlx::query_file_as!(DbDoc, "queries/projects/get_project_documents.sql", id)
		.fetch_all(&state.model.db)
		.await?;

	let items = rows
		.into_iter()
		.map(|d| ProjectDocumentItem {
			id: d.id,
			file_name: d.display_name,
			url: CONFIG.asset_url(&d.s3_key),
			file_size: d.file_size,
			doc_type: d.doc_type,
		})
		.collect();

	Ok(Json(items))
}

async fn upload_project_document(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
	mut multipart: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>), ProjectError> {
	let mut doc_type = "brochure".to_string();
	let mut display_name = String::new();
	let mut file_bytes: Option<Vec<u8>> = None;
	let mut file_name = "document.pdf".to_string();
	let mut content_type = "application/pdf".to_string();

	while let Some(field) = multipart.next_field().await.map_err(|e| ProjectError::Internal(e.to_string()))? {
		let name = field.name().unwrap_or("").to_string();
		if name == "doc_type" {
			doc_type = field.text().await.map_err(|e| ProjectError::Internal(e.to_string()))?;
		} else if name == "display_name" {
			display_name = field.text().await.map_err(|e| ProjectError::Internal(e.to_string()))?;
		} else if name == "file" {
			if let Some(fn_str) = field.file_name() {
				file_name = fn_str.to_string();
			}
			if let Some(ct) = field.content_type() {
				content_type = ct.to_string();
			}
			let bytes = field.bytes().await.map_err(|e| ProjectError::Internal(e.to_string()))?.to_vec();
			if !bytes.is_empty() {
				file_bytes = Some(bytes);
			}
		}
	}

	let bytes = file_bytes.ok_or_else(|| ProjectError::Validation("No document file provided".to_string()))?;
	if display_name.trim().is_empty() {
		display_name = file_name.clone();
	}

	let upload_res = crate::media::upload_document(
		&state.model.db,
		&state.s3_client,
		&file_name,
		&content_type,
		bytes,
	)
	.await?;

	sqlx::query_file!(
		"queries/projects/insert_project_document.sql",
		id,
		upload_res.media_id,
		display_name.trim(),
		doc_type.trim(),
	)
	.execute(&state.model.db)
	.await?;

	Ok((
		StatusCode::CREATED,
		Json(json!({
			"id": upload_res.media_id,
			"display_name": display_name,
			"url": upload_res.url,
		})),
	))
}

async fn delete_project_document(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path((id, did)): Path<(i32, Uuid)>,
) -> Result<StatusCode, ProjectError> {
	sqlx::query_file!("queries/projects/delete_project_document.sql", did, id)
		.execute(&state.model.db)
		.await?;

	Ok(StatusCode::NO_CONTENT)
}

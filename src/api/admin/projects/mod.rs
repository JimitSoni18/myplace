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
		.nest(
			"/{pid}/properties",
			crate::api::admin::properties::project_router(),
		)
}

#[derive(Deserialize, ts_rs::TS)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct ProjectListQuery {
	#[serde(default)]
	pub q: Option<String>,
	#[serde(default)]
	pub owner_id: Option<i32>,
	#[serde(default)]
	pub category: Option<String>,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct ProjectListItem {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub category: String,
	pub owner_name: String,
	pub location_name: String,
	pub properties_count: i64,
	pub thumbnail_url: Option<String>,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "admin/")]
pub struct ProjectPropertySummary {
	pub total: i64,
	pub available: i64,
	pub residential: i64,
	pub commercial: i64,
	pub land: i64,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "admin/")]
pub struct ProjectMediaItem {
	pub id: Uuid,
	pub media_type: String,
	pub url: String,
	pub thumbnail_url: String,
	pub sequence: i16,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct ProjectDocumentItem {
	pub id: Uuid,
	pub file_name: String,
	pub url: String,
	pub file_size: Option<i64>,
	pub doc_type: String,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct ProjectDetailData {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub category: String,
	pub owner_id: i32,
	pub owner_name: String,
	pub location_id: i32,
	pub location_name: String,
	pub description: Option<String>,
	pub start_date: Option<String>,
	pub launch_date: Option<String>,
	pub possession_date: Option<String>,
	pub amenities: Vec<String>,
	pub amenity_ids: Vec<i32>,
	pub property_summary: ProjectPropertySummary,
	pub media: Vec<ProjectMediaItem>,
	pub documents: Vec<ProjectDocumentItem>,
}

#[derive(Deserialize, ts_rs::TS)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct ProjectFormPayload {
	pub name: String,
	pub category: String,
	pub project_owner_id: i32,
	pub location_id: i32,
	pub description: Option<String>,
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

async fn project_list(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Query(query): Query<ProjectListQuery>,
) -> Result<Json<Vec<ProjectListItem>>, ProjectError> {
	let q_pattern = query.q.as_deref().filter(|s| !s.trim().is_empty()).map(|s| format!("%{}%", s.trim()));
	let category_filter = query.category.as_deref().filter(|s| !s.trim().is_empty());

	let rows = sqlx::query_file!(
		"queries/projects/list_projects.sql",
		query.owner_id,
		category_filter,
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
				category: r.category,
				owner_name: r.owner_name,
				location_name: r.location.unwrap_or_else(|| "Unspecified".to_string()),
				properties_count: r.properties_count,
				thumbnail_url,
			}
		})
		.collect();

	Ok(Json(items))
}

async fn project_detail(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
) -> Result<Json<ProjectDetailData>, ProjectError> {
	#[allow(dead_code)]
	struct DbProject {
		id: i32,
		name: String,
		slug: String,
		category: String,
		project_owner_id: i32,
		owner_name: String,
		location_id: i32,
		location_name: Option<String>,
		description: Option<String>,
		start_date: Option<time::Date>,
		launch_date: Option<time::Date>,
		possession_date: Option<time::Date>,
		created_at: time::OffsetDateTime,
	}

	let p = sqlx::query_file_as!(DbProject, "queries/projects/get_project_detail.sql", id)
		.fetch_one(&state.model.db)
		.await?;

	struct DbMedia {
		media_id: Uuid,
		media_type: String,
		s3_key: String,
		thumbnail_key: Option<String>,
		sequence: i16,
	}

	let media_rows = sqlx::query_file_as!(DbMedia, "queries/projects/get_project_media.sql", id)
		.fetch_all(&state.model.db)
		.await?;

	let media: Vec<ProjectMediaItem> = media_rows
		.into_iter()
		.map(|r| {
			let url = CONFIG.asset_url(&r.s3_key);
			let thumbnail_url = r
				.thumbnail_key
				.as_ref()
				.map(|k| CONFIG.asset_url(k))
				.unwrap_or_else(|| url.clone());
			ProjectMediaItem {
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
		.await?;

	let documents: Vec<ProjectDocumentItem> = doc_rows
		.into_iter()
		.map(|r| {
			let url = CONFIG.asset_url(&r.s3_key);
			let file_name = r
				.display_name
				.unwrap_or_else(|| r.s3_key.split('/').last().unwrap_or("document").to_string());
			ProjectDocumentItem {
				id: r.media_id,
				file_name,
				url,
				file_size: r.file_size,
				doc_type: r.doc_type.unwrap_or_else(|| "brochure".to_string()),
			}
		})
		.collect();

	#[derive(sqlx::FromRow)]
	struct AmenityRow {
		id: i32,
		name: String,
	}

	let amenity_rows = sqlx::query_as::<_, AmenityRow>(
		r#"
		SELECT a.id, a.name
		FROM amenities a
		JOIN project_amenities pa ON pa.amenity_id = a.id
		WHERE pa.project_id = $1
		ORDER BY a.name ASC
		"#,
	)
	.bind(id)
	.fetch_all(&state.model.db)
	.await
	.unwrap_or_default();

	let amenities = amenity_rows.iter().map(|a| a.name.clone()).collect();
	let amenity_ids = amenity_rows.iter().map(|a| a.id).collect();

	let count_rows = sqlx::query_file!(
		"queries/projects/get_project_property_counts.sql",
		id
	)
	.fetch_all(&state.model.db)
	.await
	.unwrap_or_default();

	let mut residential = 0;
	let mut commercial = 0;
	let mut land = 0;
	let mut total = 0;
	for r in count_rows {
		let cnt = r.count.unwrap_or(0);
		total += cnt;
		match r.category.as_str() {
			"RESIDENTIAL" => residential += cnt,
			"COMMERCIAL" => commercial += cnt,
			"LAND" => land += cnt,
			_ => {}
		}
	}
	let property_summary = ProjectPropertySummary {
		total,
		available: total,
		residential,
		commercial,
		land,
	};

	Ok(Json(ProjectDetailData {
		id: p.id,
		name: p.name,
		slug: p.slug,
		category: p.category,
		owner_id: p.project_owner_id,
		owner_name: p.owner_name,
		location_id: p.location_id,
		location_name: p.location_name.unwrap_or_else(|| "Unspecified".to_string()),
		description: p.description,
		start_date: format_opt_date(p.start_date),
		launch_date: format_opt_date(p.launch_date),
		possession_date: format_opt_date(p.possession_date),
		amenities,
		amenity_ids,
		property_summary,
		media,
		documents,
	}))
}

async fn create_project(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Json(form): Json<ProjectFormPayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), ProjectError> {
	let trimmed_name = form.name.trim();
	if trimmed_name.is_empty() {
		return Err(ProjectError::Validation("Project name is required".to_string()));
	}
	if form.category.trim().is_empty() {
		return Err(ProjectError::Validation("Category is required".to_string()));
	}
	if form.project_owner_id <= 0 {
		return Err(ProjectError::Validation("Valid project owner is required".to_string()));
	}
	if form.location_id <= 0 {
		return Err(ProjectError::Validation("Valid location is required".to_string()));
	}

	let slug = slugify(trimmed_name);
	let start_date = parse_opt_date(form.start_date);
	let launch_date = parse_opt_date(form.launch_date);
	let possession_date = parse_opt_date(form.possession_date);

	let mut tx = state.model.db.begin().await?;

	let row = sqlx::query_file!(
		"queries/projects/insert_project.sql",
		form.project_owner_id,
		form.location_id,
		trimmed_name,
		slug,
		form.description.as_deref(),
		form.category,
		start_date,
		launch_date,
		possession_date,
	)
	.fetch_one(&mut *tx)
	.await?;

	let new_id = row.id;

	let mut seen = std::collections::HashSet::new();
	for amenity_id in form.amenities {
		if seen.insert(amenity_id) {
			sqlx::query_file!(
				"queries/projects/insert_project_amenity.sql",
				new_id,
				amenity_id,
			)
			.execute(&mut *tx)
			.await?;
		}
	}

	tx.commit().await?;
	tracing::info!(project_id = new_id, "created project");

	Ok((
		StatusCode::CREATED,
		Json(json!({
			"id": new_id,
			"name": trimmed_name,
			"slug": slug,
		})),
	))
}

async fn update_project(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
	Json(form): Json<ProjectFormPayload>,
) -> Result<Json<serde_json::Value>, ProjectError> {
	let trimmed_name = form.name.trim();
	if trimmed_name.is_empty() {
		return Err(ProjectError::Validation("Project name is required".to_string()));
	}
	if form.category.trim().is_empty() {
		return Err(ProjectError::Validation("Category is required".to_string()));
	}
	if form.project_owner_id <= 0 {
		return Err(ProjectError::Validation("Valid project owner is required".to_string()));
	}
	if form.location_id <= 0 {
		return Err(ProjectError::Validation("Valid location is required".to_string()));
	}

	let slug = slugify(trimmed_name);
	let start_date = parse_opt_date(form.start_date);
	let launch_date = parse_opt_date(form.launch_date);
	let possession_date = parse_opt_date(form.possession_date);

	let mut tx = state.model.db.begin().await?;

	sqlx::query_file!(
		"queries/projects/update_project.sql",
		form.project_owner_id,
		form.location_id,
		trimmed_name,
		slug,
		form.description.as_deref(),
		form.category,
		start_date,
		launch_date,
		possession_date,
		id,
	)
	.execute(&mut *tx)
	.await?;

	sqlx::query_file!("queries/projects/delete_project_amenities.sql", id)
		.execute(&mut *tx)
		.await?;

	let mut seen = std::collections::HashSet::new();
	for amenity_id in form.amenities {
		if seen.insert(amenity_id) {
			sqlx::query_file!(
				"queries/projects/insert_project_amenity.sql",
				id,
				amenity_id,
			)
			.execute(&mut *tx)
			.await?;
		}
	}

	tx.commit().await?;
	tracing::info!(project_id = id, "updated project");

	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::ProjectUpdated(id))
		.await;

	Ok(Json(json!({
		"success": true,
		"id": id,
		"name": trimmed_name,
		"slug": slug,
	})))
}

async fn delete_project(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
) -> Result<StatusCode, ProjectError> {
	sqlx::query_file!("queries/projects/delete_project.sql", id)
		.execute(&state.model.db)
		.await?;

	tracing::info!(project_id = id, "soft-deleted project");
	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::ProjectUpdated(id))
		.await;

	Ok(StatusCode::NO_CONTENT)
}

async fn project_media_list(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
) -> Result<Json<Vec<ProjectMediaItem>>, ProjectError> {
	sqlx::query_file!("queries/projects/check_project_exists.sql", id)
		.fetch_one(&state.model.db)
		.await?;

	struct DbMedia {
		media_id: Uuid,
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
		.map(|r| {
			let url = CONFIG.asset_url(&r.s3_key);
			let thumbnail_url = r
				.thumbnail_key
				.as_ref()
				.map(|k| CONFIG.asset_url(k))
				.unwrap_or_else(|| url.clone());
			ProjectMediaItem {
				id: r.media_id,
				media_type: r.media_type,
				url,
				thumbnail_url,
				sequence: r.sequence,
			}
		})
		.collect();

	Ok(Json(media))
}

async fn upload_project_media(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
	mut multipart: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>), ProjectError> {
	sqlx::query_file!("queries/projects/check_project_exists.sql", id)
		.fetch_one(&state.model.db)
		.await?;

	let mut uploaded_s3_keys: Vec<String> = Vec::new();
	let mut uploaded_media_ids: Vec<Uuid> = Vec::new();

	let next_seq_row = sqlx::query_file!("queries/projects/get_next_project_media_sequence.sql", id)
		.fetch_one(&state.model.db)
		.await?;
	let mut current_seq = next_seq_row.next_seq;

	loop {
		let field = match multipart.next_field().await {
			Ok(Some(f)) => f,
			Ok(None) => break,
			Err(e) => {
				return Err(ProjectError::Validation(format!("Upload error: {e}")));
			}
		};

		let name = field.name().unwrap_or_default().to_string();
		let filename = field.file_name().unwrap_or_default().to_string();
		let content_type = field.content_type().map(|s| s.to_string());

		let bytes = field.bytes().await.map_err(|e| {
			ProjectError::Validation(format!("Failed to read file payload: {e}"))
		})?;

		if bytes.is_empty() {
			continue;
		}

		let is_video = crate::media::is_video_upload(&name, content_type.as_deref(), Some(&filename));

		let lower_name = filename.to_lowercase();
		if lower_name.ends_with(".svg") || content_type.as_deref() == Some("image/svg+xml") {
			return Err(ProjectError::Validation(
				"SVG format is not supported for gallery media. Please upload JPG, PNG, or WebP images.".to_string(),
			));
		}

		const MAX_IMAGE_BYTES: usize = 5 * 1024 * 1024; // 5 MB limit requested by user
		if !is_video && bytes.len() > MAX_IMAGE_BYTES {
			return Err(ProjectError::Validation(format!(
				"Image '{}' exceeds the 5 MB size limit ({:.2} MB uploaded).",
				filename,
				bytes.len() as f64 / 1_048_576.0
			)));
		}

		let upload_res: Result<(Uuid, Vec<String>), ProjectError> = if is_video {
			match crate::media::process_and_upload_video(
				&state.model.db,
				&state.s3_client,
				bytes.to_vec(),
				&filename,
				content_type.as_deref(),
			)
			.await
			{
				Ok(res) => {
					let mut keys = vec![res.s3_key];
					if let Some(tk) = res.thumbnail_key {
						keys.push(tk);
					}
					Ok((res.media_id, keys))
				}
				Err(e) => Err(ProjectError::Media(e)),
			}
		} else {
			match crate::media::process_and_upload_image(
				&state.model.db,
				&state.s3_client,
				bytes.to_vec(),
			)
			.await
			{
				Ok(res) => Ok((res.media_id, vec![res.original_key, res.thumbnail_key])),
				Err(e) => Err(ProjectError::Media(e)),
			}
		};

		match upload_res {
			Ok((mid, s3_keys)) => {
				uploaded_s3_keys.extend(s3_keys);
				uploaded_media_ids.push(mid);

				let insert_res = sqlx::query_file!(
					"queries/projects/insert_project_media.sql",
					id,
					mid,
					current_seq,
				)
				.execute(&state.model.db)
				.await;

				if let Err(e) = insert_res {
					for key in &uploaded_s3_keys {
						let _ = state
							.s3_client
							.delete_object()
							.bucket(&CONFIG.s3_bucket)
							.key(key)
							.send()
							.await;
					}
					for m_id in &uploaded_media_ids {
						let _ = sqlx::query_file!(
							"queries/projects/delete_project_media.sql",
							id,
							m_id,
						)
						.execute(&state.model.db)
						.await;
					}
					return Err(e.into());
				}
				current_seq += 1;
			}
			Err(e) => {
				for key in &uploaded_s3_keys {
					let _ = state
						.s3_client
						.delete_object()
						.bucket(&CONFIG.s3_bucket)
						.key(key)
						.send()
						.await;
				}
				for m_id in &uploaded_media_ids {
					let _ = sqlx::query_file!(
						"queries/projects/delete_project_media.sql",
						id,
						m_id,
					)
					.execute(&state.model.db)
					.await;
				}
				return Err(e);
			}
		}
	}

	if uploaded_media_ids.is_empty() {
		return Err(ProjectError::Validation("No media file provided in upload".to_string()));
	}

	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::ProjectUpdated(id))
		.await;

	Ok((
		StatusCode::CREATED,
		Json(json!({ "success": true, "media_ids": uploaded_media_ids })),
	))
}

#[derive(Deserialize, ts_rs::TS)]
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
	let mut tx = state.model.db.begin().await?;

	sqlx::query_file!(
		"queries/projects/shift_project_media_sequences_negative.sql",
		id
	)
	.execute(&mut *tx)
	.await?;

	for (idx, mid) in payload.order.into_iter().enumerate() {
		let seq = idx as i16;
		sqlx::query_file!(
			"queries/projects/update_project_media_sequence.sql",
			id,
			mid,
			seq
		)
		.execute(&mut *tx)
		.await?;
	}

	tx.commit().await?;
	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::ProjectUpdated(id))
		.await;

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

	crate::media::delete_media(&state.model.db, &state.s3_client, mid).await?;
	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::ProjectUpdated(id))
		.await;

	Ok(StatusCode::NO_CONTENT)
}

async fn project_documents_list(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
) -> Result<Json<Vec<ProjectDocumentItem>>, ProjectError> {
	sqlx::query_file!("queries/projects/check_project_exists.sql", id)
		.fetch_one(&state.model.db)
		.await?;

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
		.await?;

	let documents = doc_rows
		.into_iter()
		.map(|r| {
			let url = CONFIG.asset_url(&r.s3_key);
			let file_name = r
				.display_name
				.unwrap_or_else(|| r.s3_key.split('/').last().unwrap_or("document").to_string());
			ProjectDocumentItem {
				id: r.media_id,
				file_name,
				url,
				file_size: r.file_size,
				doc_type: r.doc_type.unwrap_or_else(|| "brochure".to_string()),
			}
		})
		.collect();

	Ok(Json(documents))
}

async fn upload_project_document(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
	mut multipart: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>), ProjectError> {
	sqlx::query_file!("queries/projects/check_project_exists.sql", id)
		.fetch_one(&state.model.db)
		.await?;

	let mut doc_type = "brochure".to_string();
	let mut display_name: Option<String> = None;
	let mut uploaded_file: Option<(String, String, Vec<u8>)> = None;

	while let Ok(Some(field)) = multipart.next_field().await {
		let name = field.name().unwrap_or_default().to_string();
		if name == "doc_type" {
			if let Ok(text) = field.text().await {
				let trimmed = text.trim();
				if matches!(trimmed, "rera" | "bu_approval" | "brochure" | "other") {
					doc_type = trimmed.to_string();
				}
			}
		} else if name == "display_name" {
			if let Ok(text) = field.text().await {
				let trimmed = text.trim();
				if !trimmed.is_empty() {
					display_name = Some(trimmed.to_string());
				}
			}
		} else {
			let filename = field.file_name().unwrap_or("document.pdf").to_string();
			let content_type = field.content_type().unwrap_or("application/pdf").to_string();
			if let Ok(b) = field.bytes().await {
				if !b.is_empty() {
					uploaded_file = Some((filename, content_type, b.to_vec()));
				}
			}
		}
	}

	if let Some((filename, content_type, bytes)) = uploaded_file {
		let doc_res = crate::media::upload_document(
			&state.model.db,
			&state.s3_client,
			&filename,
			&content_type,
			bytes,
		)
		.await
		.map_err(ProjectError::Media)?;

		let final_title = display_name.unwrap_or_else(|| filename.clone());

		sqlx::query_file!(
			"queries/projects/insert_project_document.sql",
			id,
			doc_res.media_id,
			Some(final_title.as_str()),
			Some(doc_type.as_str()),
		)
		.execute(&state.model.db)
		.await?;

		state
			.page_cache
			.invalidate(crate::cache::InvalidationEvent::ProjectUpdated(id))
			.await;

		return Ok((
			StatusCode::CREATED,
			Json(json!({
				"id": doc_res.media_id,
				"file_name": final_title,
				"url": doc_res.url,
				"file_size": doc_res.file_size,
				"doc_type": doc_type,
			})),
		));
	}

	Err(ProjectError::Validation("No document file provided".to_string()))
}

async fn delete_project_document(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path((id, did)): Path<(i32, Uuid)>,
) -> Result<StatusCode, ProjectError> {
	let res = sqlx::query_file!("queries/projects/delete_project_document.sql", did, id)
		.fetch_optional(&state.model.db)
		.await?;

	if let Some(doc) = res {
		crate::media::delete_media(&state.model.db, &state.s3_client, doc.media_id).await?;
	}
	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::ProjectUpdated(id))
		.await;

	Ok(StatusCode::NO_CONTENT)
}

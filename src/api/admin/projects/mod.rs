use askama::Template;
use axum::{
	Extension, Router,
	extract::{Multipart, Path, Query, State},
	http::StatusCode,
	response::{Html, Redirect},
	routing::get,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
	AppState,
	config::CONFIG,
	session_store::AuthUser,
	templates::admin::{
		AdminPage, AmenityOptionItem, LocationOptionItem, OwnerOptionItem, ProjectDetailData,
		ProjectDetailTemplate, ProjectDocumentItem, ProjectDocumentsTemplate, ProjectEditData,
		ProjectFormTemplate, ProjectListItem, ProjectMediaItem, ProjectMediaTemplate,
		ProjectPropertySummary, ProjectTemplate,
	},
	utils::{form::Form, markdown::render_markdown, sql::slugify},
};

pub mod error;
use error::ProjectError;

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/", get(project_list))
		.route("/new", get(project_create_form).post(create_project))
		.route("/{id}", get(project_detail).delete(delete_project))
		.route("/{id}/edit", get(project_edit_form).post(update_project))
		.route(
			"/{id}/media",
			get(project_media_page).post(upload_project_media),
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
			get(project_documents_page).post(upload_project_document),
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

// ---------------------------------------------------------------------------
// Query & Form payloads
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct ProjectListQuery {
	q: Option<String>,
	owner_id: Option<i32>,
	category: Option<String>,
}

#[derive(Deserialize)]
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

fn format_bytes(bytes: Option<i64>) -> String {
	let b = bytes.unwrap_or(0);
	if b < 1024 {
		format!("{b} B")
	} else if b < 1024 * 1024 {
		format!("{:.1} KB", b as f64 / 1024.0)
	} else {
		format!("{:.1} MB", b as f64 / (1024.0 * 1024.0))
	}
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn project_list(
	State(state): State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
	Query(query): Query<ProjectListQuery>,
) -> Result<Html<String>, ProjectError> {
	struct DbRow {
		id: i32,
		name: String,
		slug: String,
		category: String,
		owner_name: String,
		location: Option<String>,
		hero_thumb_key: Option<String>,
		properties_count: i64,
		media_count: i64,
	}

	let owners_rows = sqlx::query_file!("queries/project_owners/list_owner_options.sql")
		.fetch_all(&state.model.db)
		.await?;

	let owners = owners_rows
		.into_iter()
		.map(|r| OwnerOptionItem {
			id: r.id,
			name: r.name,
		})
		.collect();

	let rows = sqlx::query_file_as!(
		DbRow,
		"queries/projects/list_projects.sql",
		query.owner_id,
		query.category.as_deref(),
		query.q.as_ref().map(|q| format!("%{}%", q.trim())),
	)
	.fetch_all(&state.model.db)
	.await?;

	let projects = rows
		.into_iter()
		.map(|r| {
			let hero_thumb_url = r.hero_thumb_key.as_ref().map(|k| CONFIG.asset_url(k));
			ProjectListItem {
				id: r.id,
				name: r.name,
				slug: r.slug,
				owner_name: r.owner_name,
				category: r.category,
				location: r.location.unwrap_or_else(|| "Unspecified".to_string()),
				hero_thumb_url,
				properties_count: r.properties_count,
				media_count: r.media_count,
			}
		})
		.collect();

	Ok(Html(
		ProjectTemplate {
			admin_name: &auth_user.username,
			page: AdminPage::Projects,
			projects,
			owners,
			selected_owner_id: query.owner_id,
			selected_category: query.category,
			q: query.q.as_deref(),
		}
		.render()?,
	))
}

async fn project_create_form(
	State(state): State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
) -> Result<Html<String>, ProjectError> {
	let owners = sqlx::query_file!("queries/project_owners/list_owner_options.sql")
		.fetch_all(&state.model.db)
		.await?
		.into_iter()
		.map(|r| OwnerOptionItem {
			id: r.id,
			name: r.name,
		})
		.collect();

	let locations = sqlx::query_file!("queries/locations/list_location_options.sql")
		.fetch_all(&state.model.db)
		.await?
		.into_iter()
		.map(|r| LocationOptionItem {
			id: r.id,
			address: r.formatted_address,
		})
		.collect();

	let amenities = sqlx::query_file!("queries/amenities/list_active_amenities.sql")
		.fetch_all(&state.model.db)
		.await?
		.into_iter()
		.map(|r| AmenityOptionItem {
			id: r.id,
			name: r.name,
			selected: false,
		})
		.collect();

	Ok(Html(
		ProjectFormTemplate {
			admin_name: &auth_user.username,
			page: AdminPage::Projects,
			owners,
			locations,
			amenities,
			edit_project: None,
		}
		.render()?,
	))
}

async fn create_project(
	State(state): State<AppState>,
	Form(form): Form<ProjectFormPayload>,
) -> Result<Redirect, ProjectError> {
	let trimmed_name = form.name.trim();
	if trimmed_name.is_empty() {
		return Err(ProjectError::Validation(
			"Project name is required".to_string(),
		));
	}
	if form.category.trim().is_empty() {
		return Err(ProjectError::Validation("Category is required".to_string()));
	}
	if form.project_owner_id <= 0 {
		return Err(ProjectError::Validation(
			"Valid project owner is required".to_string(),
		));
	}
	if form.location_id <= 0 {
		return Err(ProjectError::Validation(
			"Valid location is required".to_string(),
		));
	}
	if form.amenities.iter().any(|&a| a <= 0) {
		return Err(ProjectError::Validation("Invalid amenity ID".to_string()));
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

	// Link selected amenities (deduplicating duplicate IDs)
	let mut seen = std::collections::HashSet::new();
	for amenity_id in form.amenities {
		if seen.insert(amenity_id) {
			sqlx::query_file!(
				"queries/projects/insert_project_amenity.sql",
				row.id,
				amenity_id,
			)
			.execute(&mut *tx)
			.await?;
		}
	}

	tx.commit().await?;
	tracing::info!(project_id = row.id, name = %form.name, "created new project");
	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::ProjectUpdated(row.id))
		.await;

	Ok(Redirect::to(&format!("/admin/projects/{}", row.id)))
}

async fn project_detail(
	State(state): State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
	Path(id): Path<i32>,
) -> Result<Html<String>, ProjectError> {
	struct DbProject {
		id: i32,
		project_owner_id: i32,
		owner_name: String,
		location_id: i32,
		location_name: Option<String>,
		name: String,
		slug: String,
		description: Option<String>,
		category: String,
		start_date: Option<time::Date>,
		launch_date: Option<time::Date>,
		possession_date: Option<time::Date>,
		created_at: time::OffsetDateTime,
	}

	let p = sqlx::query_file_as!(DbProject, "queries/projects/get_project_detail.sql", id)
		.fetch_one(&state.model.db)
		.await?;

	// Fetch amenities
	let amenity_rows = sqlx::query_file!("queries/projects/get_project_amenities.sql", id)
		.fetch_all(&state.model.db)
		.await?;

	let amenities = amenity_rows.into_iter().map(|r| r.name).collect();

	// Fetch media gallery
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

	// Fetch documents
	struct DbDoc {
		id: i32,
		media_id: Uuid,
		display_name: String,
		doc_type: String,
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
			let file_size_formatted = format_bytes(r.file_size);
			ProjectDocumentItem {
				id: r.id,
				media_id: r.media_id,
				display_name: r.display_name,
				doc_type: r.doc_type,
				url,
				file_size_formatted,
			}
		})
		.collect();

	// Fetch property breakdown
	struct PropCountRow {
		category: String,
		count: Option<i64>,
	}

	let prop_rows = sqlx::query_file_as!(
		PropCountRow,
		"queries/projects/get_project_property_counts.sql",
		id
	)
	.fetch_all(&state.model.db)
	.await?;

	let mut residential = 0;
	let mut commercial = 0;
	let mut land = 0;
	for r in prop_rows {
		let cnt = r.count.unwrap_or(0);
		match r.category.as_str() {
			"RESIDENTIAL" => residential += cnt,
			"COMMERCIAL" => commercial += cnt,
			"LAND" => land += cnt,
			_ => {}
		}
	}
	let total = residential + commercial + land;

	let description_html = p
		.description
		.as_ref()
		.map(|desc| render_markdown(desc))
		.unwrap_or_default();

	let date_fmt = time::macros::format_description!("[month repr:short] [day], [year]");
	let created_at_str = p
		.created_at
		.format(&date_fmt)
		.unwrap_or_else(|_| "Recently".to_string());

	let project_data = ProjectDetailData {
		id: p.id,
		owner_id: p.project_owner_id,
		owner_name: p.owner_name,
		location_id: p.location_id,
		location_name: p.location_name.unwrap_or_else(|| "Unspecified".to_string()),
		name: p.name,
		slug: p.slug,
		category: p.category,
		start_date: format_opt_date(p.start_date),
		launch_date: format_opt_date(p.launch_date),
		possession_date: format_opt_date(p.possession_date),
		created_at: created_at_str,
	};

	Ok(Html(
		ProjectDetailTemplate {
			admin_name: &auth_user.username,
			page: AdminPage::Projects,
			project: project_data,
			description_html,
			amenities,
			media,
			documents,
			property_summary: ProjectPropertySummary {
				total,
				residential,
				commercial,
				land,
			},
		}
		.render()?,
	))
}

async fn project_edit_form(
	State(state): State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
	Path(id): Path<i32>,
) -> Result<Html<String>, ProjectError> {
	let p = sqlx::query_file!("queries/projects/get_project_for_edit.sql", id)
		.fetch_one(&state.model.db)
		.await?;

	let owners = sqlx::query_file!("queries/project_owners/list_owner_options.sql")
		.fetch_all(&state.model.db)
		.await?
		.into_iter()
		.map(|r| OwnerOptionItem {
			id: r.id,
			name: r.name,
		})
		.collect();

	let locations = sqlx::query_file!("queries/locations/list_location_options.sql")
		.fetch_all(&state.model.db)
		.await?
		.into_iter()
		.map(|r| LocationOptionItem {
			id: r.id,
			address: r.formatted_address,
		})
		.collect();

	let linked_amenities: std::collections::HashSet<i32> =
		sqlx::query_file!("queries/projects/get_project_linked_amenity_ids.sql", id)
			.fetch_all(&state.model.db)
			.await?
			.into_iter()
			.map(|r| r.amenity_id)
			.collect();

	let amenities = sqlx::query_file!("queries/amenities/list_active_amenities.sql")
		.fetch_all(&state.model.db)
		.await?
		.into_iter()
		.map(|r| AmenityOptionItem {
			selected: linked_amenities.contains(&r.id),
			id: r.id,
			name: r.name,
		})
		.collect();

	let edit_data = ProjectEditData {
		id: p.id,
		owner_id: p.project_owner_id,
		location_id: p.location_id,
		name: p.name,
		description: p.description,
		category: p.category,
		start_date: format_opt_date(p.start_date),
		launch_date: format_opt_date(p.launch_date),
		possession_date: format_opt_date(p.possession_date),
	};

	Ok(Html(
		ProjectFormTemplate {
			admin_name: &auth_user.username,
			page: AdminPage::Projects,
			owners,
			locations,
			amenities,
			edit_project: Some(edit_data),
		}
		.render()?,
	))
}

async fn update_project(
	State(state): State<AppState>,
	Path(id): Path<i32>,
	Form(form): Form<ProjectFormPayload>,
) -> Result<Redirect, ProjectError> {
	let trimmed_name = form.name.trim();
	if trimmed_name.is_empty() {
		return Err(ProjectError::Validation(
			"Project name is required".to_string(),
		));
	}
	if form.category.trim().is_empty() {
		return Err(ProjectError::Validation("Category is required".to_string()));
	}
	if form.project_owner_id <= 0 {
		return Err(ProjectError::Validation(
			"Valid project owner is required".to_string(),
		));
	}
	if form.location_id <= 0 {
		return Err(ProjectError::Validation(
			"Valid location is required".to_string(),
		));
	}
	if form.amenities.iter().any(|&a| a <= 0) {
		return Err(ProjectError::Validation("Invalid amenity ID".to_string()));
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

	// Sync amenities (deduplicating duplicate IDs)
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

	Ok(Redirect::to(&format!("/admin/projects/{id}")))
}

async fn delete_project(
	State(state): State<AppState>,
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

async fn project_media_page(
	State(state): State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
	Path(id): Path<i32>,
) -> Result<Html<String>, ProjectError> {
	let project = sqlx::query_file!("queries/projects/check_project_exists.sql", id)
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

	Ok(Html(
		ProjectMediaTemplate {
			admin_name: &auth_user.username,
			page: AdminPage::Projects,
			project_id: project.id,
			project_name: project.name,
			media,
		}
		.render()?,
	))
}

async fn upload_project_media(
	State(state): State<AppState>,
	Path(id): Path<i32>,
	mut multipart: Multipart,
) -> Result<Redirect, ProjectError> {
	// Verify project exists
	sqlx::query_file!("queries/projects/check_project_exists.sql", id)
		.fetch_one(&state.model.db)
		.await?;

	let mut uploaded_s3_keys: Vec<String> = Vec::new();
	let mut uploaded_media_ids: Vec<Uuid> = Vec::new();

	// Calculate base sequence
	let next_seq_row =
		sqlx::query_file!("queries/projects/get_next_project_media_sequence.sql", id)
			.fetch_one(&state.model.db)
			.await?;
	let mut current_seq = next_seq_row.next_seq;

	let mut field_found = false;

	while let Ok(Some(field)) = multipart.next_field().await {
		let name = field.name().unwrap_or_default().to_string();
		if !matches!(
			name.as_str(),
			"image" | "video" | "media" | "file" | "files" | "files[]"
		) {
			continue;
		}

		let filename = field.file_name().unwrap_or_default().to_string();
		let content_type = field.content_type().map(|s| s.to_string());

		let bytes = match field.bytes().await {
			Ok(b) if !b.is_empty() => b,
			_ => continue,
		};

		field_found = true;
		let is_video =
			crate::media::is_video_upload(&name, content_type.as_deref(), Some(&filename));

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
							m_id
						)
						.execute(&state.model.db)
						.await;
						let _ = sqlx::query!("DELETE FROM media WHERE id = $1", m_id)
							.execute(&state.model.db)
							.await;
					}
					return Err(ProjectError::from(e));
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
					let _ =
						sqlx::query_file!("queries/projects/delete_project_media.sql", id, m_id)
							.execute(&state.model.db)
							.await;
					let _ = sqlx::query!("DELETE FROM media WHERE id = $1", m_id)
						.execute(&state.model.db)
						.await;
				}
				return Err(e);
			}
		}
	}

	if !field_found || uploaded_media_ids.is_empty() {
		return Err(ProjectError::Validation(
			"No media file provided in upload".to_string(),
		));
	}

	tracing::info!(
		project_id = id,
		count = uploaded_media_ids.len(),
		"attached media to project"
	);
	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::ProjectUpdated(id))
		.await;
	Ok(Redirect::to(&format!("/admin/projects/{id}/media")))
}

#[derive(serde::Deserialize)]
pub struct ReorderMediaPayload {
	pub order: Vec<Uuid>,
}

async fn reorder_project_media(
	State(state): State<AppState>,
	Path(id): Path<i32>,
	axum::Json(payload): axum::Json<ReorderMediaPayload>,
) -> Result<StatusCode, ProjectError> {
	let mut tx = state.model.db.begin().await?;

	// 1. Shift existing sequences to negative
	sqlx::query_file!(
		"queries/projects/shift_project_media_sequences_negative.sql",
		id
	)
	.execute(&mut *tx)
	.await?;

	// 2. Set new sequences
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
	Ok(StatusCode::OK)
}

async fn delete_project_media(
	State(state): State<AppState>,
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

async fn project_documents_page(
	State(state): State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
	Path(id): Path<i32>,
) -> Result<Html<String>, ProjectError> {
	let project = sqlx::query_file!("queries/projects/check_project_exists.sql", id)
		.fetch_one(&state.model.db)
		.await?;

	struct DbDoc {
		id: i32,
		media_id: Uuid,
		display_name: String,
		doc_type: String,
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
			let file_size_formatted = format_bytes(r.file_size);
			ProjectDocumentItem {
				id: r.id,
				media_id: r.media_id,
				display_name: r.display_name,
				doc_type: r.doc_type,
				url,
				file_size_formatted,
			}
		})
		.collect();

	Ok(Html(
		ProjectDocumentsTemplate {
			admin_name: &auth_user.username,
			page: AdminPage::Projects,
			project_id: project.id,
			project_name: project.name,
			documents,
		}
		.render()?,
	))
}

async fn upload_project_document(
	State(state): State<AppState>,
	Path(id): Path<i32>,
	mut multipart: Multipart,
) -> Result<Redirect, ProjectError> {
	let mut display_name = String::new();
	let mut doc_type = String::from("other");
	let mut file_bytes: Option<Vec<u8>> = None;
	let mut file_name = String::from("document.pdf");
	let mut mime_type = String::from("application/pdf");

	while let Ok(Some(field)) = multipart.next_field().await {
		match field.name() {
			Some("display_name") => {
				display_name = field.text().await.unwrap_or_default();
			}
			Some("doc_type") => {
				doc_type = field.text().await.unwrap_or_default();
			}
			Some("file") => {
				if let Some(name) = field.file_name() {
					file_name = name.to_string();
				}
				if let Some(ct) = field.content_type() {
					mime_type = ct.to_string();
				}
				file_bytes = Some(
					field
						.bytes()
						.await
						.map_err(|e| ProjectError::Internal(e.to_string()))?
						.to_vec(),
				);
			}
			_ => {}
		}
	}

	let bytes = file_bytes
		.ok_or_else(|| ProjectError::Validation("No document file attached".to_string()))?;

	if display_name.trim().is_empty() {
		display_name = file_name.clone();
	}

	let uploaded = crate::media::upload_document(
		&state.model.db,
		&state.s3_client,
		&file_name,
		&mime_type,
		bytes,
	)
	.await?;

	sqlx::query_file!(
		"queries/projects/insert_project_document.sql",
		id,
		uploaded.media_id,
		display_name.trim(),
		doc_type.trim(),
	)
	.execute(&state.model.db)
	.await?;

	tracing::info!(project_id = id, doc_name = %display_name, "uploaded project document");
	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::ProjectUpdated(id))
		.await;
	Ok(Redirect::to(&format!("/admin/projects/{id}/documents")))
}

async fn delete_project_document(
	State(state): State<AppState>,
	Path((id, did)): Path<(i32, i32)>,
) -> Result<StatusCode, ProjectError> {
	let row = sqlx::query_file!("queries/projects/delete_project_document.sql", did, id)
		.fetch_optional(&state.model.db)
		.await?;

	if let Some(r) = row {
		crate::media::delete_media(&state.model.db, &state.s3_client, r.media_id).await?;
	}

	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::ProjectUpdated(id))
		.await;
	Ok(StatusCode::NO_CONTENT)
}

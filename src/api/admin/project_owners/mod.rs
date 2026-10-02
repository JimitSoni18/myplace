use askama::Template;
use aws_sdk_s3::primitives::ByteStream;
use axum::{
	Extension, Form, Router,
	extract::{Multipart, Path, Query, State},
	response::{Html, Redirect},
	routing::{get, post},
};
use image::ImageReader;
use serde::Deserialize;
use webp::Encoder;

use crate::{
	AppState,
	api::admin::project_owners::error::{
		OwnerCreateFormGetError, OwnerDetailError, OwnerProjectsError, OwnerToggleActiveError,
		OwnerUpdateFormGetError, ProjectOwnerCreateError, ProjectOwnerDeleteError,
		ProjectOwnerImageUploadError, ProjectOwnerListError, ProjectOwnerUpdateError,
	},
	config::CONFIG,
	crypto::password_hash,
	session_store::AuthUser,
	templates::admin::{
		AdminPage, EditFormValues, OwnerProjectItem, ProjectOwnerDetailItem,
		ProjectOwnerDetailTemplate, ProjectOwnerEditItem, ProjectOwnerFormTemplate,
		ProjectOwnerProjectsTemplate, ProjectOwnerTemplate,
	},
	utils::{self, sql::slugify},
};

pub mod error;

// ---------------------------------------------------------------------------
// Public types (re-used by templates module)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct ProjectListQuery {
	q: Option<String>,
}

/// Row for the owner list page.
pub struct ProjectList {
	pub id: i32,
	pub name: String,
	pub active: bool,
	pub profile_img_thumb_key: Option<String>,
	pub profile_thumb_url: Option<String>,
	pub projects_count: i64,
	pub email: Option<String>,
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/", get(project_owner_list))
		.route(
			"/new",
			get(project_owner_create_form).post(create_project_owner),
		)
		.route(
			"/{id}",
			get(project_owner_detail).delete(delete_project_owner),
		)
		.route(
			"/{id}/edit",
			get(project_owner_update_form).post(update_project_owner),
		)
		.route("/{id}/toggle-active", post(toggle_active_project_owner))
		.route("/{id}/upload-image", post(add_project_owner_image))
		.route("/{id}/remove-image", post(remove_project_owner_image))
		.route("/{id}/projects", get(project_owner_projects))
}

// ---------------------------------------------------------------------------
// Form payloads
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct ProjectOwnerCreatePayload {
	pub name: String,
	pub username: String,
	pub password: String,
	pub bio: Option<String>,
	pub email: Option<String>,
	pub phone: Option<String>,
	pub website: Option<String>,
	pub active: Option<String>,
}

#[derive(Deserialize)]
pub struct ProjectOwnerUpdatePayload {
	pub name: String,
	pub bio: Option<String>,
	pub email: Option<String>,
	pub phone: Option<String>,
	pub website: Option<String>,
	pub active: Option<String>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn project_owner_list(
	state: State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
	Query(query): Query<ProjectListQuery>,
) -> Result<Html<String>, ProjectOwnerListError> {
	struct DbRow {
		id: i32,
		name: String,
		active: bool,
		profile_img_thumb_key: Option<String>,
		projects_count: i64,
		email: Option<String>,
	}

	let (q, rows) = match query.q {
		Some(ref query_str) if !query_str.is_empty() => {
			let escaped = utils::sql::escape(query_str.trim());
			let rgx = format!("%{escaped}%");
			let db_rows =
				sqlx::query_file_as!(DbRow, "queries/project_owners/search_owners.sql", rgx)
					.fetch_all(&state.model.db)
					.await
					.map_err(ProjectOwnerListError::SqlError)?;

			(query.q, db_rows)
		}
		_ => {
			let db_rows = sqlx::query_file_as!(DbRow, "queries/project_owners/list_owners.sql")
				.fetch_all(&state.model.db)
				.await
				.map_err(ProjectOwnerListError::SqlError)?;

			(None, db_rows)
		}
	};

	let owners = rows
		.into_iter()
		.map(|r| {
			let profile_thumb_url = r
				.profile_img_thumb_key
				.as_ref()
				.map(|k| CONFIG.asset_url(k));
			ProjectList {
				id: r.id,
				name: r.name,
				active: r.active,
				profile_img_thumb_key: r.profile_img_thumb_key,
				profile_thumb_url,
				projects_count: r.projects_count,
				email: r.email,
			}
		})
		.collect();

	Ok(Html(
		ProjectOwnerTemplate {
			admin_name: &auth_user.username,
			page: AdminPage::Owners,
			owners,
			q: q.as_deref(),
		}
		.render()
		.map_err(ProjectOwnerListError::RenderError)?,
	))
}

async fn project_owner_create_form(
	Extension(auth_user): Extension<AuthUser>,
) -> Result<Html<String>, OwnerCreateFormGetError> {
	Ok(Html(
		ProjectOwnerFormTemplate {
			admin_name: &auth_user.username,
			page: AdminPage::Owners,
			edit_values: None,
		}
		.render()
		.map_err(OwnerCreateFormGetError::RenderError)?,
	))
}

async fn create_project_owner(
	State(state): State<AppState>,
	Form(form): Form<ProjectOwnerCreatePayload>,
) -> Result<Redirect, ProjectOwnerCreateError> {
	let slug = slugify(&form.name);
	let is_active = form.active.as_deref() == Some("on");

	// Hash the owner password using Argon2id
	let password_hash =
		password_hash::hash(&form.password).map_err(|_| ProjectOwnerCreateError::HashingError)?;

	// Run in a single transaction: profiles + project_owners
	let mut tx = state
		.model
		.db
		.begin()
		.await
		.map_err(ProjectOwnerCreateError::SqlError)?;

	let profile_row = sqlx::query_file!(
		"queries/project_owners/insert_profile.sql",
		form.username.trim(),
		password_hash,
	)
	.fetch_one(&mut *tx)
	.await
	.map_err(ProjectOwnerCreateError::SqlError)?;

	let owner_row = sqlx::query_file!(
		"queries/project_owners/insert_owner.sql",
		profile_row.id,
		form.name.trim(),
		slug,
		form.bio.as_deref(),
		form.email.as_deref(),
		form.phone.as_deref(),
		form.website.as_deref(),
		is_active,
	)
	.fetch_one(&mut *tx)
	.await
	.map_err(ProjectOwnerCreateError::SqlError)?;

	tx.commit()
		.await
		.map_err(ProjectOwnerCreateError::SqlError)?;

	tracing::info!(
		owner_id = owner_row.id,
		name = %form.name,
		username = %form.username,
		"created project owner account"
	);

	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::OwnerUpdated(owner_row.id))
		.await;

	Ok(Redirect::to(&format!("/admin/owners/{}", owner_row.id)))
}

async fn project_owner_detail(
	State(state): State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
	Path(id): Path<i32>,
) -> Result<Html<String>, OwnerDetailError> {
	struct OwnerDbRow {
		id: i32,
		name: String,
		slug: String,
		username: Option<String>,
		bio: Option<String>,
		email: Option<String>,
		phone: Option<String>,
		website: Option<String>,
		active: bool,
		profile_img_key: Option<String>,
		profile_img_thumb_key: Option<String>,
		created_at: time::OffsetDateTime,
		projects_count: i64,
	}

	let owner_row = sqlx::query_file_as!(
		OwnerDbRow,
		"queries/project_owners/get_owner_detail.sql",
		id
	)
	.fetch_one(&state.model.db)
	.await
	.map_err(|e| match e {
		sqlx::Error::RowNotFound => OwnerDetailError::NotFound,
		_ => OwnerDetailError::SqlError(e),
	})?;

	struct ProjectDbRow {
		id: i32,
		name: String,
		slug: String,
		category: String,
		location: Option<String>,
		properties_count: i64,
	}

	let project_rows = sqlx::query_file_as!(
		ProjectDbRow,
		"queries/project_owners/get_owner_projects_summary.sql",
		id
	)
	.fetch_all(&state.model.db)
	.await
	.map_err(OwnerDetailError::SqlError)?;

	let profile_image_url = owner_row
		.profile_img_key
		.as_ref()
		.map(|k| CONFIG.asset_url(k));
	let profile_thumb_url = owner_row
		.profile_img_thumb_key
		.as_ref()
		.map(|k| CONFIG.asset_url(k));

	let date_fmt = time::macros::format_description!("[month repr:short] [day], [year]");
	let created_at_str = owner_row
		.created_at
		.format(&date_fmt)
		.unwrap_or_else(|_| "Recently".to_string());

	let detail_item = ProjectOwnerDetailItem {
		id: owner_row.id,
		name: owner_row.name,
		slug: owner_row.slug,
		username: owner_row
			.username
			.unwrap_or_else(|| "unassigned".to_string()),
		bio: owner_row.bio,
		email: owner_row.email,
		phone: owner_row.phone,
		website: owner_row.website,
		active: owner_row.active,
		profile_image_url,
		profile_thumb_url,
		created_at: created_at_str,
		projects_count: owner_row.projects_count,
	};

	let recent_projects = project_rows
		.into_iter()
		.map(|p| OwnerProjectItem {
			id: p.id,
			name: p.name,
			slug: p.slug,
			category: p.category,
			location: p.location.unwrap_or_else(|| "Unspecified".to_string()),
			properties_count: p.properties_count,
		})
		.collect();

	Ok(Html(
		ProjectOwnerDetailTemplate {
			admin_name: &auth_user.username,
			page: AdminPage::Owners,
			owner: detail_item,
			recent_projects,
		}
		.render()
		.map_err(OwnerDetailError::RenderError)?,
	))
}

async fn project_owner_update_form(
	State(state): State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
	Path(id): Path<i32>,
) -> Result<Html<String>, OwnerUpdateFormGetError> {
	struct DbRow {
		id: i32,
		name: String,
		bio: Option<String>,
		username: Option<String>,
		email: Option<String>,
		phone: Option<String>,
		website: Option<String>,
		active: bool,
		profile_img_key: Option<String>,
		profile_img_thumb_key: Option<String>,
	}

	let owner = sqlx::query_file_as!(DbRow, "queries/project_owners/get_owner_for_edit.sql", id)
		.fetch_one(&state.model.db)
		.await
		.map_err(|e| match e {
			sqlx::Error::RowNotFound => OwnerUpdateFormGetError::OwnerNotFound,
			_ => OwnerUpdateFormGetError::SqlError(e),
		})?;

	let profile_image_url = owner.profile_img_key.as_ref().map(|k| CONFIG.asset_url(k));
	let profile_thumb_url = owner
		.profile_img_thumb_key
		.as_ref()
		.map(|k| CONFIG.asset_url(k));

	Ok(Html(
		ProjectOwnerFormTemplate {
			admin_name: &auth_user.username,
			page: AdminPage::Owners,
			edit_values: Some(EditFormValues {
				default_values: ProjectOwnerEditItem {
					id: owner.id,
					name: owner.name,
					bio: owner.bio,
					username: owner
						.username
						.unwrap_or_else(|| format!("owner_{}", owner.id)),
					email: owner.email,
					phone: owner.phone,
					website: owner.website,
					active: owner.active,
					profile_image_url,
					profile_thumb_url,
				},
				error: None,
			}),
		}
		.render()
		.map_err(OwnerUpdateFormGetError::RenderError)?,
	))
}

async fn update_project_owner(
	State(state): State<AppState>,
	Path(id): Path<i32>,
	Form(form): Form<ProjectOwnerUpdatePayload>,
) -> Result<Redirect, ProjectOwnerUpdateError> {
	let slug = slugify(&form.name);
	let is_active = form.active.as_deref() == Some("on");

	sqlx::query_file!(
		"queries/project_owners/update_owner.sql",
		form.name.trim(),
		form.bio.as_deref(),
		form.email.as_deref(),
		form.phone.as_deref(),
		form.website.as_deref(),
		is_active,
		slug,
		id,
	)
	.execute(&state.model.db)
	.await
	.map_err(ProjectOwnerUpdateError::SqlError)?;

	tracing::info!(owner_id = id, "updated project owner profile");
	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::OwnerUpdated(id))
		.await;
	Ok(Redirect::to(&format!("/admin/owners/{id}")))
}

async fn toggle_active_project_owner(
	State(state): State<AppState>,
	Path(id): Path<i32>,
) -> Result<Redirect, OwnerToggleActiveError> {
	sqlx::query_file!("queries/project_owners/toggle_active.sql", id)
		.execute(&state.model.db)
		.await
		.map_err(OwnerToggleActiveError::SqlError)?;

	tracing::info!(owner_id = id, "toggled owner active status");
	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::OwnerUpdated(id))
		.await;
	Ok(Redirect::to(&format!("/admin/owners/{id}")))
}

async fn project_owner_projects(
	State(state): State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
	Path(id): Path<i32>,
) -> Result<Html<String>, OwnerProjectsError> {
	let owner = sqlx::query_file!("queries/project_owners/get_owner_name_by_id.sql", id)
		.fetch_one(&state.model.db)
		.await
		.map_err(|e| match e {
			sqlx::Error::RowNotFound => OwnerProjectsError::NotFound,
			_ => OwnerProjectsError::SqlError(e),
		})?;

	struct ProjectDbRow {
		id: i32,
		name: String,
		slug: String,
		category: String,
		location: Option<String>,
		properties_count: i64,
	}

	let project_rows = sqlx::query_file_as!(
		ProjectDbRow,
		"queries/project_owners/get_owner_projects_all.sql",
		id
	)
	.fetch_all(&state.model.db)
	.await
	.map_err(OwnerProjectsError::SqlError)?;

	let projects = project_rows
		.into_iter()
		.map(|p| OwnerProjectItem {
			id: p.id,
			name: p.name,
			slug: p.slug,
			category: p.category,
			location: p.location.unwrap_or_else(|| "Unspecified".to_string()),
			properties_count: p.properties_count,
		})
		.collect();

	Ok(Html(
		ProjectOwnerProjectsTemplate {
			admin_name: &auth_user.username,
			page: AdminPage::Owners,
			owner_id: owner.id,
			owner_name: owner.name,
			projects,
		}
		.render()
		.map_err(OwnerProjectsError::RenderError)?,
	))
}

pub async fn add_project_owner_image(
	State(state): State<AppState>,
	Path(id): Path<i32>,
	mut multipart: Multipart,
) -> Result<Redirect, ProjectOwnerImageUploadError> {
	while let Ok(Some(field)) = multipart.next_field().await {
		if field.name() != Some("image") {
			continue;
		}

		let bytes = field
			.bytes()
			.await
			.map_err(|_| ProjectOwnerImageUploadError::FileReadError)?;

		let (image_data, thumb_data) = tokio::task::spawn_blocking(move || {
			let reader = ImageReader::new(std::io::Cursor::new(bytes))
				.with_guessed_format()
				.map_err(|_| ProjectOwnerImageUploadError::FileFormatNotSupported)?;

			let img = reader
				.decode()
				.map_err(ProjectOwnerImageUploadError::ImageDecodeError)?;

			let webp = Encoder::from_image(&img)
				.map_err(|e| ProjectOwnerImageUploadError::EncoderCreateError(e.to_string()))?
				.encode(80.0);

			let thumb_img = img.thumbnail(300, 300);
			let thumb_webp = Encoder::from_image(&thumb_img)
				.map_err(|e| ProjectOwnerImageUploadError::EncoderCreateError(e.to_string()))?
				.encode(80.0);

			Ok::<_, ProjectOwnerImageUploadError>((webp.to_vec(), thumb_webp.to_vec()))
		})
		.await
		.map_err(|_| ProjectOwnerImageUploadError::Internal)??;

		let base_path = format!("owners/{id}");
		let thumb_key = format!("{base_path}/thumbnail.webp");
		let original_img_key = format!("{base_path}/original.webp");

		state
			.s3_client
			.put_object()
			.bucket(&CONFIG.s3_bucket)
			.key(&thumb_key)
			.body(ByteStream::from(thumb_data))
			.send()
			.await
			.map_err(|_| ProjectOwnerImageUploadError::Internal)?;

		state
			.s3_client
			.put_object()
			.bucket(&CONFIG.s3_bucket)
			.key(&original_img_key)
			.body(ByteStream::from(image_data))
			.send()
			.await
			.map_err(|_| ProjectOwnerImageUploadError::Internal)?;

		sqlx::query_file!(
			"queries/project_owners/update_owner_image.sql",
			original_img_key,
			thumb_key,
			id,
		)
		.execute(&state.model.db)
		.await
		.map_err(|_| ProjectOwnerImageUploadError::ImageUpdateError)?;

		tracing::info!(owner_id = id, key = %original_img_key, "uploaded owner profile image");
		state
			.page_cache
			.invalidate(crate::cache::InvalidationEvent::OwnerUpdated(id))
			.await;
		return Ok(Redirect::to(&format!("/admin/owners/{id}")));
	}

	Err(ProjectOwnerImageUploadError::ImageFieldNotPresent)
}

async fn remove_project_owner_image(
	State(state): State<AppState>,
	Path(id): Path<i32>,
) -> Result<Redirect, OwnerUpdateFormGetError> {
	let owner = sqlx::query_file!("queries/project_owners/get_owner_image_keys.sql", id)
		.fetch_one(&state.model.db)
		.await
		.map_err(OwnerUpdateFormGetError::SqlError)?;

	// Best-effort S3 object removal
	if let Some(key) = owner.profile_img_key {
		let _ = state
			.s3_client
			.delete_object()
			.bucket(&CONFIG.s3_bucket)
			.key(&key)
			.send()
			.await;
	}
	if let Some(key) = owner.profile_img_thumb_key {
		let _ = state
			.s3_client
			.delete_object()
			.bucket(&CONFIG.s3_bucket)
			.key(&key)
			.send()
			.await;
	}

	sqlx::query_file!("queries/project_owners/delete_owner_image.sql", id)
		.execute(&state.model.db)
		.await
		.map_err(OwnerUpdateFormGetError::SqlError)?;

	tracing::info!(owner_id = id, "removed owner profile image");
	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::OwnerUpdated(id))
		.await;
	Ok(Redirect::to(&format!("/admin/owners/{id}/edit")))
}

/// Soft delete — sets `deleted_at` rather than removing the row.
async fn delete_project_owner(
	State(state): State<AppState>,
	Path(id): Path<i32>,
) -> Result<Redirect, ProjectOwnerDeleteError> {
	sqlx::query_file!("queries/project_owners/delete_owner.sql", id)
		.execute(&state.model.db)
		.await
		.map_err(ProjectOwnerDeleteError::SqlError)?;

	tracing::info!(owner_id = id, "soft-deleted project owner");
	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::OwnerUpdated(id))
		.await;
	Ok(Redirect::to("/admin/owners"))
}

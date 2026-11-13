use aws_sdk_s3::primitives::ByteStream;
use axum::{
	Json, Router,
	extract::{Multipart, Path, Query, State},
	http::StatusCode,
	routing::{get, post},
};
use image::ImageReader;
use serde::{Deserialize, Serialize};
use serde_json::json;
use webp::Encoder;

use crate::{
	AppState,
	config::CONFIG,
	crypto::password_hash,
	session_store::AuthUser,
	utils::{self, sql::slugify},
};

pub mod error;

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/", get(project_owner_list).post(create_project_owner))
		.route(
			"/{id}",
			get(project_owner_detail)
				.put(update_project_owner)
				.delete(delete_project_owner),
		)
		.route("/{id}/toggle-active", post(toggle_active_project_owner))
		.route("/{id}/upload-image", post(add_project_owner_image))
		.route(
			"/{id}/image",
			post(add_project_owner_image).delete(remove_project_owner_image),
		)
		.route("/{id}/remove-image", post(remove_project_owner_image))
		.route("/{id}/projects", get(project_owner_projects))
}

#[derive(Deserialize, ts_rs::TS)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct OwnerListQuery {
	#[serde(default)]
	pub q: Option<String>,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct OwnerListItem {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub active: bool,
	pub email: Option<String>,
	pub phone: Option<String>,
	pub website: Option<String>,
	pub profile_thumb_url: Option<String>,
	pub projects_count: i64,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "admin/")]
pub struct OwnerProjectSummary {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub project_type_name: String,
	pub city: String,
	pub area: String,
	pub formatted_address: String,
	pub configurations_count: i64,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct OwnerDetailResponse {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub username: String,
	pub bio: Option<String>,
	pub email: Option<String>,
	pub phone: Option<String>,
	pub website: Option<String>,
	pub active: bool,
	pub profile_image_url: Option<String>,
	pub profile_thumb_url: Option<String>,
	pub created_at: String,
	pub projects_count: i64,
	pub projects: Vec<OwnerProjectSummary>,
}

#[derive(Deserialize, ts_rs::TS)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct ProjectOwnerCreatePayload {
	pub name: String,
	pub username: String,
	pub password: String,
	pub bio: Option<String>,
	pub email: Option<String>,
	pub phone: Option<String>,
	pub website: Option<String>,
	pub active: Option<bool>,
}

#[derive(Deserialize, ts_rs::TS)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct ProjectOwnerUpdatePayload {
	pub name: String,
	pub bio: Option<String>,
	pub email: Option<String>,
	pub phone: Option<String>,
	pub website: Option<String>,
	pub active: Option<bool>,
}

async fn project_owner_list(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Query(query): Query<OwnerListQuery>,
) -> Result<Json<Vec<OwnerListItem>>, (StatusCode, Json<serde_json::Value>)> {
	#[derive(sqlx::FromRow)]
	struct DbRow {
		id: i32,
		name: String,
		slug: String,
		active: bool,
		profile_img_thumb_key: Option<String>,
		projects_count: i64,
		email: Option<String>,
		phone: Option<String>,
		website: Option<String>,
	}

	let rows = match query.q.filter(|s| !s.trim().is_empty()) {
		Some(ref q) => {
			let escaped = utils::sql::escape(q.trim());
			let pattern = format!("%{escaped}%");
			sqlx::query_as::<_, DbRow>(
				r#"
				SELECT po.id, po.name, po.slug, po.active, po.profile_img_thumb_key,
				       po.email, po.phone, po.website,
				       COUNT(p.id) as projects_count
				FROM project_owners po
				LEFT JOIN projects p ON p.project_owner_id = po.id AND p.deleted_at IS NULL
				WHERE po.deleted_at IS NULL AND (po.name ILIKE $1 OR po.slug ILIKE $1)
				GROUP BY po.id
				ORDER BY po.id DESC
				"#,
			)
			.bind(&pattern)
			.fetch_all(&state.model.db)
			.await
		}
		None => {
			sqlx::query_as::<_, DbRow>(
				r#"
				SELECT po.id, po.name, po.slug, po.active, po.profile_img_thumb_key,
				       po.email, po.phone, po.website,
				       COUNT(p.id) as projects_count
				FROM project_owners po
				LEFT JOIN projects p ON p.project_owner_id = po.id AND p.deleted_at IS NULL
				WHERE po.deleted_at IS NULL
				GROUP BY po.id
				ORDER BY po.id DESC
				"#,
			)
			.fetch_all(&state.model.db)
			.await
		}
	}
	.map_err(|e| {
		tracing::error!(error = ?e, "failed to fetch project owners");
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(json!({ "code": "INTERNAL_ERROR", "message": "Failed to fetch project owners" })),
		)
	})?;

	let items = rows
		.into_iter()
		.map(|r| OwnerListItem {
			id: r.id,
			name: r.name,
			slug: r.slug,
			active: r.active,
			email: r.email,
			phone: r.phone,
			website: r.website,
			profile_thumb_url: r.profile_img_thumb_key.map(|k| CONFIG.asset_url(&k)),
			projects_count: r.projects_count,
		})
		.collect();

	Ok(Json(items))
}

async fn project_owner_detail(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
) -> Result<Json<OwnerDetailResponse>, (StatusCode, Json<serde_json::Value>)> {
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
		sqlx::Error::RowNotFound => (
			StatusCode::NOT_FOUND,
			Json(json!({ "code": "NOT_FOUND", "message": "Owner not found" })),
		),
		_ => (
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(json!({ "code": "INTERNAL_ERROR", "message": "Database query error" })),
		),
	})?;

	struct ProjectDbRow {
		id: i32,
		name: String,
		slug: String,
		project_type_name: String,
		city: String,
		area: String,
		formatted_address: String,
		configurations_count: i64,
	}

	let project_rows = sqlx::query_file_as!(
		ProjectDbRow,
		"queries/project_owners/get_owner_projects_summary.sql",
		id
	)
	.fetch_all(&state.model.db)
	.await
	.unwrap_or_default();

	let date_fmt = time::macros::format_description!("[month repr:short] [day], [year]");
	let created_at_str = owner_row
		.created_at
		.format(&date_fmt)
		.unwrap_or_else(|_| "Recently".to_string());

	Ok(Json(OwnerDetailResponse {
		id: owner_row.id,
		name: owner_row.name,
		slug: owner_row.slug,
		username: owner_row.username.unwrap_or_else(|| "unassigned".to_string()),
		bio: owner_row.bio,
		email: owner_row.email,
		phone: owner_row.phone,
		website: owner_row.website,
		active: owner_row.active,
		profile_image_url: owner_row.profile_img_key.map(|k| CONFIG.asset_url(&k)),
		profile_thumb_url: owner_row.profile_img_thumb_key.map(|k| CONFIG.asset_url(&k)),
		created_at: created_at_str,
		projects_count: owner_row.projects_count,
		projects: project_rows
			.into_iter()
			.map(|p| OwnerProjectSummary {
				id: p.id,
				name: p.name,
				slug: p.slug,
				project_type_name: p.project_type_name,
				city: p.city,
				area: p.area,
				formatted_address: p.formatted_address,
				configurations_count: p.configurations_count,
			})
			.collect(),
	}))
}

async fn create_project_owner(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Json(payload): Json<ProjectOwnerCreatePayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
	let name = payload.name.trim();
	let username = payload.username.trim();

	if name.is_empty() || username.is_empty() || payload.password.is_empty() {
		return Err((
			StatusCode::BAD_REQUEST,
			Json(json!({ "code": "VALIDATION_ERROR", "message": "Name, username, and password are required" })),
		));
	}

	let password_hash = password_hash::hash(&payload.password).map_err(|_| {
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(json!({ "code": "CRYPTO_ERROR", "message": "Password hashing failed" })),
		)
	})?;

	let slug = slugify(name);
	let active = payload.active.unwrap_or(true);

	let mut tx = state.model.db.begin().await.map_err(|_| {
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(json!({ "code": "DATABASE_ERROR", "message": "Transaction error" })),
		)
	})?;

	let profile = sqlx::query_file!(
		"queries/project_owners/insert_profile.sql",
		username,
		password_hash
	)
	.fetch_one(&mut *tx)
	.await
	.map_err(|e| {
		if let sqlx::Error::Database(ref db_err) = e {
			if db_err.is_unique_violation() {
				return (
					StatusCode::CONFLICT,
					Json(json!({ "code": "USERNAME_TAKEN", "message": "Username is already in use" })),
				);
			}
		}
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(json!({ "code": "DATABASE_ERROR", "message": "Failed to create profile" })),
		)
	})?;

	let owner = sqlx::query_file!(
		"queries/project_owners/insert_owner.sql",
		profile.id,
		name,
		slug,
		payload.bio.as_deref().map(|s| s.trim()),
		payload.email.as_deref().map(|s| s.trim()),
		payload.phone.as_deref().map(|s| s.trim()),
		payload.website.as_deref().map(|s| s.trim()),
		active,
	)
	.fetch_one(&mut *tx)
	.await
	.map_err(|_| {
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(json!({ "code": "DATABASE_ERROR", "message": "Failed to create project owner" })),
		)
	})?;

	tx.commit().await.map_err(|_| {
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(json!({ "code": "DATABASE_ERROR", "message": "Failed to commit transaction" })),
		)
	})?;

	Ok((
		StatusCode::CREATED,
		Json(json!({
			"id": owner.id,
			"name": name,
			"slug": slug,
			"username": username,
			"active": active,
		})),
	))
}

async fn update_project_owner(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
	Json(payload): Json<ProjectOwnerUpdatePayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
	let name = payload.name.trim();
	if name.is_empty() {
		return Err((
			StatusCode::BAD_REQUEST,
			Json(json!({ "code": "VALIDATION_ERROR", "message": "Name is required" })),
		));
	}

	let active = payload.active.unwrap_or(true);
	let slug = slugify(name);

	sqlx::query_file!(
		"queries/project_owners/update_owner.sql",
		name,
		payload.bio.as_deref().map(|s| s.trim()),
		payload.email.as_deref().map(|s| s.trim()),
		payload.phone.as_deref().map(|s| s.trim()),
		payload.website.as_deref().map(|s| s.trim()),
		active,
		slug,
		id,
	)
	.execute(&state.model.db)
	.await
	.map_err(|e| {
		tracing::error!(error = ?e, "failed to update owner");
		(
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(json!({ "code": "DATABASE_ERROR", "message": "Failed to update project owner" })),
		)
	})?;

	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::OwnerUpdated(id))
		.await;

	Ok(Json(json!({ "success": true, "message": "Owner updated successfully" })))
}

async fn toggle_active_project_owner(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
	sqlx::query_file!("queries/project_owners/toggle_active.sql", id)
		.execute(&state.model.db)
		.await
		.map_err(|e| {
			tracing::error!(error = ?e, "failed to toggle owner active");
			(
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(json!({ "code": "DATABASE_ERROR", "message": "Failed to toggle active status" })),
			)
		})?;

	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::OwnerUpdated(id))
		.await;

	Ok(Json(json!({ "success": true, "id": id })))
}

async fn add_project_owner_image(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
	mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
	while let Ok(Some(field)) = multipart.next_field().await {
		if field.name() != Some("image") {
			continue;
		}

		let bytes = field.bytes().await.map_err(|_| {
			(
				StatusCode::BAD_REQUEST,
				Json(json!({ "code": "READ_ERROR", "message": "Failed to read file buffer" })),
			)
		})?;

		let (image_data, thumb_data) = tokio::task::spawn_blocking(move || {
			let reader = ImageReader::new(std::io::Cursor::new(bytes))
				.with_guessed_format()
				.map_err(|_| "Unrecognized image format")?;

			let img = reader.decode().map_err(|_| "Failed to decode image")?;

			let webp = Encoder::from_image(&img)
				.map_err(|e| e.to_string())?
				.encode(80.0);

			let thumb_img = img.thumbnail(300, 300);
			let thumb_webp = Encoder::from_image(&thumb_img)
				.map_err(|e| e.to_string())?
				.encode(80.0);

			Ok::<_, String>((webp.to_vec(), thumb_webp.to_vec()))
		})
		.await
		.map_err(|_| {
			(
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(json!({ "code": "INTERNAL_ERROR", "message": "Image processing thread failed" })),
			)
		})?
		.map_err(|msg| {
			(
				StatusCode::BAD_REQUEST,
				Json(json!({ "code": "INVALID_IMAGE", "message": msg })),
			)
		})?;

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
			.map_err(|e| {
				tracing::error!(error = ?e, "S3 upload failed");
				(
					StatusCode::INTERNAL_SERVER_ERROR,
					Json(json!({ "code": "STORAGE_ERROR", "message": "Failed to upload to S3" })),
				)
			})?;

		state
			.s3_client
			.put_object()
			.bucket(&CONFIG.s3_bucket)
			.key(&original_img_key)
			.body(ByteStream::from(image_data))
			.send()
			.await
			.map_err(|e| {
				tracing::error!(error = ?e, "S3 upload failed");
				(
					StatusCode::INTERNAL_SERVER_ERROR,
					Json(json!({ "code": "STORAGE_ERROR", "message": "Failed to upload to S3" })),
				)
			})?;

		sqlx::query_file!(
			"queries/project_owners/update_owner_image.sql",
			original_img_key,
			thumb_key,
			id
		)
		.execute(&state.model.db)
		.await
		.map_err(|_| {
			(
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(json!({ "code": "DATABASE_ERROR", "message": "Failed to update owner image records" })),
			)
		})?;

		state
			.page_cache
			.invalidate(crate::cache::InvalidationEvent::OwnerUpdated(id))
			.await;

		return Ok(Json(json!({
			"success": true,
			"profile_image_url": CONFIG.asset_url(&original_img_key),
			"profile_thumb_url": CONFIG.asset_url(&thumb_key),
		})));
	}

	Err((
		StatusCode::BAD_REQUEST,
		Json(json!({ "code": "MISSING_FIELD", "message": "No 'image' field provided in multipart form" })),
	))
}

async fn remove_project_owner_image(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
	let owner = sqlx::query_file!("queries/project_owners/get_owner_image_keys.sql", id)
		.fetch_one(&state.model.db)
		.await
		.map_err(|_| {
			(
				StatusCode::NOT_FOUND,
				Json(json!({ "code": "NOT_FOUND", "message": "Owner not found" })),
			)
		})?;

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
		.map_err(|_| {
			(
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(json!({ "code": "DATABASE_ERROR", "message": "Failed to delete owner image keys" })),
			)
		})?;

	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::OwnerUpdated(id))
		.await;

	Ok(Json(json!({ "success": true, "message": "Avatar removed" })))
}

async fn delete_project_owner(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
	sqlx::query_file!("queries/project_owners/delete_owner.sql", id)
		.execute(&state.model.db)
		.await
		.map_err(|_| {
			(
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(json!({ "code": "DATABASE_ERROR", "message": "Failed to delete project owner" })),
			)
		})?;

	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::OwnerUpdated(id))
		.await;

	Ok(StatusCode::NO_CONTENT)
}

async fn project_owner_projects(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
) -> Result<Json<Vec<OwnerProjectSummary>>, (StatusCode, Json<serde_json::Value>)> {
	struct ProjectDbRow {
		id: i32,
		name: String,
		slug: String,
		project_type_name: String,
		city: String,
		area: String,
		formatted_address: String,
		configurations_count: i64,
	}

	let project_rows = sqlx::query_file_as!(
		ProjectDbRow,
		"queries/project_owners/get_owner_projects_all.sql",
		id
	)
	.fetch_all(&state.model.db)
	.await
	.unwrap_or_default();

	Ok(Json(
		project_rows
			.into_iter()
			.map(|p| OwnerProjectSummary {
				id: p.id,
				name: p.name,
				slug: p.slug,
				project_type_name: p.project_type_name,
				city: p.city,
				area: p.area,
				formatted_address: p.formatted_address,
				configurations_count: p.configurations_count,
			})
			.collect(),
	))
}

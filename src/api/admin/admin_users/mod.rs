use axum::{
	Json, Router,
	extract::{Path, Query, State},
	http::StatusCode,
	routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
	AppState,
	api::admin::admin_users::error::AdminUserError,
	crypto::password_hash,
	session_store::AuthUser,
};

pub mod error;

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/", get(list_admin_users).post(create_admin_user))
		.route(
			"/{id}",
			get(get_admin_user)
				.put(update_admin_user)
				.delete(delete_admin_user),
		)
}

#[derive(Deserialize, ts_rs::TS, utoipa::ToSchema, utoipa::IntoParams)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct AdminUserListQuery {
	#[serde(default)]
	pub q: Option<String>,
}

#[derive(Serialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, export_to = "admin/")]
pub struct AdminUserItem {
	pub id: i32,
	pub username: String,
	pub created_at: String,
	pub is_current_user: bool,
}

#[derive(Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, export_to = "admin/")]
pub struct CreateAdminUserPayload {
	pub username: String,
	pub password: String,
	pub confirm_password: String,
}

#[derive(Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct UpdateAdminUserPayload {
	pub username: String,
	pub password: Option<String>,
	pub confirm_password: Option<String>,
}

struct DbAdminRow {
	id: i32,
	username: String,
	created_at: time::OffsetDateTime,
}

async fn list_admin_users(
	State(state): State<AppState>,
	auth_user: AuthUser,
	Query(query): Query<AdminUserListQuery>,
) -> Result<Json<Vec<AdminUserItem>>, AdminUserError> {
	let db = &state.model.db;

	let db_rows = if let Some(q) = query.q.as_deref().filter(|s| !s.trim().is_empty()) {
		let pattern = format!("%{}%", q.trim());
		sqlx::query_file_as!(
			DbAdminRow,
			"queries/admin_users/search_admin_users.sql",
			pattern
		)
		.fetch_all(db)
		.await?
	} else {
		sqlx::query_file_as!(DbAdminRow, "queries/admin_users/list_admin_users.sql")
			.fetch_all(db)
			.await?
	};

	let date_fmt = time::macros::format_description!("[month repr:short] [day], [year]");
	let users = db_rows
		.into_iter()
		.map(|r| {
			let created_at_str = r
				.created_at
				.format(&date_fmt)
				.unwrap_or_else(|_| "Recently".to_string());
			AdminUserItem {
				id: r.id,
				username: r.username,
				created_at: created_at_str,
				is_current_user: r.id == auth_user.user_id,
			}
		})
		.collect();

	Ok(Json(users))
}

async fn get_admin_user(
	State(state): State<AppState>,
	auth_user: AuthUser,
	Path(id): Path<i32>,
) -> Result<Json<AdminUserItem>, AdminUserError> {
	let user = sqlx::query_file_as!(
		DbAdminRow,
		"queries/admin_users/get_admin_user_by_id.sql",
		id
	)
	.fetch_one(&state.model.db)
	.await
	.map_err(|e| match e {
		sqlx::Error::RowNotFound => AdminUserError::NotFound,
		_ => AdminUserError::SqlError(e),
	})?;

	let date_fmt = time::macros::format_description!("[month repr:short] [day], [year]");
	let created_at_str = user
		.created_at
		.format(&date_fmt)
		.unwrap_or_else(|_| "Recently".to_string());

	Ok(Json(AdminUserItem {
		id: user.id,
		username: user.username,
		created_at: created_at_str,
		is_current_user: user.id == auth_user.user_id,
	}))
}

async fn create_admin_user(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Json(payload): Json<CreateAdminUserPayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), AdminUserError> {
	let username = payload.username.trim();

	if username.len() < 3 {
		return Err(AdminUserError::BadRequest(
			"Username must be at least 3 characters".to_string(),
		));
	}

	if payload.password.len() < 6 {
		return Err(AdminUserError::BadRequest(
			"Password must be at least 6 characters".to_string(),
		));
	}

	if payload.password != payload.confirm_password {
		return Err(AdminUserError::BadRequest(
			"Passwords do not match".to_string(),
		));
	}

	let password_hash =
		password_hash::hash(&payload.password).map_err(|_| AdminUserError::HashingError)?;

	let mut tx = state.model.db.begin().await?;

	let profile_row = match sqlx::query_file!(
		"queries/admin_users/insert_profile.sql",
		username,
		password_hash,
	)
	.fetch_one(&mut *tx)
	.await
	{
		Ok(row) => row,
		Err(e) => {
			if let sqlx::Error::Database(db_err) = &e {
				if db_err.is_unique_violation() {
					return Err(AdminUserError::BadRequest("Username is already taken".to_string()));
				}
			}
			return Err(AdminUserError::SqlError(e));
		}
	};

	sqlx::query_file!("queries/admin_users/insert_admin_user.sql", profile_row.id)
		.execute(&mut *tx)
		.await?;

	tx.commit().await?;

	tracing::info!(admin_id = profile_row.id, username = %username, "created new admin user");
	Ok((
		StatusCode::CREATED,
		Json(json!({ "id": profile_row.id, "username": username })),
	))
}

async fn update_admin_user(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
	Json(payload): Json<UpdateAdminUserPayload>,
) -> Result<Json<serde_json::Value>, AdminUserError> {
	let username = payload.username.trim();
	if username.len() < 3 {
		return Err(AdminUserError::BadRequest(
			"Username must be at least 3 characters".to_string(),
		));
	}

	// Verify target admin exists
	let _ = sqlx::query_file_as!(
		DbAdminRow,
		"queries/admin_users/get_admin_user_by_id.sql",
		id
	)
	.fetch_one(&state.model.db)
	.await
	.map_err(|e| match e {
		sqlx::Error::RowNotFound => AdminUserError::NotFound,
		_ => AdminUserError::SqlError(e),
	})?;

	let new_password = payload.password.as_deref().unwrap_or("").trim();

	if !new_password.is_empty() {
		if new_password.len() < 6 {
			return Err(AdminUserError::BadRequest(
				"Password must be at least 6 characters".to_string(),
			));
		}
		let confirm = payload.confirm_password.as_deref().unwrap_or("").trim();
		if new_password != confirm {
			return Err(AdminUserError::BadRequest(
				"Passwords do not match".to_string(),
			));
		}

		let password_hash =
			password_hash::hash(new_password).map_err(|_| AdminUserError::HashingError)?;

		sqlx::query_file!(
			"queries/admin_users/update_admin_user.sql",
			username,
			password_hash,
			id,
		)
		.execute(&state.model.db)
		.await
		.map_err(|e| {
			if let sqlx::Error::Database(db_err) = &e {
				if db_err.is_unique_violation() {
					return AdminUserError::BadRequest("Username is already taken".to_string());
				}
			}
			AdminUserError::SqlError(e)
		})?;
	} else {
		sqlx::query_file!(
			"queries/admin_users/update_admin_username.sql",
			username,
			id,
		)
		.execute(&state.model.db)
		.await
		.map_err(|e| {
			if let sqlx::Error::Database(db_err) = &e {
				if db_err.is_unique_violation() {
					return AdminUserError::BadRequest("Username is already taken".to_string());
				}
			}
			AdminUserError::SqlError(e)
		})?;
	}

	tracing::info!(admin_id = id, username = %username, "updated admin user");
	Ok(Json(json!({ "id": id, "username": username, "message": "Admin user updated" })))
}

async fn delete_admin_user(
	State(state): State<AppState>,
	auth_user: AuthUser,
	Path(id): Path<i32>,
) -> Result<StatusCode, AdminUserError> {
	if auth_user.user_id == id {
		return Err(AdminUserError::Forbidden(
			"You cannot delete your own account".to_string(),
		));
	}

	let count_row: i64 = sqlx::query_file_scalar!("queries/admin_users/count_admin_users.sql")
		.fetch_one(&state.model.db)
		.await?
		.unwrap_or(0);

	if count_row <= 1 {
		return Err(AdminUserError::Forbidden(
			"Cannot delete the last admin user".to_string(),
		));
	}

	let _ = sqlx::query_file_as!(
		DbAdminRow,
		"queries/admin_users/get_admin_user_by_id.sql",
		id
	)
	.fetch_one(&state.model.db)
	.await
	.map_err(|e| match e {
		sqlx::Error::RowNotFound => AdminUserError::NotFound,
		_ => AdminUserError::SqlError(e),
	})?;

	let mut tx = state.model.db.begin().await?;

	sqlx::query_file!("queries/admin_users/delete_admin_user.sql", id)
		.execute(&mut *tx)
		.await?;

	sqlx::query_file!("queries/admin_users/delete_admin_profile.sql", id)
		.execute(&mut *tx)
		.await?;

	tx.commit().await?;

	tracing::info!(admin_id = id, "deleted admin user");
	Ok(StatusCode::NO_CONTENT)
}

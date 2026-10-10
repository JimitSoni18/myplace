use askama::Template;
use axum::{
	Extension, Form, Router,
	extract::{Path, Query, State},
	http::StatusCode,
	response::{Html, IntoResponse, Redirect},
	routing::get,
};
use serde::Deserialize;

use crate::{
	AppState,
	api::admin::admin_users::error::AdminUserError,
	crypto::password_hash,
	session_store::AuthUser,
	templates::admin::{
		AdminPage, AdminUserEditItem, AdminUserEditValues, AdminUserFormTemplate, AdminUserItem,
		AdminUserListTemplate,
	},
};

pub mod error;

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/", get(admin_user_list))
		.route("/new", get(admin_user_create_form).post(create_admin_user))
		.route(
			"/{id}",
			get(admin_user_edit_form)
				.put(update_admin_user_api)
				.delete(delete_admin_user_api),
		)
		.route(
			"/{id}/edit",
			get(admin_user_edit_form).post(update_admin_user_form),
		)
		.route("/{id}/delete", axum::routing::post(delete_admin_user_form))
}

#[derive(Deserialize)]
pub struct AdminUserListQuery {
	#[serde(default, deserialize_with = "crate::utils::form::empty_string_as_none")]
	pub q: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateAdminUserPayload {
	pub username: String,
	pub password: String,
	pub confirm_password: String,
}

#[derive(Deserialize)]
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

async fn admin_user_list(
	State(state): State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
	Query(query): Query<AdminUserListQuery>,
) -> Result<Html<String>, AdminUserError> {
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

	let template = AdminUserListTemplate {
		admin_name: &auth_user.username,
		page: AdminPage::AdminUsers,
		users,
		q: query.q.as_deref(),
	};

	Ok(Html(template.render()?))
}

async fn admin_user_create_form(
	Extension(auth_user): Extension<AuthUser>,
) -> Result<Html<String>, AdminUserError> {
	let template = AdminUserFormTemplate {
		admin_name: &auth_user.username,
		page: AdminPage::AdminUsers,
		edit_values: None,
		is_edit: false,
	};
	Ok(Html(template.render()?))
}

async fn create_admin_user(
	State(state): State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
	Form(payload): Form<CreateAdminUserPayload>,
) -> Result<impl IntoResponse, AdminUserError> {
	let username = payload.username.trim();

	if username.len() < 3 {
		let template = AdminUserFormTemplate {
			admin_name: &auth_user.username,
			page: AdminPage::AdminUsers,
			edit_values: Some(AdminUserEditValues {
				default_values: AdminUserEditItem {
					id: 0,
					username: username.to_string(),
				},
				error: Some("Username must be at least 3 characters"),
			}),
			is_edit: false,
		};
		return Ok(Html(template.render()?).into_response());
	}

	if payload.password.len() < 6 {
		let template = AdminUserFormTemplate {
			admin_name: &auth_user.username,
			page: AdminPage::AdminUsers,
			edit_values: Some(AdminUserEditValues {
				default_values: AdminUserEditItem {
					id: 0,
					username: username.to_string(),
				},
				error: Some("Password must be at least 6 characters"),
			}),
			is_edit: false,
		};
		return Ok(Html(template.render()?).into_response());
	}

	if payload.password != payload.confirm_password {
		let template = AdminUserFormTemplate {
			admin_name: &auth_user.username,
			page: AdminPage::AdminUsers,
			edit_values: Some(AdminUserEditValues {
				default_values: AdminUserEditItem {
					id: 0,
					username: username.to_string(),
				},
				error: Some("Passwords do not match"),
			}),
			is_edit: false,
		};
		return Ok(Html(template.render()?).into_response());
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
					let template = AdminUserFormTemplate {
						admin_name: &auth_user.username,
						page: AdminPage::AdminUsers,
						edit_values: Some(AdminUserEditValues {
							default_values: AdminUserEditItem {
								id: 0,
								username: username.to_string(),
							},
							error: Some("Username is already taken"),
						}),
						is_edit: false,
					};
					return Ok(Html(template.render()?).into_response());
				}
			}
			return Err(AdminUserError::SqlError(e));
		}
	};

	sqlx::query_file!("queries/admin_users/insert_admin_user.sql", profile_row.id,)
		.execute(&mut *tx)
		.await?;

	tx.commit().await?;

	tracing::info!(admin_id = profile_row.id, username = %username, "created new admin user");
	Ok(Redirect::to("/admin/users").into_response())
}

async fn admin_user_edit_form(
	State(state): State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
	Path(id): Path<i32>,
) -> Result<Html<String>, AdminUserError> {
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

	let template = AdminUserFormTemplate {
		admin_name: &auth_user.username,
		page: AdminPage::AdminUsers,
		edit_values: Some(AdminUserEditValues {
			default_values: AdminUserEditItem {
				id: user.id,
				username: user.username,
			},
			error: None,
		}),
		is_edit: true,
	};

	Ok(Html(template.render()?))
}

async fn update_admin_user_form(
	State(state): State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
	Path(id): Path<i32>,
	Form(payload): Form<UpdateAdminUserPayload>,
) -> Result<impl IntoResponse, AdminUserError> {
	let username_for_rerender = payload.username.clone();
	match perform_update_admin_user(&state, id, payload).await {
		Ok(()) => Ok(Redirect::to("/admin/users").into_response()),
		Err(AdminUserError::BadRequest(msg)) => {
			let template = AdminUserFormTemplate {
				admin_name: &auth_user.username,
				page: AdminPage::AdminUsers,
				edit_values: Some(AdminUserEditValues {
					default_values: AdminUserEditItem {
						id,
						username: username_for_rerender,
					},
					error: Some(&msg),
				}),
				is_edit: true,
			};
			Ok(Html(template.render()?).into_response())
		}
		Err(e) => Err(e),
	}
}

async fn update_admin_user_api(
	State(state): State<AppState>,
	Path(id): Path<i32>,
	Form(payload): Form<UpdateAdminUserPayload>,
) -> Result<StatusCode, AdminUserError> {
	perform_update_admin_user(&state, id, payload).await?;
	Ok(StatusCode::OK)
}

async fn perform_update_admin_user(
	state: &AppState,
	id: i32,
	payload: UpdateAdminUserPayload,
) -> Result<(), AdminUserError> {
	let username = payload.username.trim();
	if username.len() < 3 {
		return Err(AdminUserError::BadRequest(
			"Username must be at least 3 characters".to_string(),
		));
	}

	// Verify admin exists
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
	Ok(())
}

async fn delete_admin_user_form(
	State(state): State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
	Path(id): Path<i32>,
) -> Result<Redirect, AdminUserError> {
	perform_delete_admin_user(&state, auth_user.user_id, id).await?;
	Ok(Redirect::to("/admin/users"))
}

async fn delete_admin_user_api(
	State(state): State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
	Path(id): Path<i32>,
) -> Result<StatusCode, AdminUserError> {
	perform_delete_admin_user(&state, auth_user.user_id, id).await?;
	Ok(StatusCode::NO_CONTENT)
}

async fn perform_delete_admin_user(
	state: &AppState,
	current_admin_id: i32,
	target_id: i32,
) -> Result<(), AdminUserError> {
	if current_admin_id == target_id {
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

	// Verify target admin exists
	let _ = sqlx::query_file_as!(
		DbAdminRow,
		"queries/admin_users/get_admin_user_by_id.sql",
		target_id
	)
	.fetch_one(&state.model.db)
	.await
	.map_err(|e| match e {
		sqlx::Error::RowNotFound => AdminUserError::NotFound,
		_ => AdminUserError::SqlError(e),
	})?;

	let mut tx = state.model.db.begin().await?;

	sqlx::query_file!("queries/admin_users/delete_admin_user.sql", target_id,)
		.execute(&mut *tx)
		.await?;

	sqlx::query_file!("queries/admin_users/delete_admin_profile.sql", target_id,)
		.execute(&mut *tx)
		.await?;

	tx.commit().await?;

	tracing::info!(admin_id = target_id, "deleted admin user");
	Ok(())
}

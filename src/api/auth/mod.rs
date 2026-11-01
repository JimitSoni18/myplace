use axum::{
	Json, Router,
	extract::State,
	http::{StatusCode, header},
	response::IntoResponse,
	routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub mod errors;
use errors::LoginError;

use crate::{
	AppState,
	constants::misc::SIX_HOURS_SECONDS,
	crypto::{password_hash, sign_cookie},
	response_types::SetAuthCookie,
	session_store::{AuthUser, SessionId, SessionStoreTrait as _, UserRole},
};

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/login", post(admin_login))
		.route("/admin/login", post(admin_login))
		.route("/owner/login", post(owner_login))
		.route("/logout", post(logout))
		.route("/me", get(get_current_user))
}

#[derive(Deserialize, ts_rs::TS)]
#[ts(export, export_to = "auth/")]
pub struct LoginPayload {
	pub username: String,
	pub password: String,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "auth/", optional_fields = nullable)]
pub struct UserInfo {
	pub id: i32,
	pub username: String,
	pub role: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub owner_id: Option<i32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub owner_name: Option<String>,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "auth/")]
pub struct LoginResponse {
	pub token: String,
	pub user: UserInfo,
}

struct AdminUserRow {
	id: i32,
	username: String,
	password: String,
	#[allow(dead_code)]
	profile_id: i32,
}

async fn admin_login(
	State(state): State<AppState>,
	Json(payload): Json<LoginPayload>,
) -> Result<(SetAuthCookie, Json<LoginResponse>), LoginError> {
	let username = payload.username.trim();
	if username.is_empty() || payload.password.is_empty() {
		return Err(LoginError::BadRequest);
	}

	let db = &state.model.db;
	let user = sqlx::query_file_as!(
		AdminUserRow,
		"queries/auth/get_login_user.sql",
		username
	)
	.fetch_one(db)
	.await
	.map_err(|why| {
		if matches!(why, sqlx::Error::RowNotFound) {
			LoginError::InvalidCredentials
		} else {
			tracing::error!(?why, "database error during admin login");
			LoginError::InternalError
		}
	})?;

	password_hash::verify(&payload.password, &user.password)
		.or(Err(LoginError::InvalidCredentials))?;

	let auth_user = AuthUser::new(user.id, user.username.clone(), UserRole::Admin);
	let session_id = state.session_store.create_session(auth_user).await;
	let signature = sign_cookie::get_signature(&session_id);
	let signed_token = format!("{}.{signature}", session_id.0);

	tracing::info!(user_id = user.id, username = %user.username, "admin login successful");

	Ok((
		SetAuthCookie::new_with_name("admin_session_id", &signed_token, SIX_HOURS_SECONDS),
		Json(LoginResponse {
			token: signed_token,
			user: UserInfo {
				id: user.id,
				username: user.username,
				role: "admin".to_string(),
				owner_id: None,
				owner_name: None,
			},
		}),
	))
}

async fn owner_login(
	State(state): State<AppState>,
	Json(payload): Json<LoginPayload>,
) -> Result<(SetAuthCookie, Json<LoginResponse>), LoginError> {
	let username = payload.username.trim();
	if username.is_empty() || payload.password.is_empty() {
		return Err(LoginError::BadRequest);
	}

	let db = &state.model.db;

	#[derive(sqlx::FromRow)]
	struct OwnerUserRow {
		id: i32,
		username: String,
		password: String,
		active: bool,
		owner_id: i32,
		owner_name: String,
	}

	let user = sqlx::query_as::<_, OwnerUserRow>(
		r#"
		SELECT p.id, p.username, p.password, po.active, po.id as owner_id, po.name as owner_name
		FROM profiles p
		JOIN project_owners po ON po.profile_id = p.id
		WHERE p.username = $1 AND po.deleted_at IS NULL
		"#,
	)
	.bind(username)
	.fetch_one(db)
	.await
	.map_err(|why| {
		if matches!(why, sqlx::Error::RowNotFound) {
			LoginError::InvalidCredentials
		} else {
			tracing::error!(?why, "database error during owner login");
			LoginError::InternalError
		}
	})?;

	if !user.active {
		tracing::warn!(user_id = user.id, "inactive owner login rejected");
		return Err(LoginError::AccountInactive);
	}

	password_hash::verify(&payload.password, &user.password)
		.or(Err(LoginError::InvalidCredentials))?;

	let auth_user = AuthUser::new(user.id, user.username.clone(), UserRole::ProjectOwner);
	let session_id = state.session_store.create_session(auth_user).await;
	let signature = sign_cookie::get_signature(&session_id);
	let signed_token = format!("{}.{signature}", session_id.0);

	tracing::info!(user_id = user.id, owner_id = user.owner_id, "owner login successful");

	Ok((
		SetAuthCookie::new_with_name("owner_session_id", &signed_token, SIX_HOURS_SECONDS),
		Json(LoginResponse {
			token: signed_token,
			user: UserInfo {
				id: user.id,
				username: user.username,
				role: "owner".to_string(),
				owner_id: Some(user.owner_id),
				owner_name: Some(user.owner_name),
			},
		}),
	))
}

async fn logout(
	State(state): State<AppState>,
	headers: axum::http::HeaderMap,
) -> impl IntoResponse {
	// Extract session id if present to delete from store
	let session_id = headers
		.get(header::AUTHORIZATION)
		.and_then(|h| h.to_str().ok())
		.and_then(|h| h.strip_prefix("Bearer "))
		.map(|s| s.trim())
		.or_else(|| {
			headers
				.get(header::COOKIE)
				.and_then(|h| h.to_str().ok())
				.and_then(|cookies| {
					cookies.split("; ").find_map(|c| {
						let (name, value) = c.trim().split_once('=')?;
						if name == "admin_session_id" || name == "owner_session_id" || name == "session_id" {
							Some(value)
						} else {
							None
						}
					})
				})
		})
		.and_then(|raw| {
			let (id_str, sign) = raw.split_once('.')?;
			let id = uuid::Uuid::parse_str(id_str).ok()?;
			if sign_cookie::verify_cookie(&id, sign) {
				Some(SessionId(id))
			} else {
				None
			}
		});

	if let Some(sid) = session_id {
		state.session_store.delete_session(&sid).await;
	}

	(
		SetAuthCookie::clear_all(),
		Json(json!({ "success": true, "message": "Logged out" })),
	)
}

async fn get_current_user(
	State(state): State<AppState>,
	headers: axum::http::HeaderMap,
) -> Result<Json<UserInfo>, (StatusCode, Json<serde_json::Value>)> {
	let unauth_err = (
		StatusCode::UNAUTHORIZED,
		Json(json!({ "code": "UNAUTHORIZED", "message": "Not authenticated" })),
	);

	let session_id = headers
		.get(header::AUTHORIZATION)
		.and_then(|h| h.to_str().ok())
		.and_then(|h| h.strip_prefix("Bearer "))
		.map(|s| s.trim())
		.or_else(|| {
			headers
				.get(header::COOKIE)
				.and_then(|h| h.to_str().ok())
				.and_then(|cookies| {
					cookies.split("; ").find_map(|c| {
						let (name, value) = c.trim().split_once('=')?;
						if name == "admin_session_id" || name == "owner_session_id" || name == "session_id" {
							Some(value)
						} else {
							None
						}
					})
				})
		})
		.and_then(|raw| {
			let (id_str, sign) = raw.split_once('.')?;
			let id = uuid::Uuid::parse_str(id_str).ok()?;
			if sign_cookie::verify_cookie(&id, sign) {
				Some(SessionId(id))
			} else {
				None
			}
		})
		.ok_or(unauth_err.clone())?;

	let auth_user = state
		.session_store
		.get_session(&session_id)
		.await
		.ok_or(unauth_err.clone())?;

	let (role_str, owner_id, owner_name) = match auth_user.role {
		UserRole::Admin => ("admin".to_string(), None, None),
		UserRole::ProjectOwner => {
			#[derive(sqlx::FromRow)]
			struct OwnerInfoRow {
				id: i32,
				name: String,
			}

			let row = sqlx::query_as::<_, OwnerInfoRow>(
				"SELECT id, name FROM project_owners WHERE profile_id = $1 AND deleted_at IS NULL",
			)
			.bind(auth_user.user_id)
			.fetch_optional(&state.model.db)
			.await
			.ok()
			.flatten();

			if let Some(r) = row {
				("owner".to_string(), Some(r.id), Some(r.name))
			} else {
				("owner".to_string(), None, None)
			}
		}
	};

	Ok(Json(UserInfo {
		id: auth_user.user_id,
		username: auth_user.username,
		role: role_str,
		owner_id,
		owner_name,
	}))
}

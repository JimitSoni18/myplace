use askama::Template;
use axum::{
	Form, Router,
	extract::State,
	response::{Html, Redirect},
	routing::MethodRouter,
};
use serde::Deserialize;

pub mod errors;
use errors::LoginError;

use crate::{
	AppState,
	constants::misc::SIX_HOURS_SECONDS,
	crypto::{password_hash, sign_cookie::get_signature},
	pages::LOGIN_HTML,
	response_types::SetAuthCookie,
	session_store::{AuthUser, SessionStoreTrait as _, UserRole},
	templates::auth::OwnerLoginTemplate,
};

pub fn admin_auth_router() -> Router<AppState> {
	let admin_login_router = MethodRouter::new()
		.get(async || Html(&LOGIN_HTML as &str))
		.post(login);

	Router::new()
		.route("/admin-login", admin_login_router.clone())
		.route("/login", admin_login_router)
}

pub fn owner_auth_router() -> Router<AppState> {
	let owner_login_router = MethodRouter::new()
		.get(async || {
			let html = OwnerLoginTemplate::default().render().unwrap_or_default();
			Html(html)
		})
		.post(owner_login);

	Router::new()
		.route("/owner-login", owner_login_router.clone())
		.route("/login", owner_login_router)
}

pub fn router() -> Router<AppState> {
	let admin_login_router = MethodRouter::new()
		.get(async || Html(&LOGIN_HTML as &str))
		.post(login);

	let owner_login_router = MethodRouter::new()
		.get(async || {
			let html = OwnerLoginTemplate::default().render().unwrap_or_default();
			Html(html)
		})
		.post(owner_login);

	Router::new()
		.route("/admin-login", admin_login_router.clone())
		.route("/owner-login", owner_login_router)
		.route("/login", admin_login_router)
}

#[derive(Deserialize)]
struct LoginPayload {
	username: String,
	password: String,
}

/// Fetches the profile+admin link for a given username.
/// Only returns a row if the profile exists AND is in admin_users.
struct AdminUserRow {
	id: i32,
	username: String,
	password: String,
	#[allow(dead_code)]
	profile_id: i32,
}

async fn login(
	state: State<AppState>,
	payload: Form<LoginPayload>,
) -> Result<(SetAuthCookie, Redirect), LoginError> {
	if payload.username.is_empty() || payload.password.is_empty() {
		return Err(LoginError::BadRequest);
	}

	let db = &state.model.db;

	let user = sqlx::query_file_as!(
		AdminUserRow,
		"queries/auth/get_login_user.sql",
		payload.username.trim()
	)
	.fetch_one(db)
	.await
	.map_err(|why| {
		if matches!(why, sqlx::Error::RowNotFound) {
			return LoginError::InvalidCredentials;
		}
		tracing::error!(?why, "database error during admin login");
		LoginError::InternalError
	})?;

	password_hash::verify(&payload.password, &user.password)
		.or(Err(LoginError::InvalidCredentials))?;

	// Create session with Admin role.
	let auth_user = AuthUser::new(user.id, user.username, UserRole::Admin);
	let session_id = state.session_store.create_session(auth_user).await;
	let cookie_signature = get_signature(&session_id);
	let signed_session_id = format!("{}.{cookie_signature}", session_id.0);

	tracing::info!(user_id = user.id, "admin login successful");

	Ok((
		SetAuthCookie::new(&signed_session_id, SIX_HOURS_SECONDS),
		Redirect::to("/admin"),
	))
}

async fn owner_login(
	state: State<AppState>,
	payload: Form<LoginPayload>,
) -> Result<(SetAuthCookie, Redirect), LoginError> {
	if payload.username.is_empty() || payload.password.is_empty() {
		return Err(LoginError::BadRequest);
	}

	let db = &state.model.db;

	struct OwnerUserRow {
		id: i32,
		username: String,
		password: String,
		active: bool,
	}

	let user = sqlx::query_file_as!(
		OwnerUserRow,
		"queries/auth/get_owner_login_user.sql",
		payload.username.trim()
	)
	.fetch_one(db)
	.await
	.map_err(|why| {
		if matches!(why, sqlx::Error::RowNotFound) {
			return LoginError::InvalidCredentials;
		}
		tracing::error!(?why, "database error during owner login");
		LoginError::InternalError
	})?;

	if !user.active {
		tracing::warn!(user_id = user.id, "inactive owner login rejected");
		return Err(LoginError::InvalidCredentials);
	}

	password_hash::verify(&payload.password, &user.password)
		.or(Err(LoginError::InvalidCredentials))?;

	let auth_user = AuthUser::new(user.id, user.username, UserRole::ProjectOwner);
	let session_id = state.session_store.create_session(auth_user).await;
	let cookie_signature = get_signature(&session_id);
	let signed_session_id = format!("{}.{cookie_signature}", session_id.0);

	tracing::info!(user_id = user.id, "owner login successful");

	Ok((
		SetAuthCookie::new(&signed_session_id, SIX_HOURS_SECONDS),
		Redirect::to("/owner"),
	))
}

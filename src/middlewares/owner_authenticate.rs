use std::str::FromStr as _;

use axum::{
	extract::FromRequestParts,
	http::{StatusCode, header},
	response::Json,
};
use serde_json::json;

use crate::{
	AppState, crypto,
	session_store::{SessionId, SessionStoreTrait as _, UserRole},
};

#[derive(Clone, Debug, serde::Serialize)]
pub struct OwnerUser {
	pub profile_id: i32,
	pub owner_id: i32,
	pub owner_name: String,
	pub username: String,
}

fn parse_signed_session_id(raw: &str) -> Option<uuid::Uuid> {
	let (id_str, sign) = raw.split_once('.')?;
	let id = uuid::Uuid::from_str(id_str).ok()?;
	if crypto::sign_cookie::verify_cookie(&id, sign) {
		Some(id)
	} else {
		None
	}
}

impl FromRequestParts<AppState> for OwnerUser {
	type Rejection = (StatusCode, Json<serde_json::Value>);

	async fn from_request_parts(
		parts: &mut axum::http::request::Parts,
		state: &AppState,
	) -> Result<Self, Self::Rejection> {
		let unauth_err = (
			StatusCode::UNAUTHORIZED,
			Json(json!({
				"code": "UNAUTHORIZED",
				"message": "Authentication required or session expired"
			})),
		);

		let mut candidate_raw_tokens = Vec::new();

		// 1. Check Authorization: Bearer <token> header first
		if let Some(h) = parts
			.headers
			.get(header::AUTHORIZATION)
			.and_then(|h| h.to_str().ok())
		{
			if let Some(token) = h.strip_prefix("Bearer ") {
				candidate_raw_tokens.push(token.trim().to_string());
			}
		}

		// 2. Check ?token=<signed_session_id> query parameter (URL-decoded, essential for EventSource SSE)
		if let Some(q) = parts.uri.query() {
			#[derive(serde::Deserialize)]
			struct QueryParams {
				token: Option<String>,
			}
			if let Ok(qp) = serde_urlencoded::from_str::<QueryParams>(q) {
				if let Some(t) = qp.token {
					candidate_raw_tokens.push(t);
				}
			}
		}

		// 3. Check Cookies: owner_session_id takes precedence, fallback to session_id
		if let Some(cookies) = parts.headers.get(header::COOKIE).and_then(|h| h.to_str().ok()) {
			let mut owner_cookie = None;
			let mut general_cookie = None;

			for part in cookies.split(';') {
				if let Some((name, val)) = part.trim().split_once('=') {
					if name == "owner_session_id" {
						owner_cookie = Some(val.to_string());
					} else if name == "session_id" && general_cookie.is_none() {
						general_cookie = Some(val.to_string());
					}
				}
			}

			if let Some(c) = owner_cookie {
				candidate_raw_tokens.push(c);
			}
			if let Some(c) = general_cookie {
				candidate_raw_tokens.push(c);
			}
		}

		// 4. Find valid owner session
		let mut authenticated_session = None;
		for raw in candidate_raw_tokens {
			if let Some(uuid) = parse_signed_session_id(&raw) {
				let session_id = SessionId(uuid);
				if let Some(auth_user) = state.session_store.get_session(&session_id).await {
					if auth_user.role == UserRole::ProjectOwner {
						authenticated_session = Some((session_id, auth_user));
						break;
					}
				}
			}
		}

		let (session_id, auth_user) = match authenticated_session {
			Some(s) => s,
			None => {
				tracing::debug!("owner request missing or invalid credentials");
				return Err(unauth_err);
			}
		};

		// 5. Look up project owner record
		let owner_row = sqlx::query_file!(
			"queries/auth/get_owner_by_profile_id.sql",
			auth_user.user_id
		)
		.fetch_optional(&state.model.db)
		.await
		.map_err(|e| {
			tracing::error!(error = ?e, "db lookup failed during owner auth");
			(
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(json!({
					"code": "INTERNAL_ERROR",
					"message": "Database query error"
				})),
			)
		})?
		.ok_or_else(|| {
			tracing::warn!(
				profile_id = auth_user.user_id,
				"no project_owner linked to profile"
			);
			unauth_err.clone()
		})?;

		if !owner_row.active {
			tracing::warn!(
				owner_id = owner_row.id,
				"inactive owner attempted to access portal"
			);
			return Err((
				StatusCode::FORBIDDEN,
				Json(json!({
					"code": "ACCOUNT_INACTIVE",
					"message": "Owner account is disabled"
				})),
			));
		}

		let owner_user = OwnerUser {
			profile_id: auth_user.user_id,
			owner_id: owner_row.id,
			owner_name: owner_row.name,
			username: auth_user.username,
		};

		parts.extensions.insert(owner_user.clone());
		parts.extensions.insert(session_id);

		Ok(owner_user)
	}
}

use std::str::FromStr as _;

use axum::{
	extract::FromRequestParts,
	http::{StatusCode, header},
	response::Json,
};
use serde_json::json;

use crate::{
	AppState, crypto,
	session_store::{AuthUser, SessionId, SessionStoreTrait as _, UserRole},
};

fn parse_signed_session_id(raw: &str) -> Option<uuid::Uuid> {
	let (id_str, sign) = raw.split_once('.')?;
	let id = uuid::Uuid::from_str(id_str).ok()?;
	if crypto::sign_cookie::verify_cookie(&id, sign) {
		Some(id)
	} else {
		None
	}
}

impl FromRequestParts<AppState> for AuthUser {
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

		// 1. Check Authorization: Bearer <token> header first (supports cross-domain static frontends)
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

		// 3. Check Cookies: admin_session_id takes precedence, fallback to session_id
		if let Some(cookies) = parts.headers.get(header::COOKIE).and_then(|h| h.to_str().ok()) {
			let mut admin_cookie = None;
			let mut general_cookie = None;

			for part in cookies.split(';') {
				if let Some((name, val)) = part.trim().split_once('=') {
					if name == "admin_session_id" {
						admin_cookie = Some(val.to_string());
					} else if name == "session_id" && general_cookie.is_none() {
						general_cookie = Some(val.to_string());
					}
				}
			}

			if let Some(c) = admin_cookie {
				candidate_raw_tokens.push(c);
			}
			if let Some(c) = general_cookie {
				candidate_raw_tokens.push(c);
			}
		}

		// 4. Try each candidate in priority order
		for raw in candidate_raw_tokens {
			if let Some(uuid) = parse_signed_session_id(&raw) {
				let session_id = SessionId(uuid);
				if let Some(auth_user) = state.session_store.get_session(&session_id).await {
					if auth_user.role == UserRole::Admin {
						parts.extensions.insert(auth_user.clone());
						parts.extensions.insert(session_id);
						return Ok(auth_user);
					}
				}
			}
		}

		tracing::debug!("admin request missing or invalid session credentials");
		Err(unauth_err)
	}
}

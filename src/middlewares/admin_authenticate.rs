use std::str::FromStr as _;

use axum::{extract::FromRequestParts, http::StatusCode, response::Html};

use crate::{
	AppState, crypto,
	pages::NO_ACCESS_PAGE,
	session_store::{AuthUser, SessionId, SessionStoreTrait as _, UserRole},
};

impl FromRequestParts<AppState> for AuthUser {
	type Rejection = (StatusCode, Html<&'static str>);

	async fn from_request_parts(
		parts: &mut axum::http::request::Parts,
		state: &AppState,
	) -> Result<Self, Self::Rejection> {
		let error = (StatusCode::UNAUTHORIZED, NO_ACCESS_PAGE);

		// 1. Parse and verify the signed session cookie.
		let session_id = SessionId(
			parts
				.headers
				.get(axum::http::header::COOKIE)
				.and_then(|h| h.to_str().ok())
				.and_then(|cookies| {
					cookies.split("; ").find_map(|c| {
						if let ("session_id", value) = c.trim().split_once('=')? {
							let (session_id_str, sign) = value.split_once('.')?;
							let session_id = uuid::Uuid::from_str(session_id_str).ok()?;
							if crypto::sign_cookie::verify_cookie(&session_id, sign) {
								return Some(session_id);
							}
						}
						None
					})
				})
				.ok_or_else(|| {
					tracing::debug!("request missing or invalid session cookie");
					error
				})?,
		);

		// 2. Look up session — get_session returns None for expired sessions.
		let auth_user = state
			.session_store
			.get_session(&session_id)
			.await
			.ok_or_else(|| {
				tracing::warn!(?session_id, "session not found or expired");
				error
			})?;

		// 3. Enforce that only Admin users can access admin routes.
		if auth_user.role != UserRole::Admin {
			tracing::warn!(
				user_id = auth_user.user_id,
				"non-admin attempted to access admin route"
			);
			return Err(error);
		}

		// 4. Attach to request extensions so handlers can extract them cheaply.
		parts.extensions.insert(auth_user.clone());
		parts.extensions.insert(session_id);

		Ok(auth_user)
	}
}

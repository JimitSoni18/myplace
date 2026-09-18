use std::str::FromStr as _;

use axum::{extract::FromRequestParts, http::StatusCode, response::Html};

use crate::{
	AppState, crypto,
	pages::NO_ACCESS_PAGE,
	session_store::{SessionId, SessionStoreTrait as _, UserRole},
};

#[derive(Clone, Debug)]
pub struct OwnerUser {
	pub profile_id: i32,
	pub owner_id: i32,
	pub owner_name: String,
	pub username: String,
}

impl FromRequestParts<AppState> for OwnerUser {
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
					tracing::debug!("owner request missing or invalid session cookie");
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

		// 3. Enforce that only ProjectOwner users can access owner routes.
		if auth_user.role != UserRole::ProjectOwner {
			tracing::warn!(
				user_id = auth_user.user_id,
				"non-owner attempted to access owner route"
			);
			return Err(error);
		}

		// 4. Verify project owner status in DB: active must be true, not deleted.
		let owner_row = sqlx::query!(
			"SELECT id, name, active FROM project_owners WHERE profile_id = $1 AND deleted_at IS NULL",
			auth_user.user_id
		)
		.fetch_optional(&state.model.db)
		.await
		.map_err(|e| {
			tracing::error!(error = ?e, "db lookup failed during owner auth");
			error
		})?
		.ok_or_else(|| {
			tracing::warn!(profile_id = auth_user.user_id, "no project_owner linked to profile");
			error
		})?;

		if !owner_row.active {
			tracing::warn!(owner_id = owner_row.id, "inactive owner attempted to access portal");
			return Err(error);
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

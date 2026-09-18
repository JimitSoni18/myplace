use askama::Template;
use axum::{
	http::StatusCode,
	response::{Html, IntoResponse},
};

use crate::templates::auth::LoginTemplate;

pub enum LoginError {
	InvalidCredentials,
	InternalError,
	BadRequest,
}

impl IntoResponse for LoginError {
	fn into_response(self) -> axum::response::Response {
		match self {
			Self::InvalidCredentials => (
				StatusCode::UNAUTHORIZED,
				Html(
					LoginTemplate {
						incorrect_password_error: true,
					}
					.render()
					.unwrap(),
				),
			)
				.into_response(),
			Self::InternalError => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
			Self::BadRequest => StatusCode::BAD_REQUEST.into_response(),
		}
	}
}

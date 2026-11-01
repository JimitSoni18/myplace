use axum::{
	http::StatusCode,
	response::{IntoResponse, Json},
};
use serde_json::json;

#[derive(Debug)]
pub enum LoginError {
	InvalidCredentials,
	AccountInactive,
	InternalError,
	BadRequest,
}

impl IntoResponse for LoginError {
	fn into_response(self) -> axum::response::Response {
		let (status, code, message) = match self {
			Self::InvalidCredentials => (
				StatusCode::UNAUTHORIZED,
				"INVALID_CREDENTIALS",
				"Invalid username or password",
			),
			Self::AccountInactive => (
				StatusCode::FORBIDDEN,
				"ACCOUNT_INACTIVE",
				"This account has been deactivated",
			),
			Self::BadRequest => (
				StatusCode::BAD_REQUEST,
				"BAD_REQUEST",
				"Username and password are required",
			),
			Self::InternalError => (
				StatusCode::INTERNAL_SERVER_ERROR,
				"INTERNAL_ERROR",
				"An internal server error occurred",
			),
		};

		(status, Json(json!({ "code": code, "message": message }))).into_response()
	}
}

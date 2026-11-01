use axum::{
	http::StatusCode,
	response::{IntoResponse, Json},
};
use serde_json::json;

#[derive(Debug)]
pub enum AdminUserError {
	NotFound,
	BadRequest(String),
	Forbidden(String),
	SqlError(sqlx::Error),
	HashingError,
}

impl From<sqlx::Error> for AdminUserError {
	fn from(e: sqlx::Error) -> Self {
		match &e {
			sqlx::Error::RowNotFound => Self::NotFound,
			sqlx::Error::Database(db_err) => {
				if db_err.is_unique_violation() {
					Self::BadRequest("Username is already taken".to_string())
				} else {
					Self::SqlError(e)
				}
			}
			_ => Self::SqlError(e),
		}
	}
}

impl IntoResponse for AdminUserError {
	fn into_response(self) -> axum::response::Response {
		tracing::error!(error = ?self, "admin user route error");
		match self {
			Self::NotFound => (
				StatusCode::NOT_FOUND,
				Json(json!({ "code": "NOT_FOUND", "message": "Admin user not found" })),
			)
				.into_response(),
			Self::BadRequest(msg) => (
				StatusCode::BAD_REQUEST,
				Json(json!({ "code": "VALIDATION_ERROR", "message": msg })),
			)
				.into_response(),
			Self::Forbidden(msg) => (
				StatusCode::FORBIDDEN,
				Json(json!({ "code": "FORBIDDEN", "message": msg })),
			)
				.into_response(),
			Self::HashingError => (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(json!({ "code": "CRYPTO_ERROR", "message": "Password hashing error" })),
			)
				.into_response(),
			Self::SqlError(_) => (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(json!({ "code": "DATABASE_ERROR", "message": "Database query failure" })),
			)
				.into_response(),
		}
	}
}

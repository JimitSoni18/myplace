use axum::{
	http::StatusCode,
	response::{IntoResponse, Json},
};
use serde_json::json;

#[derive(Debug)]
pub enum ProjectError {
	NotFound,
	SqlError(sqlx::Error),
	Validation(String),
	Media(crate::media::MediaError),
	Internal(String),
}

impl From<sqlx::Error> for ProjectError {
	fn from(e: sqlx::Error) -> Self {
		match &e {
			sqlx::Error::RowNotFound => Self::NotFound,
			sqlx::Error::Database(db_err) => {
				if db_err.is_foreign_key_violation()
					|| db_err.is_check_violation()
					|| db_err.is_unique_violation()
					|| db_err.code().as_deref() == Some("23502")
					|| db_err.code().as_deref() == Some("22P02")
					|| db_err.code().as_deref() == Some("22003")
				{
					Self::Validation(db_err.message().to_string())
				} else {
					Self::SqlError(e)
				}
			}
			_ => Self::SqlError(e),
		}
	}
}

impl From<crate::media::MediaError> for ProjectError {
	fn from(e: crate::media::MediaError) -> Self {
		Self::Media(e)
	}
}

impl IntoResponse for ProjectError {
	fn into_response(self) -> axum::response::Response {
		tracing::error!(error = ?self, "project route error");
		match self {
			Self::NotFound => (
				StatusCode::NOT_FOUND,
				Json(json!({ "code": "NOT_FOUND", "message": "Project not found" })),
			)
				.into_response(),
			Self::Validation(msg) => (
				StatusCode::BAD_REQUEST,
				Json(json!({ "code": "VALIDATION_ERROR", "message": msg })),
			)
				.into_response(),
			Self::Media(crate::media::MediaError::ImageFormatNotSupported)
			| Self::Media(crate::media::MediaError::ImageDecodeError(_))
			| Self::Media(crate::media::MediaError::VideoProcessingError(_)) => (
				StatusCode::BAD_REQUEST,
				Json(json!({ "code": "INVALID_MEDIA", "message": "Media format not supported or invalid" })),
			)
				.into_response(),
			Self::Media(e) => (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(json!({ "code": "MEDIA_ERROR", "message": format!("{:?}", e) })),
			)
				.into_response(),
			Self::Internal(msg) => (
				StatusCode::INTERNAL_SERVER_ERROR,
				Json(json!({ "code": "INTERNAL_ERROR", "message": msg })),
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

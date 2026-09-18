use axum::{http::StatusCode, response::IntoResponse};

#[derive(Debug)]
pub enum PropertyError {
	NotFound,
	SqlError(sqlx::Error),
	RenderError(askama::Error),
	Validation(String),
	Media(crate::media::MediaError),
	Internal(String),
}

impl From<sqlx::Error> for PropertyError {
	fn from(e: sqlx::Error) -> Self {
		match e {
			sqlx::Error::RowNotFound => Self::NotFound,
			_ => Self::SqlError(e),
		}
	}
}

impl From<askama::Error> for PropertyError {
	fn from(e: askama::Error) -> Self {
		Self::RenderError(e)
	}
}

impl From<crate::media::MediaError> for PropertyError {
	fn from(e: crate::media::MediaError) -> Self {
		Self::Media(e)
	}
}

impl IntoResponse for PropertyError {
	fn into_response(self) -> axum::response::Response {
		tracing::error!(error = ?self, "property route error");
		match self {
			Self::NotFound => StatusCode::NOT_FOUND.into_response(),
			Self::Validation(_) => StatusCode::BAD_REQUEST.into_response(),
			_ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
		}
	}
}

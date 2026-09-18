use axum::{http::StatusCode, response::IntoResponse};

#[derive(Debug)]
pub enum ProjectError {
	NotFound,
	SqlError(sqlx::Error),
	RenderError(askama::Error),
	Validation(String),
	Media(crate::media::MediaError),
	Internal(String),
}

impl From<sqlx::Error> for ProjectError {
	fn from(e: sqlx::Error) -> Self {
		match e {
			sqlx::Error::RowNotFound => Self::NotFound,
			_ => Self::SqlError(e),
		}
	}
}

impl From<askama::Error> for ProjectError {
	fn from(e: askama::Error) -> Self {
		Self::RenderError(e)
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
			Self::NotFound => StatusCode::NOT_FOUND.into_response(),
			Self::Validation(_) => StatusCode::BAD_REQUEST.into_response(),
			_ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
		}
	}
}

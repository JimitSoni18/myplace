use axum::{http::StatusCode, response::IntoResponse};
use image::ImageError;

#[derive(Debug)]
pub enum OwnerUpdateFormGetError {
	OwnerNotFound,
	RenderError(askama::Error),
	SqlError(sqlx::Error),
}

impl IntoResponse for OwnerUpdateFormGetError {
	fn into_response(self) -> axum::response::Response {
		tracing::error!(error = ?self, "failed to get owner update form");
		match self {
			Self::OwnerNotFound => StatusCode::NOT_FOUND.into_response(),
			Self::RenderError(_) | Self::SqlError(_) => {
				StatusCode::INTERNAL_SERVER_ERROR.into_response()
			}
		}
	}
}

#[derive(Debug)]
pub enum OwnerCreateFormGetError {
	RenderError(askama::Error),
}

impl IntoResponse for OwnerCreateFormGetError {
	fn into_response(self) -> axum::response::Response {
		tracing::error!(error = ?self, "failed to render owner create form");
		StatusCode::INTERNAL_SERVER_ERROR.into_response()
	}
}

#[derive(Debug)]
pub enum ProjectOwnerListError {
	SqlError(sqlx::Error),
	RenderError(askama::Error),
}

impl IntoResponse for ProjectOwnerListError {
	fn into_response(self) -> axum::response::Response {
		tracing::error!(error = ?self, "failed to render project owner list");
		StatusCode::INTERNAL_SERVER_ERROR.into_response()
	}
}

#[derive(Debug)]
pub enum ProjectOwnerCreateError {
	SqlError(sqlx::Error),
	HashingError,
}

impl IntoResponse for ProjectOwnerCreateError {
	fn into_response(self) -> axum::response::Response {
		tracing::error!(error = ?self, "failed to create project owner");
		StatusCode::INTERNAL_SERVER_ERROR.into_response()
	}
}

#[derive(Debug)]
pub enum ProjectOwnerUpdateError {
	SqlError(sqlx::Error),
}

impl IntoResponse for ProjectOwnerUpdateError {
	fn into_response(self) -> axum::response::Response {
		tracing::error!(error = ?self, "failed to update project owner");
		StatusCode::INTERNAL_SERVER_ERROR.into_response()
	}
}

#[derive(Debug)]
pub enum ProjectOwnerDeleteError {
	SqlError(sqlx::Error),
}

impl IntoResponse for ProjectOwnerDeleteError {
	fn into_response(self) -> axum::response::Response {
		tracing::error!(error = ?self, "failed to delete project owner");
		StatusCode::INTERNAL_SERVER_ERROR.into_response()
	}
}

#[derive(Debug)]
pub enum OwnerDetailError {
	NotFound,
	SqlError(sqlx::Error),
	RenderError(askama::Error),
}

impl IntoResponse for OwnerDetailError {
	fn into_response(self) -> axum::response::Response {
		tracing::error!(error = ?self, "failed to get project owner detail");
		match self {
			Self::NotFound => StatusCode::NOT_FOUND.into_response(),
			_ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
		}
	}
}

#[derive(Debug)]
pub enum OwnerProjectsError {
	NotFound,
	SqlError(sqlx::Error),
	RenderError(askama::Error),
}

impl IntoResponse for OwnerProjectsError {
	fn into_response(self) -> axum::response::Response {
		tracing::error!(error = ?self, "failed to get project owner projects");
		match self {
			Self::NotFound => StatusCode::NOT_FOUND.into_response(),
			_ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
		}
	}
}

#[derive(Debug)]
pub enum OwnerToggleActiveError {
	SqlError(sqlx::Error),
}

impl IntoResponse for OwnerToggleActiveError {
	fn into_response(self) -> axum::response::Response {
		tracing::error!(error = ?self, "failed to toggle owner active status");
		StatusCode::INTERNAL_SERVER_ERROR.into_response()
	}
}

#[derive(Debug)]
pub enum ProjectOwnerImageUploadError {
	UnidentifiedImageFormat(std::io::Error),
	ImageDecodeError(ImageError),
	EncoderCreateError(String),
	FileFormatNotSupported,
	ImageFieldNotPresent,
	ImageUpdateError,
	FileNameMissing,
	FileReadError,
	Internal,
}

impl IntoResponse for ProjectOwnerImageUploadError {
	fn into_response(self) -> axum::response::Response {
		tracing::error!(error = ?self, "image upload error");
		StatusCode::INTERNAL_SERVER_ERROR.into_response()
	}
}

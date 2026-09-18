use axum::{http::StatusCode, response::{IntoResponse, Redirect}};

pub enum LocationListError {
	InternalError,
}

impl IntoResponse for LocationListError {
	fn into_response(self) -> axum::response::Response {
		match self {
			Self::InternalError => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
		}
	}
}

pub enum LocationEditFormGetError {
	NotFound,
	InternalError,
}

impl IntoResponse for LocationEditFormGetError {
	fn into_response(self) -> axum::response::Response {
		match self {
			Self::InternalError => (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
			Self::NotFound => (StatusCode::NOT_FOUND).into_response(),
		}
	}
}

pub enum LocationDeleteError {
    NotFound,
}

impl IntoResponse for LocationDeleteError {
	fn into_response(self) -> axum::response::Response {
		match self {
			Self::NotFound => Redirect::to("/admin/locations").into_response(),
		}
	}
}


use axum::{http::StatusCode, response::IntoResponse};

#[derive(Debug)]
pub enum AdminUserError {
	NotFound,
	BadRequest(String),
	Forbidden(String),
	SqlError(sqlx::Error),
	RenderError(askama::Error),
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

impl From<askama::Error> for AdminUserError {
	fn from(e: askama::Error) -> Self {
		Self::RenderError(e)
	}
}

impl IntoResponse for AdminUserError {
	fn into_response(self) -> axum::response::Response {
		tracing::error!(error = ?self, "admin user route error");
		match self {
			Self::NotFound => (StatusCode::NOT_FOUND, "Admin user not found").into_response(),
			Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
			Self::Forbidden(msg) => (StatusCode::FORBIDDEN, msg).into_response(),
			_ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
		}
	}
}

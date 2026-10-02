use axum::{
	extract::{FromRequest, Request},
	http::StatusCode,
	response::{IntoResponse, Response},
};
use std::fmt;

#[derive(Debug)]
pub enum FormRejection {
	InvalidBody(String),
	Deserialization(String),
}

impl fmt::Display for FormRejection {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::InvalidBody(msg) => write!(f, "Failed to read form body: {msg}"),
			Self::Deserialization(msg) => write!(f, "Failed to deserialize form body: {msg}"),
		}
	}
}

impl std::error::Error for FormRejection {}

impl IntoResponse for FormRejection {
	fn into_response(self) -> Response {
		tracing::warn!(rejection = %self, "form rejection");
		(StatusCode::BAD_REQUEST, self.to_string()).into_response()
	}
}

/// An Axum extractor for HTML form data backed by `serde_html_form`.
/// Supports:
/// - Single checkbox value (`amenities=1`) -> `Vec<i32>`
/// - Repeated checkbox values (`amenities=1&amenities=2`) -> `Vec<i32>`
/// - Unchecked / missing fields -> `Vec::default()` with `#[serde(default)]`
/// - Safe error handling: malformed / unprocessable inputs return 400 Bad Request instead of 500.
#[derive(Debug, Clone, Copy, Default)]
pub struct Form<T>(pub T);

impl<T> std::ops::Deref for Form<T> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T> std::ops::DerefMut for Form<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl<S, T> FromRequest<S> for Form<T>
where
	T: serde::de::DeserializeOwned,
	S: Send + Sync,
{
	type Rejection = FormRejection;

	async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
		let bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
			.await
			.map_err(|e| FormRejection::InvalidBody(e.to_string()))?;

		let value = serde_html_form::from_bytes::<T>(&bytes)
			.map_err(|e| FormRejection::Deserialization(e.to_string()))?;

		Ok(Form(value))
	}
}

#[cfg(test)]
mod tests {
	use serde::Deserialize;

	#[derive(Debug, PartialEq, Eq, Deserialize)]
	struct ProjectTestPayload {
		name: String,
		category: String,
		project_owner_id: i32,
		location_id: i32,
		#[serde(default)]
		amenities: Vec<i32>,
	}

	#[test]
	fn test_no_amenities() {
		let body = b"name=Sunset&category=Residential&project_owner_id=1&location_id=1";
		let parsed: ProjectTestPayload = serde_html_form::from_bytes(body).unwrap();
		assert_eq!(parsed.name, "Sunset");
		assert_eq!(parsed.amenities, Vec::<i32>::new());
	}

	#[test]
	fn test_one_amenity() {
		let body = b"name=Sunset&category=Residential&project_owner_id=1&location_id=1&amenities=1";
		let parsed: ProjectTestPayload = serde_html_form::from_bytes(body).unwrap();
		assert_eq!(parsed.name, "Sunset");
		assert_eq!(parsed.amenities, vec![1]);
	}

	#[test]
	fn test_multiple_amenities() {
		let body = b"name=Sunset&category=Residential&project_owner_id=1&location_id=1&amenities=1&amenities=2&amenities=5";
		let parsed: ProjectTestPayload = serde_html_form::from_bytes(body).unwrap();
		assert_eq!(parsed.name, "Sunset");
		assert_eq!(parsed.amenities, vec![1, 2, 5]);
	}

	#[test]
	fn test_duplicate_amenities() {
		let body = b"name=Sunset&category=Residential&project_owner_id=1&location_id=1&amenities=1&amenities=1";
		let parsed: ProjectTestPayload = serde_html_form::from_bytes(body).unwrap();
		assert_eq!(parsed.amenities, vec![1, 1]);
	}

	#[test]
	fn test_malformed_numeric_amenity() {
		let body =
			b"name=Sunset&category=Residential&project_owner_id=1&location_id=1&amenities=abc";
		let res: Result<ProjectTestPayload, _> = serde_html_form::from_bytes(body);
		assert!(res.is_err());
	}

	#[test]
	fn test_missing_required_fields() {
		let body = b"amenities=1";
		let res: Result<ProjectTestPayload, _> = serde_html_form::from_bytes(body);
		assert!(res.is_err());
	}

	#[derive(Debug, PartialEq, Deserialize)]
	struct NumericTestPayload {
		bedroom_count: Option<f64>,
		bathroom_count: Option<f64>,
		balcony_count: Option<i16>,
	}

	#[test]
	fn test_numeric_empty_string_and_values() {
		// Test empty strings
		let empty_body = b"bedroom_count=&bathroom_count=&balcony_count=";
		let parsed: NumericTestPayload = serde_html_form::from_bytes(empty_body).unwrap();
		assert_eq!(parsed.bedroom_count, None);
		assert_eq!(parsed.bathroom_count, None);
		assert_eq!(parsed.balcony_count, None);

		// Test fractional and large values
		let val_body = b"bedroom_count=2.5&bathroom_count=1.5&balcony_count=3";
		let parsed: NumericTestPayload = serde_html_form::from_bytes(val_body).unwrap();
		assert_eq!(parsed.bedroom_count, Some(2.5));
		assert_eq!(parsed.bathroom_count, Some(1.5));
		assert_eq!(parsed.balcony_count, Some(3));
	}
}

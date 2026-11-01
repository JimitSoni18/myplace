use axum::{
	http::{HeaderValue, StatusCode, header::SET_COOKIE},
	response::IntoResponseParts,
};

pub struct SetAuthCookie {
	cookie_values: Vec<String>,
}

impl SetAuthCookie {
	pub fn new(session_id: &str, expiration_duration_secs: u64) -> Self {
		Self::new_with_name("session_id", session_id, expiration_duration_secs)
	}

	pub fn new_with_name(name: &str, session_id: &str, expiration_duration_secs: u64) -> Self {
		Self {
			cookie_values: vec![format!(
				"{name}={session_id}; Max-Age={expiration_duration_secs}; HttpOnly; SameSite=Lax; Path=/"
			)],
		}
	}

	pub fn clear_all() -> Self {
		Self {
			cookie_values: vec![
				"admin_session_id=; Max-Age=0; HttpOnly; SameSite=Lax; Path=/".to_string(),
				"owner_session_id=; Max-Age=0; HttpOnly; SameSite=Lax; Path=/".to_string(),
				"session_id=; Max-Age=0; HttpOnly; SameSite=Lax; Path=/".to_string(),
			],
		}
	}
}

impl IntoResponseParts for SetAuthCookie {
	type Error = StatusCode;
	fn into_response_parts(
		self,
		mut res: axum::response::ResponseParts,
	) -> Result<axum::response::ResponseParts, Self::Error> {
		for cv in self.cookie_values {
			let header_value =
				HeaderValue::from_str(&cv).or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;
			res.headers_mut().append(SET_COOKIE, header_value);
		}
		Ok(res)
	}
}

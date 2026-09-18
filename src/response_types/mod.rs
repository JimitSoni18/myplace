use axum::{
	http::{HeaderValue, StatusCode, header::SET_COOKIE},
	response::IntoResponseParts,
};

pub struct SetAuthCookie {
	cookie_value: String,
}

impl SetAuthCookie {
	pub fn new(session_id: &str, expiration_duration_secs: u64) -> Self {
		// FIXME: Secure;
		Self {
			cookie_value: format!(
				"session_id={session_id}; Max-Age={expiration_duration_secs}; HttpOnly; SameSite=Strict; Path=/"
			),
		}
	}
}

impl IntoResponseParts for SetAuthCookie {
	type Error = StatusCode;
	fn into_response_parts(
		self,
		mut res: axum::response::ResponseParts,
	) -> Result<axum::response::ResponseParts, Self::Error> {
		let header_value =
			HeaderValue::from_str(&self.cookie_value).or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;
		res.headers_mut().insert(SET_COOKIE, header_value);
		Ok(res)
	}
}

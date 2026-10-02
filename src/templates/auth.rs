use askama::Template;

#[derive(Template)]
#[template(path = "auth/login.html")]
pub struct LoginTemplate {
	pub incorrect_password_error: bool,
}

#[derive(Template, Default)]
#[template(path = "auth/owner-login.html")]
pub struct OwnerLoginTemplate {
	pub incorrect_password_error: bool,
	pub inactive_account_error: bool,
}

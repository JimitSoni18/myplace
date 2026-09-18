use std::sync::LazyLock;

use askama::Template;

use crate::templates::{
	admin::{AdminPage, LocationForm},
	auth::LoginTemplate,
};

pub static LOGIN_HTML: LazyLock<&'static str> = LazyLock::new(|| {
	LoginTemplate {
		incorrect_password_error: false,
	}
	.render()
	.unwrap()
	.leak()
});

use axum::response::Html;

pub const NO_ACCESS_PAGE: Html<&'static str> = Html(include_str!("../static/no-access.html"));

pub static LOCATION_CREATE_FORM_HTML: LazyLock<&'static str> = LazyLock::new(|| {
	LocationForm {
		page: AdminPage::Locations,
		..Default::default()
	}
	.render()
	.unwrap()
	.leak()
});


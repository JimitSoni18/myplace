use axum::{
	Extension, Router,
	extract::State,
	response::{IntoResponse, Redirect},
	routing::get,
};

use crate::{
	AppState,
	response_types::SetAuthCookie,
	session_store::{SessionId, SessionStoreTrait as _},
};

pub mod amenities;
pub mod dashboard;
pub mod locations;
pub mod project_owners;
pub mod projects;
pub mod properties;

pub fn router() -> Router<AppState> {
	Router::new()
		.merge(dashboard::router())
		.nest("/dashboard", dashboard::router())
		.nest("/locations", locations::router())
		.nest("/owners", project_owners::router())
		.nest("/projects", projects::router())
		.nest("/properties", properties::router())
		.nest("/amenities", amenities::router())
		.route("/admin-logout", get(logout))
}

async fn logout(session_id: Extension<SessionId>, state: State<AppState>) -> impl IntoResponse {
	state.session_store.delete_session(&session_id).await;
	(SetAuthCookie::new("", 0), Redirect::to("/auth/admin-login"))
}

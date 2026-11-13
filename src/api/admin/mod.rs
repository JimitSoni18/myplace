use axum::Router;

use crate::AppState;

pub mod admin_users;
pub mod amenities;
pub mod dashboard;
pub mod locations;
pub mod meta;
pub mod project_owners;
pub mod projects;
pub mod search;

pub fn router() -> Router<AppState> {
	Router::new()
		.nest("/search", search::router())
		.nest("/meta", meta::router())
		.nest("/dashboard", dashboard::router())
		.nest("/locations", locations::router())
		.nest("/owners", project_owners::router())
		.nest("/project_owners", project_owners::router())
		.nest("/projects", projects::router())
		.nest("/amenities", amenities::router())
		.nest("/users", admin_users::router())
		.nest("/notifications", crate::api::notifications::admin_router())
}

use askama::Template;
use axum::{
	Extension, Form, Router,
	extract::{Path, Query, State},
	http::StatusCode,
	response::{Html, IntoResponse, Redirect},
	routing::{MethodRouter, get},
};
use serde::Deserialize;

use crate::{
	AppState,
	api::admin::locations::error::{LocationDeleteError, LocationEditFormGetError},
	session_store::AuthUser,
	templates::admin::{AdminPage, LocationEditFormValues, LocationForm, LocationTemplate},
	utils,
};

mod error;

// ---------------------------------------------------------------------------
// DB row — matches the new structured locations schema
// ---------------------------------------------------------------------------
pub struct LocationItem {
	pub id: i32,
	pub formatted_address: String,
	pub city: Option<String>,
	pub state_or_province: Option<String>,
	pub area: Option<String>,
}

#[derive(Deserialize)]
struct CreateOrUpdateLocationPayload {
	formatted_address: String,
	city: Option<String>,
	state_or_province: Option<String>,
	area: Option<String>,
}

pub fn router() -> Router<AppState> {
	let create_router = MethodRouter::new()
		.get(location_create_form)
		.post(create_location);
	let edit_router = MethodRouter::new()
		.get(location_edit_form)
		.put(update_location)
		.delete(delete_location);
	Router::new()
		.route("/", get(location_list))
		.route("/new", create_router)
		.route("/{id}", edit_router)
}

async fn location_create_form(
	Extension(auth_user): Extension<AuthUser>,
) -> Result<Html<String>, error::LocationListError> {
	Ok(Html(
		LocationForm {
			page: AdminPage::Locations,
			admin_name: &auth_user.username,
			edit_values: None,
		}
		.render()
		.or(Err(error::LocationListError::InternalError))?,
	))
}

async fn update_location(
	state: State<AppState>,
	Path(id): Path<i32>,
	payload: Form<CreateOrUpdateLocationPayload>,
) -> impl IntoResponse {
	let result = sqlx::query_file!(
		"queries/locations/update_location.sql",
		payload.formatted_address,
		payload.city,
		payload.state_or_province,
		payload.area,
		id,
	)
	.execute(&state.model.db)
	.await;

	if result.is_err() {
		return Err(StatusCode::INTERNAL_SERVER_ERROR);
	}
	Ok(StatusCode::OK)
}

async fn delete_location(
	state: State<AppState>,
	Path(id): Path<i32>,
) -> Result<Redirect, LocationDeleteError> {
	sqlx::query_file!("queries/locations/delete_location.sql", id)
		.execute(&state.model.db)
		.await
		.or(Err(LocationDeleteError::NotFound))?;
	Ok(Redirect::to("/admin/locations"))
}

async fn location_edit_form(
	state: State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
	Path(id): Path<i32>,
) -> Result<Html<String>, LocationEditFormGetError> {
	let location =
		sqlx::query_file_as!(LocationItem, "queries/locations/get_location_by_id.sql", id)
			.fetch_one(&state.model.db)
			.await
			.map_err(|e| match e {
				sqlx::Error::RowNotFound => LocationEditFormGetError::NotFound,
				_ => LocationEditFormGetError::InternalError,
			})?;
	Ok(Html(
		LocationForm {
			edit_values: Some(LocationEditFormValues {
				error: None,
				default_values: location,
			}),
			page: AdminPage::Locations,
			admin_name: &auth_user.username,
		}
		.render()
		.or(Err(LocationEditFormGetError::InternalError))?,
	))
}

async fn create_location(
	state: State<AppState>,
	payload: Form<CreateOrUpdateLocationPayload>,
) -> impl IntoResponse {
	let result = sqlx::query_file!(
		"queries/locations/create_location.sql",
		payload.formatted_address,
		payload.city,
		payload.state_or_province,
		payload.area,
	)
	.execute(&state.model.db)
	.await;
	if result.is_err() {
		return Err(StatusCode::INTERNAL_SERVER_ERROR);
	}
	Ok(Redirect::to("/admin/locations"))
}

#[derive(Deserialize)]
struct LocationQuery {
	q: Option<String>,
}

async fn location_list(
	Extension(auth_user): Extension<AuthUser>,
	state: State<AppState>,
	Query(query): Query<LocationQuery>,
) -> Result<Html<String>, error::LocationListError> {
	let (query_str, locations) = match query.q {
		Some(ref q) if !q.is_empty() => {
			let escaped = utils::sql::escape(q.trim());
			let rgx = format!("%{escaped}%");
			(
				query.q,
				sqlx::query_file_as!(LocationItem, "queries/locations/search_locations.sql", rgx,)
					.fetch_all(&state.model.db)
					.await
					.or(Err(error::LocationListError::InternalError))?,
			)
		}
		_ => (
			None,
			sqlx::query_file_as!(LocationItem, "queries/locations/list_locations.sql")
				.fetch_all(&state.model.db)
				.await
				.or(Err(error::LocationListError::InternalError))?,
		),
	};

	Ok(Html(
		LocationTemplate {
			admin_name: &auth_user.username,
			page: AdminPage::Locations,
			q: query_str.as_deref(),
			locations,
		}
		.render()
		.or(Err(error::LocationListError::InternalError))?,
	))
}

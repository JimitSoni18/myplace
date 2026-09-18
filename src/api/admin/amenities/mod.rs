use askama::Template;
use axum::{
	Extension, Form, Router,
	extract::{Path, State},
	http::StatusCode,
	response::{Html, Redirect},
	routing::{get, post},
};
use serde::Deserialize;

use crate::{
	AppState,
	session_store::AuthUser,
	templates::admin::{AdminPage, AmenityItem, AmenityTemplate},
	utils::sql::slugify,
};

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/", get(amenity_list).post(create_amenity))
		.route("/{id}/toggle", post(toggle_amenity))
		.route("/{id}", axum::routing::delete(delete_amenity))
}

#[derive(Deserialize)]
pub struct CreateAmenityPayload {
	pub name: String,
}

async fn amenity_list(
	State(state): State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
) -> Result<Html<String>, StatusCode> {
	struct DbRow {
		id: i32,
		name: String,
		slug: String,
		is_active: bool,
		projects_count: i64,
	}

	let rows = sqlx::query_as!(
		DbRow,
		r#"
		SELECT
			a.id, a.name, a.slug, a.is_active,
			COUNT(pa.project_id) as "projects_count!: i64"
		FROM amenities a
			LEFT JOIN project_amenities pa ON pa.amenity_id = a.id
		GROUP BY a.id
		ORDER BY a.name ASC
		"#
	)
	.fetch_all(&state.model.db)
	.await
	.map_err(|e| {
		tracing::error!(error = ?e, "failed to fetch amenities");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	let amenities = rows
		.into_iter()
		.map(|r| AmenityItem {
			id: r.id,
			name: r.name,
			slug: r.slug,
			is_active: r.is_active,
			projects_count: r.projects_count,
		})
		.collect();

	let template = AmenityTemplate {
		admin_name: &auth_user.username,
		page: AdminPage::Amenities,
		amenities,
	};

	let html = template.render().map_err(|e| {
		tracing::error!(error = ?e, "failed to render amenities template");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	Ok(Html(html))
}

async fn create_amenity(
	State(state): State<AppState>,
	Form(payload): Form<CreateAmenityPayload>,
) -> Result<Redirect, StatusCode> {
	let trimmed = payload.name.trim();
	if trimmed.is_empty() {
		return Ok(Redirect::to("/admin/amenities"));
	}

	let slug = slugify(trimmed);

	sqlx::query!(
		"INSERT INTO amenities (name, slug) VALUES ($1, $2) ON CONFLICT (name) DO NOTHING",
		trimmed,
		slug,
	)
	.execute(&state.model.db)
	.await
	.map_err(|e| {
		tracing::error!(error = ?e, "failed to insert amenity");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	tracing::info!(name = trimmed, slug = %slug, "created amenity");
	Ok(Redirect::to("/admin/amenities"))
}

async fn toggle_amenity(
	State(state): State<AppState>,
	Path(id): Path<i32>,
) -> Result<Redirect, StatusCode> {
	sqlx::query!(
		"UPDATE amenities SET is_active = NOT is_active WHERE id = $1",
		id
	)
	.execute(&state.model.db)
	.await
	.map_err(|e| {
		tracing::error!(error = ?e, "failed to toggle amenity");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	Ok(Redirect::to("/admin/amenities"))
}

async fn delete_amenity(
	State(state): State<AppState>,
	Path(id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
	sqlx::query!("DELETE FROM amenities WHERE id = $1", id)
		.execute(&state.model.db)
		.await
		.map_err(|e| {
			tracing::error!(error = ?e, "failed to delete amenity");
			StatusCode::INTERNAL_SERVER_ERROR
		})?;

	Ok(StatusCode::NO_CONTENT)
}

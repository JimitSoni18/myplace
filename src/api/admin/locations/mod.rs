use axum::{
	Json, Router,
	extract::{Query, State},
	routing::get,
};
use serde::{Deserialize, Serialize};

use crate::{AppState, session_store::AuthUser};

#[derive(Deserialize, ts_rs::TS)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct LocationSuggestionQuery {
	pub field: Option<String>,
	pub city: Option<String>,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "admin/")]
pub struct LocationSuggestionsResponse {
	pub cities: Vec<String>,
	pub areas: Vec<String>,
	pub states: Vec<String>,
}

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/suggestions", get(get_suggestions))
		.route("/cities", get(get_cities))
		.route("/areas", get(get_areas))
		.route("/states", get(get_states))
}

async fn get_suggestions(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Query(query): Query<LocationSuggestionQuery>,
) -> Json<LocationSuggestionsResponse> {
	let db = &state.model.db;

	let cities = sqlx::query_file_scalar!("queries/locations/suggest_cities.sql")
		.fetch_all(db)
		.await
		.unwrap_or_default();

	let city_filter = query.city.as_deref().filter(|s| !s.trim().is_empty());
	let areas = sqlx::query_file_scalar!("queries/locations/suggest_areas.sql", city_filter)
		.fetch_all(db)
		.await
		.unwrap_or_default();

	let states = sqlx::query_file_scalar!("queries/locations/suggest_states.sql")
		.fetch_all(db)
		.await
		.unwrap_or_default();

	Json(LocationSuggestionsResponse {
		cities,
		areas,
		states,
	})
}

async fn get_cities(
	State(state): State<AppState>,
	_auth_user: AuthUser,
) -> Json<Vec<String>> {
	let cities = sqlx::query_file_scalar!("queries/locations/suggest_cities.sql")
		.fetch_all(&state.model.db)
		.await
		.unwrap_or_default();
	Json(cities)
}

async fn get_areas(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Query(query): Query<LocationSuggestionQuery>,
) -> Json<Vec<String>> {
	let city_filter = query.city.as_deref().filter(|s| !s.trim().is_empty());
	let areas = sqlx::query_file_scalar!("queries/locations/suggest_areas.sql", city_filter)
		.fetch_all(&state.model.db)
		.await
		.unwrap_or_default();
	Json(areas)
}

async fn get_states(
	State(state): State<AppState>,
	_auth_user: AuthUser,
) -> Json<Vec<String>> {
	let states = sqlx::query_file_scalar!("queries/locations/suggest_states.sql")
		.fetch_all(&state.model.db)
		.await
		.unwrap_or_default();
	Json(states)
}

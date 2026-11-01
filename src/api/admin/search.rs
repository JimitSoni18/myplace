use axum::{
	Json, Router,
	extract::{Query, State},
	routing::get,
};
use serde::{Deserialize, Serialize};

use crate::{AppState, session_store::AuthUser};

pub fn router() -> Router<AppState> {
	Router::new().route("/", get(global_search))
}

#[derive(Deserialize, ts_rs::TS)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct SearchQuery {
	#[serde(default)]
	pub q: Option<String>,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "admin/")]
pub struct SearchResultProject {
	pub id: i32,
	pub name: String,
	pub slug: String,
	pub category: String,
	pub url: String,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct SearchResultProperty {
	pub id: i32,
	pub unit_number: Option<String>,
	pub building: Option<String>,
	pub project_name: String,
	pub url: String,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "admin/")]
pub struct SearchResultOwner {
	pub id: i32,
	pub name: String,
	pub active: bool,
	pub url: String,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "admin/")]
pub struct SearchResults {
	pub projects: Vec<SearchResultProject>,
	pub properties: Vec<SearchResultProperty>,
	pub owners: Vec<SearchResultOwner>,
}

async fn global_search(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Query(query): Query<SearchQuery>,
) -> Json<SearchResults> {
	let q_str = query.q.unwrap_or_default().trim().to_string();
	if q_str.len() < 2 {
		return Json(SearchResults {
			projects: vec![],
			properties: vec![],
			owners: vec![],
		});
	}

	let pattern = format!("%{q_str}%");
	let db = &state.model.db;

	#[derive(sqlx::FromRow)]
	struct DbProj {
		id: i32,
		name: String,
		slug: String,
		category: String,
	}

	let projects_raw = sqlx::query_as::<_, DbProj>(
		r#"
		SELECT id, name, slug, category
		FROM projects
		WHERE (name ILIKE $1 OR slug ILIKE $1) AND deleted_at IS NULL
		ORDER BY id DESC
		LIMIT 5
		"#,
	)
	.bind(&pattern)
	.fetch_all(db)
	.await
	.unwrap_or_default();

	#[derive(sqlx::FromRow)]
	struct DbProp {
		id: i32,
		project_id: i32,
		unit_number: Option<String>,
		building: Option<String>,
		project_name: String,
	}

	let properties_raw = sqlx::query_as::<_, DbProp>(
		r#"
		SELECT p.id, p.project_id, p.unit_number, p.building, pr.name as project_name
		FROM properties p
		JOIN projects pr ON p.project_id = pr.id
		WHERE (p.unit_number ILIKE $1 OR p.building ILIKE $1) AND p.deleted_at IS NULL
		ORDER BY p.id DESC
		LIMIT 5
		"#,
	)
	.bind(&pattern)
	.fetch_all(db)
	.await
	.unwrap_or_default();

	#[derive(sqlx::FromRow)]
	struct DbOwner {
		id: i32,
		name: String,
		active: bool,
	}

	let owners_raw = sqlx::query_as::<_, DbOwner>(
		r#"
		SELECT id, name, active
		FROM project_owners
		WHERE name ILIKE $1 AND deleted_at IS NULL
		ORDER BY id DESC
		LIMIT 5
		"#,
	)
	.bind(&pattern)
	.fetch_all(db)
	.await
	.unwrap_or_default();

	Json(SearchResults {
		projects: projects_raw
			.into_iter()
			.map(|p| SearchResultProject {
				url: format!("/projects/{}", p.id),
				id: p.id,
				name: p.name,
				slug: p.slug,
				category: p.category,
			})
			.collect(),
		properties: properties_raw
			.into_iter()
			.map(|p| SearchResultProperty {
				url: format!("/projects/{}?tab=units", p.project_id),
				id: p.id,
				unit_number: p.unit_number,
				building: p.building,
				project_name: p.project_name,
			})
			.collect(),
		owners: owners_raw
			.into_iter()
			.map(|o| SearchResultOwner {
				url: format!("/owners/{}", o.id),
				id: o.id,
				name: o.name,
				active: o.active,
			})
			.collect(),
	})
}

use askama::Template;
use axum::{
	Extension, Router,
	extract::{Path, State},
	http::StatusCode,
	response::{Html, IntoResponse, Redirect},
	routing::get,
};
use uuid::Uuid;

use crate::{
	AppState,
	config::CONFIG,
	middlewares::owner_authenticate::OwnerUser,
	response_types::SetAuthCookie,
	session_store::{SessionId, SessionStoreTrait as _},
	templates::{
		admin::{ProjectDocumentItem, ProjectMediaItem},
		owner::{
			OwnerDashboardProject, OwnerDashboardTemplate, OwnerProjectDetailData,
			OwnerProjectDetailTemplate, OwnerProjectListTemplate,
		},
	},
	utils::markdown::render_markdown,
};

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/", get(owner_dashboard))
		.route("/projects", get(owner_projects_list))
		.route("/projects/{id}", get(owner_project_detail))
		.route("/logout", get(owner_logout))
}

fn format_opt_date(d: Option<time::Date>) -> Option<String> {
	let fmt = time::macros::format_description!("[month repr:short] [day], [year]");
	d.and_then(|date| date.format(&fmt).ok())
}

fn format_bytes(bytes: Option<i64>) -> String {
	let b = bytes.unwrap_or(0);
	if b < 1024 {
		format!("{b} B")
	} else if b < 1024 * 1024 {
		format!("{:.1} KB", b as f64 / 1024.0)
	} else {
		format!("{:.1} MB", b as f64 / (1024.0 * 1024.0))
	}
}

async fn owner_dashboard(
	owner: OwnerUser,
	State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
	struct DbRow {
		id: i32,
		name: String,
		slug: String,
		category: String,
		location: Option<String>,
		properties_count: i64,
		hero_thumb_key: Option<String>,
	}

	let rows = sqlx::query_file_as!(
		DbRow,
		"queries/projects/list_owner_portal_projects.sql",
		owner.owner_id
	)
	.fetch_all(&state.model.db)
	.await
	.map_err(|e| {
		tracing::error!(error = ?e, "failed to fetch owner projects");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	let total_projects = rows.len() as i64;
	let total_properties: i64 = rows.iter().map(|r| r.properties_count).sum();

	let recent_projects = rows
		.into_iter()
		.map(|r| {
			let hero_thumb_url = r.hero_thumb_key.as_ref().map(|k| CONFIG.asset_url(k));
			OwnerDashboardProject {
				id: r.id,
				name: r.name,
				slug: r.slug,
				category: r.category,
				location: r.location.unwrap_or_else(|| "Unspecified".to_string()),
				properties_count: r.properties_count,
				hero_thumb_url,
			}
		})
		.collect();

	let template = OwnerDashboardTemplate {
		owner_name: &owner.owner_name,
		username: &owner.username,
		total_projects,
		total_properties,
		recent_projects,
	};

	let html = template.render().map_err(|e| {
		tracing::error!(error = ?e, "failed to render owner dashboard");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	Ok(Html(html))
}

async fn owner_projects_list(
	owner: OwnerUser,
	State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
	struct DbRow {
		id: i32,
		name: String,
		slug: String,
		category: String,
		location: Option<String>,
		properties_count: i64,
		hero_thumb_key: Option<String>,
	}

	let rows = sqlx::query_file_as!(
		DbRow,
		"queries/projects/list_owner_portal_projects.sql",
		owner.owner_id
	)
	.fetch_all(&state.model.db)
	.await
	.map_err(|e| {
		tracing::error!(error = ?e, "failed to fetch owner project list");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	let projects = rows
		.into_iter()
		.map(|r| {
			let hero_thumb_url = r.hero_thumb_key.as_ref().map(|k| CONFIG.asset_url(k));
			OwnerDashboardProject {
				id: r.id,
				name: r.name,
				slug: r.slug,
				category: r.category,
				location: r.location.unwrap_or_else(|| "Unspecified".to_string()),
				properties_count: r.properties_count,
				hero_thumb_url,
			}
		})
		.collect();

	let template = OwnerProjectListTemplate {
		owner_name: &owner.owner_name,
		username: &owner.username,
		projects,
	};

	let html = template.render().map_err(|e| {
		tracing::error!(error = ?e, "failed to render owner project list");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	Ok(Html(html))
}

async fn owner_project_detail(
	owner: OwnerUser,
	State(state): State<AppState>,
	Path(id): Path<i32>,
) -> Result<Html<String>, StatusCode> {
	struct DbProject {
		id: i32,
		name: String,
		slug: String,
		description: Option<String>,
		category: String,
		location_name: Option<String>,
		start_date: Option<time::Date>,
		launch_date: Option<time::Date>,
		possession_date: Option<time::Date>,
	}

	// Strictly scoped to owner_id
	let p = sqlx::query_file_as!(
		DbProject,
		"queries/projects/get_owner_portal_project_detail.sql",
		id,
		owner.owner_id
	)
	.fetch_optional(&state.model.db)
	.await
	.map_err(|e| {
		tracing::error!(error = ?e, "db error fetching project");
		StatusCode::INTERNAL_SERVER_ERROR
	})?
	.ok_or(StatusCode::NOT_FOUND)?;

	let count_row = sqlx::query_file!("queries/projects/count_project_properties.sql", id)
		.fetch_one(&state.model.db)
		.await
		.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

	struct DbMedia {
		media_id: Uuid,
		media_type: String,
		s3_key: String,
		thumbnail_key: Option<String>,
		sequence: i16,
	}

	let media_rows = sqlx::query_file_as!(DbMedia, "queries/projects/get_project_media.sql", id)
		.fetch_all(&state.model.db)
		.await
		.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

	let media = media_rows
		.into_iter()
		.map(|r| {
			let url = CONFIG.asset_url(&r.s3_key);
			let thumbnail_url = r
				.thumbnail_key
				.as_ref()
				.map(|k| CONFIG.asset_url(k))
				.unwrap_or_else(|| url.clone());
			ProjectMediaItem {
				id: r.media_id,
				media_type: r.media_type,
				url,
				thumbnail_url,
				sequence: r.sequence,
			}
		})
		.collect();

	struct DbDoc {
		id: i32,
		media_id: Uuid,
		display_name: String,
		doc_type: String,
		s3_key: String,
		file_size: Option<i64>,
	}

	let doc_rows = sqlx::query_file_as!(DbDoc, "queries/projects/get_project_documents.sql", id)
		.fetch_all(&state.model.db)
		.await
		.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

	let documents = doc_rows
		.into_iter()
		.map(|r| {
			let url = CONFIG.asset_url(&r.s3_key);
			let file_size_formatted = format_bytes(r.file_size);
			ProjectDocumentItem {
				id: r.id,
				media_id: r.media_id,
				display_name: r.display_name,
				doc_type: r.doc_type,
				url,
				file_size_formatted,
			}
		})
		.collect();

	let description_html = p
		.description
		.as_ref()
		.map(|desc| render_markdown(desc))
		.unwrap_or_default();

	let project_data = OwnerProjectDetailData {
		id: p.id,
		name: p.name,
		slug: p.slug,
		category: p.category,
		location_name: p.location_name.unwrap_or_else(|| "Unspecified".to_string()),
		start_date: format_opt_date(p.start_date),
		launch_date: format_opt_date(p.launch_date),
		possession_date: format_opt_date(p.possession_date),
	};

	let template = OwnerProjectDetailTemplate {
		owner_name: &owner.owner_name,
		username: &owner.username,
		project: project_data,
		description_html,
		properties_count: count_row.count,
		media,
		documents,
	};

	let html = template.render().map_err(|e| {
		tracing::error!(error = ?e, "failed to render owner project detail");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	Ok(Html(html))
}

async fn owner_logout(
	_owner: OwnerUser,
	session_id: Extension<SessionId>,
	State(state): State<AppState>,
) -> impl IntoResponse {
	state.session_store.delete_session(&session_id).await;
	(SetAuthCookie::new("", 0), Redirect::to("/auth/owner-login"))
}

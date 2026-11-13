use askama::Template;
use axum::{
	Form, Router,
	extract::{Path, State},
	http::{StatusCode, header},
	response::{Html, IntoResponse, Redirect, Response},
	routing::{get, post},
};
use serde::Deserialize;

use crate::{
	AppState,
	config::CONFIG,
	templates::public::{
		CategoryStat, HomeTemplate, OwnerDetailTemplate, ProjectDetailTemplate,
		ProjectsListTemplate, PublicConfigurationItem, PublicDocumentItem, PublicFloorPlanItem,
		PublicLandmarkItem, PublicMediaItem, PublicOwnerCard, PublicOwnerDetail,
		PublicOwnerSummary, PublicProjectCard, PublicProjectDetail, PublicSpecificationItem,
		PublicTypeOption,
	},
	utils::{
		form::{empty_as_none, HtmlQuery},
		markdown::render_markdown,
	},
};

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/", get(home_page))
		.route("/projects", get(projects_list))
		.route("/projects/{slug}", get(project_detail))
		.route("/projects/{id}/enquire", post(project_enquire))
		.route("/owners/{slug}", get(owner_detail))
		.route("/properties", get(properties_redirect))
		.route("/properties/{slug}", get(properties_redirect))
		.route("/sitemap.xml", get(sitemap_index_xml))
		.route("/sitemaps/{shard}", get(sitemap_shard_xml))
		.route("/robots.txt", get(robots_txt))
}

async fn properties_redirect() -> Response {
	redirect_301("/projects")
}

fn redirect_301(location: &str) -> Response {
	(
		StatusCode::MOVED_PERMANENTLY,
		[(header::LOCATION, location.to_string())],
	)
		.into_response()
}

fn format_opt_date(d: Option<time::Date>) -> Option<String> {
	let fmt = time::macros::format_description!("[month repr:short] [year]");
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

fn format_price(p: f64) -> String {
	if p <= 0.0 {
		"Price on Request".to_string()
	} else if p >= 10_000_000.0 {
		format!("₹ {:.2} Cr", p / 10_000_000.0)
	} else if p >= 100_000.0 {
		format!("₹ {:.2} L", p / 100_000.0)
	} else {
		format!("₹ {:.0}", p)
	}
}

// ---------------------------------------------------------------------------
// 1. HOME PAGE
// ---------------------------------------------------------------------------
async fn home_page(State(state): State<AppState>) -> Result<Html<String>, StatusCode> {
	let cache_key = "/";
	if let Some(cached) = state.page_cache.get(cache_key).await {
		return Ok(Html(cached));
	}

	struct DbProjectRow {
		id: i32,
		name: String,
		slug: String,
		project_type_name: String,
		city: String,
		area: String,
		formatted_address: String,
		configurations_count: i64,
		possession_date: Option<time::Date>,
		owner_id: i32,
		owner_name: String,
		owner_slug: String,
		hero_thumb_key: Option<String>,
		min_price: Option<f64>,
	}

	let project_rows = sqlx::query_file_as!(DbProjectRow, "queries/projects/featured_projects.sql")
		.fetch_all(&state.model.db)
		.await
		.unwrap_or_default();

	let featured_projects: Vec<PublicProjectCard> = project_rows
		.into_iter()
		.map(|p| {
			let hero_thumb_url = p.hero_thumb_key.as_ref().map(|k| CONFIG.asset_url(k));
			let min_price_formatted = p.min_price.map(format_price);
			PublicProjectCard {
				id: p.id,
				name: p.name,
				slug: p.slug,
				project_type_name: p.project_type_name,
				city: p.city,
				area: p.area,
				formatted_address: p.formatted_address,
				hero_thumb_url,
				configurations_count: p.configurations_count,
				possession_date: format_opt_date(p.possession_date),
				owner_id: p.owner_id,
				owner_name: p.owner_name,
				owner_slug: p.owner_slug,
				min_price_formatted,
				bhk_summary: None,
			}
		})
		.collect();

	// Featured Owners
	struct DbOwnerRow {
		id: i32,
		name: String,
		slug: String,
		profile_img_thumb_key: Option<String>,
		project_count: i64,
	}

	let owner_rows = sqlx::query_file_as!(DbOwnerRow, "queries/project_owners/featured_owners.sql")
		.fetch_all(&state.model.db)
		.await
		.unwrap_or_default();

	let featured_owners: Vec<PublicOwnerCard> = owner_rows
		.into_iter()
		.map(|o| {
			let image_url = o.profile_img_thumb_key.as_ref().map(|k| CONFIG.asset_url(k));
			PublicOwnerCard {
				id: o.id,
				name: o.name,
				slug: o.slug,
				image_url,
				project_count: o.project_count,
			}
		})
		.collect();

	// Category Stats (from project types)
	#[derive(sqlx::FromRow)]
	struct TypeStatRow {
		name: String,
		count: i64,
	}

	let type_stat_rows = sqlx::query_as::<_, TypeStatRow>(
		"SELECT pt.name, COUNT(p.id) as count
		 FROM project_types pt
		 LEFT JOIN projects p ON p.project_type_id = pt.id AND p.deleted_at IS NULL
		 GROUP BY pt.id, pt.name
		 ORDER BY pt.id ASC",
	)
	.fetch_all(&state.model.db)
	.await
	.unwrap_or_default();

	let categories_stats = vec![
		CategoryStat {
			name: "Residential",
			count: type_stat_rows.iter().find(|r| r.name == "Residential").map(|r| r.count).unwrap_or(0),
			icon: "home",
		},
		CategoryStat {
			name: "Commercial",
			count: type_stat_rows.iter().find(|r| r.name == "Commercial").map(|r| r.count).unwrap_or(0),
			icon: "briefcase",
		},
		CategoryStat {
			name: "Mixed Use",
			count: type_stat_rows.iter().find(|r| r.name == "Mixed Use").map(|r| r.count).unwrap_or(0),
			icon: "building",
		},
		CategoryStat {
			name: "Plotted",
			count: type_stat_rows.iter().find(|r| r.name == "Plotted").map(|r| r.count).unwrap_or(0),
			icon: "layers",
		},
	];

	let available_cities = sqlx::query_file_scalar!("queries/locations/suggest_cities.sql")
		.fetch_all(&state.model.db)
		.await
		.unwrap_or_default();

	let project_types_rows = sqlx::query_file!("queries/project_types/list_project_types.sql")
		.fetch_all(&state.model.db)
		.await
		.unwrap_or_default();

	let project_types = project_types_rows
		.into_iter()
		.map(|pt| PublicTypeOption { id: pt.id, name: pt.name })
		.collect();

	let tmpl = HomeTemplate {
		meta_title: "myplace | Discover Premium Real Estate Projects & Developments",
		meta_description: "Explore verified real estate development projects, master planned communities, and direct developer inventories.",
		canonical_url: &format!("{}/", CONFIG.public_site_origin),
		og_image: None,
		featured_projects,
		featured_owners,
		categories_stats,
		available_cities,
		project_types,
	};

	let html = tmpl.render().map_err(|e| {
		tracing::error!(error = ?e, "failed to render home page template");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	state.page_cache.insert(cache_key, &html).await;
	Ok(Html(html))
}

// ---------------------------------------------------------------------------
// 2. PROJECTS DIRECTORY
// ---------------------------------------------------------------------------
#[derive(Deserialize, Debug, Default)]
pub struct ProjectsFilterQuery {
	#[serde(default, deserialize_with = "empty_as_none")]
	pub type_id: Option<i32>,
	#[serde(default, deserialize_with = "empty_as_none")]
	pub city: Option<String>,
	#[serde(default, deserialize_with = "empty_as_none")]
	pub area: Option<String>,
	#[serde(default, deserialize_with = "empty_as_none")]
	pub bhk: Option<f64>,
	#[serde(default, deserialize_with = "empty_as_none")]
	pub q: Option<String>,
}

async fn projects_list(
	State(state): State<AppState>,
	HtmlQuery(filter): HtmlQuery<ProjectsFilterQuery>,
) -> Result<Html<String>, StatusCode> {
	let q_pattern = filter.q.as_deref().filter(|s| !s.trim().is_empty()).map(|s| format!("%{}%", s.trim()));
	let city_pattern = filter.city.as_deref().filter(|s| !s.trim().is_empty()).map(|s| format!("%{}%", s.trim()));
	let area_pattern = filter.area.as_deref().filter(|s| !s.trim().is_empty()).map(|s| format!("%{}%", s.trim()));

	struct DbRow {
		id: i32,
		name: String,
		slug: String,
		project_type_name: String,
		city: String,
		area: String,
		formatted_address: String,
		configurations_count: i64,
		possession_date: Option<time::Date>,
		owner_id: i32,
		owner_name: String,
		owner_slug: String,
		hero_thumb_key: Option<String>,
		min_price: Option<f64>,
		bhk_summary: Option<String>,
	}

	let rows = sqlx::query_file_as!(
		DbRow,
		"queries/projects/search_public_projects.sql",
		filter.type_id,
		city_pattern.as_deref(),
		area_pattern.as_deref(),
		q_pattern.as_deref(),
		filter.bhk,
	)
	.fetch_all(&state.model.db)
	.await
	.unwrap_or_default();

	let projects: Vec<PublicProjectCard> = rows
		.into_iter()
		.map(|p| {
			let hero_thumb_url = p.hero_thumb_key.as_ref().map(|k| CONFIG.asset_url(k));
			let min_price_formatted = p.min_price.map(format_price);
			PublicProjectCard {
				id: p.id,
				name: p.name,
				slug: p.slug,
				project_type_name: p.project_type_name,
				city: p.city,
				area: p.area,
				formatted_address: p.formatted_address,
				hero_thumb_url,
				configurations_count: p.configurations_count,
				possession_date: format_opt_date(p.possession_date),
				owner_id: p.owner_id,
				owner_name: p.owner_name,
				owner_slug: p.owner_slug,
				min_price_formatted,
				bhk_summary: p.bhk_summary,
			}
		})
		.collect();

	let available_cities = sqlx::query_file_scalar!("queries/locations/suggest_cities.sql")
		.fetch_all(&state.model.db)
		.await
		.unwrap_or_default();

	let city_for_areas = filter.city.as_deref().filter(|s| !s.trim().is_empty());
	let available_areas = sqlx::query_file_scalar!("queries/locations/suggest_areas.sql", city_for_areas)
		.fetch_all(&state.model.db)
		.await
		.unwrap_or_default();

	let project_types_rows = sqlx::query_file!("queries/project_types/list_project_types.sql")
		.fetch_all(&state.model.db)
		.await
		.unwrap_or_default();

	let project_types = project_types_rows
		.into_iter()
		.map(|pt| PublicTypeOption { id: pt.id, name: pt.name })
		.collect();

	let tmpl = ProjectsListTemplate {
		meta_title: "Real Estate Projects & Developments | myplace",
		meta_description: "Explore all verified real estate projects and residential developments.",
		canonical_url: &format!("{}/projects", CONFIG.public_site_origin),
		og_image: None,
		projects,
		project_types,
		selected_type_id: filter.type_id,
		selected_city: filter.city.as_deref(),
		selected_area: filter.area.as_deref(),
		selected_bhk: filter.bhk,
		search_query: filter.q.as_deref(),
		available_cities,
		available_areas,
	};

	let html = tmpl.render().map_err(|e| {
		tracing::error!(error = ?e, "failed to render projects list template");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	Ok(Html(html))
}

// ---------------------------------------------------------------------------
// 3. PROJECT DETAIL
// ---------------------------------------------------------------------------
async fn project_detail(
	Path(slug_or_id): Path<String>,
	State(state): State<AppState>,
	HtmlQuery(query_map): HtmlQuery<std::collections::HashMap<String, String>>,
) -> Result<Response, StatusCode> {
	struct DbProj {
		id: i32,
		project_owner_id: i32,
		name: String,
		slug: String,
		description: Option<String>,
		project_type_name: String,
		formatted_address: String,
		city: String,
		area: String,
		state_or_province: String,
		postal_code: Option<String>,
		latitude: Option<f64>,
		longitude: Option<f64>,
		rera_number: Option<String>,
		rera_authority: Option<String>,
		total_land_area: Option<f64>,
		towers_count: Option<i32>,
		floors_count: Option<i32>,
		units_count: Option<i32>,
		start_date: Option<time::Date>,
		launch_date: Option<time::Date>,
		possession_date: Option<time::Date>,
	}

	let parsed_id = if let Some((id_str, _)) = slug_or_id.split_once('-') {
		id_str.parse::<i32>().ok()
	} else {
		slug_or_id.parse::<i32>().ok()
	};

	let proj = match parsed_id {
		Some(id) => {
			let proj_opt =
				sqlx::query_file_as!(DbProj, "queries/projects/get_public_project_by_id.sql", id)
					.fetch_optional(&state.model.db)
					.await
					.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

			match proj_opt {
				Some(p) => {
					let canonical_param = format!("{}-{}", p.id, p.slug);
					if slug_or_id != canonical_param {
						return Ok(redirect_301(&format!("/projects/{canonical_param}")));
					}
					p
				}
				None => return Err(StatusCode::NOT_FOUND),
			}
		}
		None => {
			let proj_opt = sqlx::query_file_as!(
				DbProj,
				"queries/projects/get_public_project_by_slug.sql",
				&slug_or_id
			)
			.fetch_optional(&state.model.db)
			.await
			.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

			match proj_opt {
				Some(p) => {
					let canonical_param = format!("{}-{}", p.id, p.slug);
					return Ok(redirect_301(&format!("/projects/{canonical_param}")));
				}
				None => return Err(StatusCode::NOT_FOUND),
			}
		}
	};

	let canonical_key = format!("/projects/{}-{}", proj.id, proj.slug);
	if query_map.is_empty() {
		if let Some(cached_html) = state.page_cache.get(&canonical_key).await {
			return Ok(Html(cached_html).into_response());
		}
	}

	// Owner summary
	struct DbOwner {
		id: i32,
		name: String,
		slug: String,
		profile_img_thumb_key: Option<String>,
	}
	let owner_row = sqlx::query_file_as!(DbOwner, "queries/project_owners/get_public_owner_summary.sql", proj.project_owner_id)
		.fetch_one(&state.model.db)
		.await
		.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

	let owner = PublicOwnerSummary {
		id: owner_row.id,
		name: owner_row.name,
		slug: owner_row.slug,
		image_url: owner_row.profile_img_thumb_key.map(|k| CONFIG.asset_url(&k)),
	};

	// Media gallery
	struct DbMedia {
		media_type: String,
		s3_key: String,
		thumbnail_key: Option<String>,
		sequence: i16,
	}
	let media_rows = sqlx::query_file_as!(DbMedia, "queries/media/get_public_project_media.sql", proj.id)
		.fetch_all(&state.model.db)
		.await
		.unwrap_or_default();

	let gallery: Vec<PublicMediaItem> = media_rows
		.into_iter()
		.map(|m| {
			let url = CONFIG.asset_url(&m.s3_key);
			let thumbnail_url = m.thumbnail_key.as_ref().map(|k| CONFIG.asset_url(k)).unwrap_or_else(|| url.clone());
			PublicMediaItem {
				media_type: m.media_type,
				url,
				thumbnail_url,
				sequence: m.sequence,
			}
		})
		.collect();

	// Documents
	struct DbDoc {
		display_name: String,
		doc_type: String,
		s3_key: String,
		file_size: Option<i64>,
	}
	let doc_rows = sqlx::query_file_as!(DbDoc, "queries/media/get_public_project_documents.sql", proj.id)
		.fetch_all(&state.model.db)
		.await
		.unwrap_or_default();

	let documents: Vec<PublicDocumentItem> = doc_rows
		.into_iter()
		.map(|d| PublicDocumentItem {
			display_name: d.display_name,
			doc_type: d.doc_type,
			url: CONFIG.asset_url(&d.s3_key),
			file_size_formatted: format_bytes(d.file_size),
		})
		.collect();

	// Amenities
	#[derive(sqlx::FromRow)]
	struct AmenityRow {
		name: String,
	}
	let amenity_rows = sqlx::query_as::<_, AmenityRow>(
		"SELECT a.name FROM amenities a
		 JOIN project_amenities pa ON pa.amenity_id = a.id
		 WHERE pa.project_id = $1 AND a.is_active = TRUE ORDER BY a.name ASC",
	)
	.bind(proj.id)
	.fetch_all(&state.model.db)
	.await
	.unwrap_or_default();

	let amenities = amenity_rows.into_iter().map(|a| a.name).collect();

	// Configurations
	let config_rows = sqlx::query_file!("queries/configurations/list_project_configurations.sql", proj.id)
		.fetch_all(&state.model.db)
		.await
		.unwrap_or_default();

	let configurations: Vec<PublicConfigurationItem> = config_rows
		.into_iter()
		.filter(|c| c.is_active)
		.map(|c| {
			let floor_plan_url = c.floor_plan_key.as_deref().map(|k| CONFIG.asset_url(k));
			let floor_plan_thumb_url = c.floor_plan_thumb_key.as_deref().map(|k| CONFIG.asset_url(k));
			let price_formatted = if c.price_on_request {
				"Price on Request".to_string()
			} else {
				c.price.map(format_price).unwrap_or_else(|| "Price on Request".to_string())
			};
			PublicConfigurationItem {
				id: c.id,
				name: c.name,
				bhk: c.bhk,
				carpet_area: c.carpet_area,
				built_up_area: c.built_up_area,
				super_built_up_area: c.super_built_up_area,
				price: c.price,
				price_formatted,
				price_on_request: c.price_on_request,
				floor_plan_url,
				floor_plan_thumb_url,
			}
		})
		.collect();

	// Project Floor Plans
	let floor_plan_rows = sqlx::query_file!("queries/floor_plans/get_project_floor_plans.sql", proj.id)
		.fetch_all(&state.model.db)
		.await
		.unwrap_or_default();

	let floor_plans: Vec<PublicFloorPlanItem> = floor_plan_rows
		.into_iter()
		.map(|fp| {
			let url = CONFIG.asset_url(&fp.s3_key);
			let thumbnail_url = fp.thumbnail_key.as_ref().map(|k| CONFIG.asset_url(k)).unwrap_or_else(|| url.clone());
			PublicFloorPlanItem {
				id: fp.id,
				title: fp.title,
				url,
				thumbnail_url,
			}
		})
		.collect();

	// Specifications
	let spec_rows = sqlx::query_file!("queries/specifications/get_project_specifications.sql", proj.id)
		.fetch_all(&state.model.db)
		.await
		.unwrap_or_default();

	let specifications: Vec<PublicSpecificationItem> = spec_rows
		.into_iter()
		.map(|s| PublicSpecificationItem {
			category: s.category,
			items: s.items,
		})
		.collect();

	// Landmarks
	let landmark_rows = sqlx::query_file!("queries/landmarks/get_project_landmarks.sql", proj.id)
		.fetch_all(&state.model.db)
		.await
		.unwrap_or_default();

	let landmarks: Vec<PublicLandmarkItem> = landmark_rows
		.into_iter()
		.map(|l| PublicLandmarkItem {
			category: l.category,
			name: l.name,
			distance: l.distance,
			duration: l.duration,
		})
		.collect();

	let description_html = render_markdown(proj.description.as_deref().unwrap_or(""));

	let enquiry_success = query_map.get("enquiry").map(|s| s.as_str()) == Some("success");
	let enquiry_error = query_map.get("enquiry").and_then(|s| {
		if s == "error" {
			Some("Please check your input and try again.".to_string())
		} else {
			None
		}
	});

	let og_image = gallery.first().map(|m| m.url.clone());

	let project_detail_data = PublicProjectDetail {
		id: proj.id,
		name: proj.name.clone(),
		slug: proj.slug.clone(),
		project_type_name: proj.project_type_name,
		formatted_address: proj.formatted_address,
		city: proj.city,
		area: proj.area,
		state_or_province: proj.state_or_province,
		postal_code: proj.postal_code,
		latitude: proj.latitude,
		longitude: proj.longitude,
		rera_number: proj.rera_number,
		rera_authority: proj.rera_authority,
		total_land_area: proj.total_land_area,
		towers_count: proj.towers_count,
		floors_count: proj.floors_count,
		units_count: proj.units_count,
		start_date: format_opt_date(proj.start_date),
		launch_date: format_opt_date(proj.launch_date),
		possession_date: format_opt_date(proj.possession_date),
	};

	let tmpl = ProjectDetailTemplate {
		meta_title: &format!("{} | Real Estate Project", proj.name),
		meta_description: &format!("Explore {} located in {}. View floor plans, pricing, amenities, and RERA registration.", proj.name, project_detail_data.city),
		canonical_url: &format!("{}/projects/{}-{}", CONFIG.public_site_origin, proj.id, proj.slug),
		og_image,
		project: project_detail_data,
		owner,
		description_html,
		gallery,
		documents,
		configurations,
		floor_plans,
		specifications,
		landmarks,
		amenities,
		enquiry_success,
		enquiry_error,
	};

	let html = tmpl.render().map_err(|e| {
		tracing::error!(error = ?e, "failed to render project detail template");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	if query_map.is_empty() {
		state.page_cache.insert(canonical_key, html.clone()).await;
	}

	Ok(Html(html).into_response())
}

// ---------------------------------------------------------------------------
// 4. PROJECT ENQUIRY (LEAD SUBMISSION)
// ---------------------------------------------------------------------------
#[derive(Deserialize)]
pub struct EnquiryForm {
	name: String,
	phone: String,
	email: Option<String>,
	message: Option<String>,
	configuration_id: Option<i32>,
}

async fn project_enquire(
	Path(id): Path<i32>,
	State(state): State<AppState>,
	Form(form): Form<EnquiryForm>,
) -> Result<Redirect, StatusCode> {
	let p = sqlx::query_file!("queries/projects/get_project_slug_by_id.sql", id)
		.fetch_optional(&state.model.db)
		.await
		.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
		.ok_or(StatusCode::NOT_FOUND)?;

	let name = form.name.trim();
	let phone = form.phone.trim();
	if name.is_empty() || phone.is_empty() {
		return Ok(Redirect::to(&format!(
			"/projects/{}-{}?enquiry=error",
			id, p.slug
		)));
	}

	let email = form.email.as_deref().map(str::trim).filter(|e| !e.is_empty());
	let message = form.message.as_deref().map(str::trim).filter(|m| !m.is_empty());

	sqlx::query_file!(
		"queries/projects/insert_project_enquiry.sql",
		id,
		name,
		phone,
		email,
		message,
		form.configuration_id,
	)
	.execute(&state.model.db)
	.await
	.map_err(|e| {
		tracing::error!(error = ?e, "db error saving project enquiry");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	Ok(Redirect::to(&format!(
		"/projects/{}-{}?enquiry=success",
		id, p.slug
	)))
}

// ---------------------------------------------------------------------------
// 5. DEVELOPER / OWNER DETAIL
// ---------------------------------------------------------------------------
async fn owner_detail(
	Path(slug_or_id): Path<String>,
	State(state): State<AppState>,
) -> Result<Response, StatusCode> {
	struct DbOwner {
		id: i32,
		name: String,
		slug: String,
		bio: Option<String>,
		website: Option<String>,
		email: Option<String>,
		phone: Option<String>,
		profile_img_key: Option<String>,
	}

	let parsed_id = if let Some((id_str, _)) = slug_or_id.split_once('-') {
		id_str.parse::<i32>().ok()
	} else {
		slug_or_id.parse::<i32>().ok()
	};

	let owner = match parsed_id {
		Some(id) => {
			let owner_opt = sqlx::query_file_as!(
				DbOwner,
				"queries/project_owners/get_public_owner_by_id.sql",
				id
			)
			.fetch_optional(&state.model.db)
			.await
			.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

			match owner_opt {
				Some(o) => {
					let canonical_param = format!("{}-{}", o.id, o.slug);
					if slug_or_id != canonical_param {
						return Ok(redirect_301(&format!("/owners/{canonical_param}")));
					}
					o
				}
				None => return Err(StatusCode::NOT_FOUND),
			}
		}
		None => {
			let owner_opt = sqlx::query_file_as!(
				DbOwner,
				"queries/project_owners/get_public_owner_by_slug.sql",
				&slug_or_id
			)
			.fetch_optional(&state.model.db)
			.await
			.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

			match owner_opt {
				Some(o) => {
					let canonical_param = format!("{}-{}", o.id, o.slug);
					return Ok(redirect_301(&format!("/owners/{canonical_param}")));
				}
				None => return Err(StatusCode::NOT_FOUND),
			}
		}
	};

	let canonical_key = format!("/owners/{}-{}", owner.id, owner.slug);
	if let Some(cached_html) = state.page_cache.get(&canonical_key).await {
		return Ok(Html(cached_html).into_response());
	}

	struct DbProjRow {
		id: i32,
		name: String,
		slug: String,
		project_type_name: String,
		city: String,
		area: String,
		formatted_address: String,
		configurations_count: i64,
		possession_date: Option<time::Date>,
		hero_thumb_key: Option<String>,
		min_price: Option<f64>,
	}

	let proj_rows = sqlx::query_file_as!(DbProjRow, "queries/projects/get_public_owner_projects.sql", owner.id)
		.fetch_all(&state.model.db)
		.await
		.unwrap_or_default();

	let projects: Vec<PublicProjectCard> = proj_rows
		.into_iter()
		.map(|p| {
			let hero_thumb_url = p.hero_thumb_key.as_ref().map(|k| CONFIG.asset_url(k));
			let min_price_formatted = p.min_price.map(format_price);
			PublicProjectCard {
				id: p.id,
				name: p.name,
				slug: p.slug,
				project_type_name: p.project_type_name,
				city: p.city,
				area: p.area,
				formatted_address: p.formatted_address,
				hero_thumb_url,
				configurations_count: p.configurations_count,
				possession_date: format_opt_date(p.possession_date),
				owner_id: owner.id,
				owner_name: owner.name.clone(),
				owner_slug: owner.slug.clone(),
				min_price_formatted,
				bhk_summary: None,
			}
		})
		.collect();

	let bio_html = render_markdown(owner.bio.as_deref().unwrap_or(""));
	let image_url = owner.profile_img_key.map(|k| CONFIG.asset_url(&k));

	let owner_detail_data = PublicOwnerDetail {
		id: owner.id,
		name: owner.name.clone(),
		slug: owner.slug.clone(),
		phone: owner.phone,
		email: owner.email,
		website: owner.website,
		image_url: image_url.clone(),
	};

	let tmpl = OwnerDetailTemplate {
		meta_title: &format!("{} | Real Estate Developer", owner.name),
		meta_description: &format!("Explore residential and commercial developments by {}.", owner.name),
		canonical_url: &format!("{}/owners/{}-{}", CONFIG.public_site_origin, owner.id, owner.slug),
		og_image: image_url.as_deref(),
		owner: owner_detail_data,
		bio_html,
		projects,
	};

	let html = tmpl.render().map_err(|e| {
		tracing::error!(error = ?e, "failed to render owner detail template");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	state.page_cache.insert(canonical_key, html.clone()).await;

	Ok(Html(html).into_response())
}

// ---------------------------------------------------------------------------
// 6. SITEMAP & ROBOTS.TXT
// ---------------------------------------------------------------------------
async fn sitemap_index_xml(State(state): State<AppState>) -> Result<Response, StatusCode> {
	let cache_key = "/sitemap.xml";
	if let Some(cached) = state.page_cache.get(cache_key).await {
		return Ok((
			StatusCode::OK,
			[(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
			cached,
		).into_response());
	}

	let xml = crate::sitemap::generate_sitemap_index(&state.model.db, &CONFIG.public_site_origin, &CONFIG).await?;
	state.page_cache.insert(cache_key, &xml).await;

	Ok((
		StatusCode::OK,
		[(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
		xml,
	).into_response())
}

async fn sitemap_shard_xml(
	Path(shard): Path<String>,
	State(state): State<AppState>,
) -> Result<Response, StatusCode> {
	let cache_key = format!("/sitemaps/{}", shard);
	if let Some(cached) = state.page_cache.get(&cache_key).await {
		return Ok((
			StatusCode::OK,
			[(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
			cached,
		).into_response());
	}

	if shard == "pages.xml" {
		let xml = crate::sitemap::generate_pages_sitemap(&CONFIG.public_site_origin)?;
		state.page_cache.insert(&cache_key, &xml).await;
		return Ok((
			StatusCode::OK,
			[(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
			xml,
		).into_response());
	}

	let (entity, start_id, end_id) = crate::sitemap::parse_shard_param(&shard).ok_or(StatusCode::NOT_FOUND)?;
	let xml_opt = crate::sitemap::generate_shard(&state.model.db, &CONFIG.public_site_origin, entity, start_id, end_id).await?;
	let xml = xml_opt.ok_or(StatusCode::NOT_FOUND)?;

	state.page_cache.insert(&cache_key, &xml).await;
	Ok((
		StatusCode::OK,
		[(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
		xml,
	).into_response())
}

async fn robots_txt() -> Response {
	let sitemap_url = format!("{}/sitemap.xml", CONFIG.public_site_origin);
	let content = format!(
		"User-agent: *\nAllow: /\nDisallow: /admin/\nDisallow: /owner/\n\nSitemap: {}\n",
		sitemap_url
	);
	(
		StatusCode::OK,
		[(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
		content,
	).into_response()
}

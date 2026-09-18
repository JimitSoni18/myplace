use askama::Template;
use axum::{
	Form, Router,
	extract::{Path, Query, State},
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
		ProjectsListTemplate, PropertiesListTemplate, PropertyDetailTemplate, PublicDocumentItem,
		PublicMediaItem, PublicOwnerCard, PublicOwnerDetail, PublicOwnerSummary, PublicProjectCard,
		PublicProjectDetail, PublicProjectSummary, PublicPropertyCard, PublicPropertyDetail,
		SitemapTemplate, SitemapUrl,
	},
	utils::markdown::render_markdown,
};

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/", get(home_page))
		.route("/projects", get(projects_list))
		.route("/projects/{slug}", get(project_detail))
		.route("/projects/{id}/enquire", post(project_enquire))
		.route("/owners/{slug}", get(owner_detail))
		.route("/properties", get(properties_list))
		.route("/properties/{slug}", get(property_detail))
		.route("/properties/{id}/enquire", post(property_enquire))
		.route("/sitemap.xml", get(sitemap_xml))
		.route("/robots.txt", get(robots_txt))
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

fn format_price(
	price: Option<f64>,
	currency: &str,
	listing_type: &str,
	period: Option<&str>,
) -> String {
	match price {
		None => "Price on Request".to_string(),
		Some(p) if p <= 0.0 => "Price on Request".to_string(),
		Some(p) => {
			let curr_sym = if currency == "INR" { "₹" } else { currency };
			let base_str = if p >= 10_000_000.0 {
				format!("{} {:.2} Cr", curr_sym, p / 10_000_000.0)
			} else if p >= 100_000.0 {
				format!("{} {:.2} L", curr_sym, p / 100_000.0)
			} else {
				format!("{curr_sym} {:.0}", p)
			};

			if listing_type == "rent" || listing_type == "lease" {
				if let Some(per) = period {
					format!("{base_str} /{per}")
				} else {
					format!("{base_str} /mo")
				}
			} else {
				base_str
			}
		}
	}
}

// ---------------------------------------------------------------------------
// 1. HOME PAGE: GET /
// ---------------------------------------------------------------------------

async fn home_page(State(state): State<AppState>) -> Result<Html<String>, StatusCode> {
	let cache_key = "/";
	if let Some(cached_html) = state.page_cache.get(cache_key).await {
		return Ok(Html(cached_html));
	}

	// Fetch featured projects (up to 6)
	struct DbProjectRow {
		id: i32,
		name: String,
		slug: String,
		category: String,
		location: Option<String>,
		properties_count: i64,
		possession_date: Option<time::Date>,
		owner_name: String,
		owner_slug: String,
		hero_thumb_key: Option<String>,
	}

	let project_rows = sqlx::query_as!(
		DbProjectRow,
		r#"
		SELECT
			p.id, p.name, p.slug, p.category,
			loc.formatted_address as "location?",
			COUNT(DISTINCT prop.id) as "properties_count!: i64",
			p.possession_date,
			po.name as owner_name,
			po.slug as owner_slug,
			(
				SELECT m.thumbnail_key
				FROM project_media pm
				JOIN media m ON m.id = pm.media_id
				WHERE pm.project_id = p.id
				ORDER BY pm.sequence ASC
				LIMIT 1
			) as hero_thumb_key
		FROM projects p
			JOIN project_owners po ON po.id = p.project_owner_id AND po.active = TRUE AND po.deleted_at IS NULL
			LEFT JOIN locations loc ON loc.id = p.location_id
			LEFT JOIN properties prop ON prop.project_id = p.id AND prop.deleted_at IS NULL
		WHERE p.deleted_at IS NULL
		GROUP BY p.id, loc.formatted_address, po.name, po.slug
		ORDER BY p.id DESC
		LIMIT 6
		"#
	)
	.fetch_all(&state.model.db)
	.await
	.map_err(|e| {
		tracing::error!(error = ?e, "db error fetching featured projects");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	let featured_projects = project_rows
		.into_iter()
		.map(|r| {
			let hero_thumb_url = r.hero_thumb_key.as_ref().map(|k| CONFIG.asset_url(k));
			PublicProjectCard {
				id: r.id,
				name: r.name,
				slug: r.slug,
				category: r.category,
				location_name: r.location.unwrap_or_else(|| "Unspecified".to_string()),
				hero_thumb_url,
				property_count: r.properties_count,
				possession_date: format_opt_date(r.possession_date),
				owner_name: r.owner_name,
				owner_slug: r.owner_slug,
			}
		})
		.collect();

	// Fetch featured properties (up to 6)
	struct DbPropRow {
		id: i32,
		slug: String,
		unit_number: Option<String>,
		category: String,
		property_type_name: String,
		built_up_area: Option<f64>,
		bedroom_count: Option<f64>,
		bathroom_count: Option<i16>,
		price: Option<f64>,
		currency_code: Option<String>,
		listing_type: Option<String>,
		billing_period: Option<String>,
		project_name: String,
		project_slug: String,
		location: Option<String>,
		hero_thumb_key: Option<String>,
	}

	let prop_rows = sqlx::query_as!(
		DbPropRow,
		r#"
		SELECT
			p.id, p.slug, p.unit_number,
			pt.category, pt.name as property_type_name,
			p.built_up_area::float8 as "built_up_area?",
			rpd.bedroom_count::float8 as "bedroom_count?",
			rpd.bathroom_count as "bathroom_count?",
			pl.price::float8 as "price?",
			pl.currency_code as "currency_code?",
			pl.listing_type as "listing_type?",
			pl.billing_period as "billing_period?",
			proj.name as project_name,
			proj.slug as project_slug,
			loc.formatted_address as "location?",
			(
				SELECT m.thumbnail_key
				FROM property_media pm
				JOIN media m ON m.id = pm.media_id
				WHERE pm.property_id = p.id
				ORDER BY pm.sequence ASC
				LIMIT 1
			) as hero_thumb_key
		FROM properties p
			JOIN property_types pt ON pt.id = p.property_type_id
			JOIN projects proj ON proj.id = p.project_id AND proj.deleted_at IS NULL
			JOIN project_owners po ON po.id = proj.project_owner_id AND po.active = TRUE AND po.deleted_at IS NULL
			LEFT JOIN locations loc ON loc.id = proj.location_id
			LEFT JOIN residential_property_details rpd ON rpd.property_id = p.id
			LEFT JOIN property_listings pl ON pl.property_id = p.id AND pl.status = 'active'
		WHERE p.deleted_at IS NULL
		ORDER BY p.id DESC
		LIMIT 6
		"#
	)
	.fetch_all(&state.model.db)
	.await
	.map_err(|e| {
		tracing::error!(error = ?e, "db error fetching featured properties");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	let featured_properties = prop_rows
		.into_iter()
		.map(|r| {
			let hero_thumb_url = r.hero_thumb_key.as_ref().map(|k| CONFIG.asset_url(k));
			let currency = r.currency_code.unwrap_or_else(|| "INR".to_string());
			let l_type = r.listing_type.unwrap_or_else(|| "sale".to_string());
			let price_formatted = format_price(r.price, &currency, &l_type, r.billing_period.as_deref());

			let title = if let Some(bhk) = r.bedroom_count {
				format!("{bhk} BHK {} in {}", r.property_type_name, r.project_name)
			} else {
				format!("{} in {}", r.property_type_name, r.project_name)
			};

			PublicPropertyCard {
				id: r.id,
				slug: r.slug,
				title,
				category: r.category,
				property_type_name: r.property_type_name,
				price_formatted,
				listing_type: l_type,
				built_up_area: r.built_up_area,
				bedroom_count: r.bedroom_count,
				bathroom_count: r.bathroom_count,
				hero_thumb_url,
				project_name: r.project_name,
				project_slug: r.project_slug,
				location_name: r.location.unwrap_or_else(|| "Prime Location".to_string()),
			}
		})
		.collect();

	// Fetch top project owners
	struct DbOwnerRow {
		id: i32,
		name: String,
		slug: String,
		profile_img_thumb_key: Option<String>,
		project_count: i64,
	}

	let owner_rows = sqlx::query_as!(
		DbOwnerRow,
		r#"
		SELECT
			po.id, po.name, po.slug, po.profile_img_thumb_key,
			COUNT(p.id) as "project_count!: i64"
		FROM project_owners po
			LEFT JOIN projects p ON p.project_owner_id = po.id AND p.deleted_at IS NULL
		WHERE po.active = TRUE AND po.deleted_at IS NULL
		GROUP BY po.id
		ORDER BY COUNT(p.id) DESC, po.id DESC
		LIMIT 4
		"#
	)
	.fetch_all(&state.model.db)
	.await
	.map_err(|e| {
		tracing::error!(error = ?e, "db error fetching featured owners");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	let featured_owners = owner_rows
		.into_iter()
		.map(|r| {
			let image_url = r.profile_img_thumb_key.as_ref().map(|k| CONFIG.asset_url(k));
			PublicOwnerCard {
				id: r.id,
				name: r.name,
				slug: r.slug,
				image_url,
				project_count: r.project_count,
			}
		})
		.collect();

	// Category counts
	let res_count = sqlx::query!("SELECT COUNT(p.id) as \"c!: i64\" FROM properties p JOIN property_types pt ON pt.id = p.property_type_id WHERE pt.category = 'RESIDENTIAL' AND p.deleted_at IS NULL")
		.fetch_one(&state.model.db).await.map(|r| r.c).unwrap_or(0);
	let com_count = sqlx::query!("SELECT COUNT(p.id) as \"c!: i64\" FROM properties p JOIN property_types pt ON pt.id = p.property_type_id WHERE pt.category = 'COMMERCIAL' AND p.deleted_at IS NULL")
		.fetch_one(&state.model.db).await.map(|r| r.c).unwrap_or(0);
	let land_count = sqlx::query!("SELECT COUNT(p.id) as \"c!: i64\" FROM properties p JOIN property_types pt ON pt.id = p.property_type_id WHERE pt.category = 'LAND' AND p.deleted_at IS NULL")
		.fetch_one(&state.model.db).await.map(|r| r.c).unwrap_or(0);

	let categories_stats = vec![
		CategoryStat {
			name: "Residential Homes",
			slug: "RESIDENTIAL",
			count: res_count,
			icon: "🏡",
		},
		CategoryStat {
			name: "Commercial Spaces",
			slug: "COMMERCIAL",
			count: com_count,
			icon: "🏢",
		},
		CategoryStat {
			name: "Plots & Land",
			slug: "LAND",
			count: land_count,
			icon: "🌾",
		},
	];

	let canonical_url = format!("{}/", CONFIG.site_base_url.trim_end_matches('/'));
	let template = HomeTemplate {
		meta_title: "MyPlace — Premium Residential & Commercial Real Estate Platform",
		meta_description: "Explore verified real-estate projects, luxurious flats, villas, corporate offices, and investment plots from trusted developers.",
		canonical_url: &canonical_url,
		og_image: None,
		featured_projects,
		featured_properties,
		featured_owners,
		categories_stats,
	};

	let html = template.render().map_err(|e| {
		tracing::error!(error = ?e, "failed to render home template");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	state.page_cache.insert(cache_key.to_string(), html.clone()).await;
	Ok(Html(html))
}

// ---------------------------------------------------------------------------
// 2. PROJECTS LIST: GET /projects
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct ProjectsFilter {
	category: Option<String>,
	q: Option<String>,
}

async fn projects_list(
	Query(filter): Query<ProjectsFilter>,
	State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
	let is_unfiltered = filter.category.is_none() && filter.q.is_none();
	let cache_key = "/projects";
	if is_unfiltered {
		if let Some(cached_html) = state.page_cache.get(cache_key).await {
			return Ok(Html(cached_html));
		}
	}

	struct DbRow {
		id: i32,
		name: String,
		slug: String,
		category: String,
		location: Option<String>,
		properties_count: i64,
		possession_date: Option<time::Date>,
		owner_name: String,
		owner_slug: String,
		hero_thumb_key: Option<String>,
	}

	let search_pattern = filter.q.as_ref().map(|q| format!("%{q}%"));

	let rows = sqlx::query_as!(
		DbRow,
		r#"
		SELECT
			p.id, p.name, p.slug, p.category,
			loc.formatted_address as "location?",
			COUNT(DISTINCT prop.id) as "properties_count!: i64",
			p.possession_date,
			po.name as owner_name,
			po.slug as owner_slug,
			(
				SELECT m.thumbnail_key
				FROM project_media pm
				JOIN media m ON m.id = pm.media_id
				WHERE pm.project_id = p.id
				ORDER BY pm.sequence ASC
				LIMIT 1
			) as hero_thumb_key
		FROM projects p
			JOIN project_owners po ON po.id = p.project_owner_id AND po.active = TRUE AND po.deleted_at IS NULL
			LEFT JOIN locations loc ON loc.id = p.location_id
			LEFT JOIN properties prop ON prop.project_id = p.id AND prop.deleted_at IS NULL
		WHERE p.deleted_at IS NULL
		  AND ($1::text IS NULL OR p.category = $1)
		  AND ($2::text IS NULL OR p.name ILIKE $2 OR loc.formatted_address ILIKE $2)
		GROUP BY p.id, loc.formatted_address, po.name, po.slug
		ORDER BY p.id DESC
		"#,
		filter.category.as_deref(),
		search_pattern.as_deref()
	)
	.fetch_all(&state.model.db)
	.await
	.map_err(|e| {
		tracing::error!(error = ?e, "db error fetching projects list");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	let projects = rows
		.into_iter()
		.map(|r| {
			let hero_thumb_url = r.hero_thumb_key.as_ref().map(|k| CONFIG.asset_url(k));
			PublicProjectCard {
				id: r.id,
				name: r.name,
				slug: r.slug,
				category: r.category,
				location_name: r.location.unwrap_or_else(|| "Unspecified".to_string()),
				hero_thumb_url,
				property_count: r.properties_count,
				possession_date: format_opt_date(r.possession_date),
				owner_name: r.owner_name,
				owner_slug: r.owner_slug,
			}
		})
		.collect();

	let canonical_url = format!("{}/projects", CONFIG.site_base_url.trim_end_matches('/'));
	let template = ProjectsListTemplate {
		meta_title: "Real Estate Projects & Developments - MyPlace",
		meta_description: "Explore premier residential and commercial developments from verified real-estate builders.",
		canonical_url: &canonical_url,
		og_image: None,
		category_filter: filter.category.as_deref(),
		search_query: filter.q.as_deref(),
		projects,
	};

	let html = template.render().map_err(|e| {
		tracing::error!(error = ?e, "failed to render projects list template");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	if is_unfiltered {
		state.page_cache.insert(cache_key.to_string(), html.clone()).await;
	}

	Ok(Html(html))
}

// ---------------------------------------------------------------------------
// 3. PROJECT DETAIL: GET /projects/{slug}
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct EnquiryFeedback {
	enquiry: Option<String>,
}

async fn project_detail(
	Path(slug): Path<String>,
	Query(feedback): Query<EnquiryFeedback>,
	State(state): State<AppState>,
) -> Result<Response, StatusCode> {
	let cache_key = format!("/projects/{}", slug);
	if feedback.enquiry.is_none() {
		if let Some(cached_html) = state.page_cache.get(&cache_key).await {
			return Ok(Html(cached_html).into_response());
		}
	}

	struct DbProj {
		id: i32,
		project_owner_id: i32,
		name: String,
		slug: String,
		description: Option<String>,
		category: String,
		location_name: Option<String>,
		start_date: Option<time::Date>,
		launch_date: Option<time::Date>,
		possession_date: Option<time::Date>,
	}

	// 1. Try finding project by slug (case-insensitive)
	let proj_opt = sqlx::query_as!(
		DbProj,
		r#"
		SELECT
			p.id, p.project_owner_id, p.name, p.slug, p.description, p.category,
			loc.formatted_address as "location_name?",
			p.start_date, p.launch_date, p.possession_date
		FROM projects p
			LEFT JOIN locations loc ON loc.id = p.location_id
		WHERE LOWER(p.slug) = LOWER($1) AND p.deleted_at IS NULL
		"#,
		slug
	)
	.fetch_optional(&state.model.db)
	.await
	.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

	let proj = match proj_opt {
		Some(p) => {
			// Slug canonicalization check: if requested slug doesn't match canonical p.slug exactly:
			if p.slug != slug {
				return Ok(Redirect::permanent(&format!("/projects/{}", p.slug)).into_response());
			}
			p
		}
		None => {
			// Fallback: If slug is numeric ID or ends with numeric ID, check if project exists
			if let Ok(id) = slug.parse::<i32>() {
				let canonical = sqlx::query!("SELECT slug FROM projects WHERE id = $1 AND deleted_at IS NULL", id)
					.fetch_optional(&state.model.db)
					.await
					.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

				if let Some(can) = canonical {
					return Ok(Redirect::permanent(&format!("/projects/{}", can.slug)).into_response());
				}
			}
			return Err(StatusCode::NOT_FOUND);
		}
	};

	// 2. Fetch Developer Summary
	struct DbOwner {
		id: i32,
		name: String,
		slug: String,
		profile_img_thumb_key: Option<String>,
	}
	let owner_row = sqlx::query_as!(
		DbOwner,
		"SELECT id, name, slug, profile_img_thumb_key FROM project_owners WHERE id = $1",
		proj.project_owner_id
	)
	.fetch_one(&state.model.db)
	.await
	.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

	let owner_summary = PublicOwnerSummary {
		id: owner_row.id,
		name: owner_row.name,
		slug: owner_row.slug,
		image_url: owner_row.profile_img_thumb_key.as_ref().map(|k| CONFIG.asset_url(k)),
	};

	// 3. Fetch Media Gallery
	struct DbMedia {
		s3_key: String,
		thumbnail_key: Option<String>,
		sequence: i16,
	}
	let media_rows = sqlx::query_as!(
		DbMedia,
		r#"
		SELECT m.s3_key, m.thumbnail_key, pm.sequence
		FROM project_media pm
		JOIN media m ON m.id = pm.media_id
		WHERE pm.project_id = $1
		ORDER BY pm.sequence ASC
		"#,
		proj.id
	)
	.fetch_all(&state.model.db)
	.await
	.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

	let mut og_image = None;
	let gallery: Vec<PublicMediaItem> = media_rows
		.into_iter()
		.map(|r| {
			let url = CONFIG.asset_url(&r.s3_key);
			let thumbnail_url = r
				.thumbnail_key
				.as_ref()
				.map(|k| CONFIG.asset_url(k))
				.unwrap_or_else(|| url.clone());
			if og_image.is_none() {
				og_image = Some(url.clone());
			}
			PublicMediaItem {
				url,
				thumbnail_url,
				sequence: r.sequence,
			}
		})
		.collect();

	// 4. Fetch Documents
	struct DbDoc {
		display_name: String,
		doc_type: String,
		s3_key: String,
		file_size: Option<i64>,
	}
	let doc_rows = sqlx::query_as!(
		DbDoc,
		r#"
		SELECT pd.display_name, pd.doc_type, m.s3_key, m.file_size
		FROM project_documents pd
		JOIN media m ON m.id = pd.media_id
		WHERE pd.project_id = $1
		ORDER BY pd.id DESC
		"#,
		proj.id
	)
	.fetch_all(&state.model.db)
	.await
	.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

	let documents = doc_rows
		.into_iter()
		.map(|r| {
			let url = CONFIG.asset_url(&r.s3_key);
			let file_size_formatted = format_bytes(r.file_size);
			PublicDocumentItem {
				display_name: r.display_name,
				doc_type: r.doc_type,
				url,
				file_size_formatted,
			}
		})
		.collect();

	// 5. Fetch Project Amenities
	let amenity_rows = sqlx::query!(
		r#"
		SELECT a.name
		FROM project_amenities pa
		JOIN amenities a ON a.id = pa.amenity_id
		WHERE pa.project_id = $1 AND a.is_active = TRUE
		ORDER BY a.name ASC
		"#,
		proj.id
	)
	.fetch_all(&state.model.db)
	.await
	.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

	let amenities = amenity_rows.into_iter().map(|r| r.name).collect();

	// 6. Fetch Available Properties in this project
	struct DbPropRow {
		id: i32,
		slug: String,
		category: String,
		property_type_name: String,
		built_up_area: Option<f64>,
		bedroom_count: Option<f64>,
		bathroom_count: Option<i16>,
		price: Option<f64>,
		currency_code: Option<String>,
		listing_type: Option<String>,
		billing_period: Option<String>,
		hero_thumb_key: Option<String>,
	}

	let prop_rows = sqlx::query_as!(
		DbPropRow,
		r#"
		SELECT
			p.id, p.slug,
			pt.category, pt.name as property_type_name,
			p.built_up_area::float8 as "built_up_area?",
			rpd.bedroom_count::float8 as "bedroom_count?",
			rpd.bathroom_count as "bathroom_count?",
			pl.price::float8 as "price?",
			pl.currency_code as "currency_code?",
			pl.listing_type as "listing_type?",
			pl.billing_period as "billing_period?",
			(
				SELECT m.thumbnail_key
				FROM property_media pm
				JOIN media m ON m.id = pm.media_id
				WHERE pm.property_id = p.id
				ORDER BY pm.sequence ASC
				LIMIT 1
			) as hero_thumb_key
		FROM properties p
			JOIN property_types pt ON pt.id = p.property_type_id
			LEFT JOIN residential_property_details rpd ON rpd.property_id = p.id
			LEFT JOIN property_listings pl ON pl.property_id = p.id AND pl.status = 'active'
		WHERE p.project_id = $1 AND p.deleted_at IS NULL
		ORDER BY p.id DESC
		"#,
		proj.id
	)
	.fetch_all(&state.model.db)
	.await
	.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

	let properties = prop_rows
		.into_iter()
		.map(|r| {
			let hero_thumb_url = r.hero_thumb_key.as_ref().map(|k| CONFIG.asset_url(k));
			let currency = r.currency_code.unwrap_or_else(|| "INR".to_string());
			let l_type = r.listing_type.unwrap_or_else(|| "sale".to_string());
			let price_formatted = format_price(r.price, &currency, &l_type, r.billing_period.as_deref());

			let title = if let Some(bhk) = r.bedroom_count {
				format!("{bhk} BHK {}", r.property_type_name)
			} else {
				r.property_type_name.clone()
			};

			PublicPropertyCard {
				id: r.id,
				slug: r.slug,
				title,
				category: r.category,
				property_type_name: r.property_type_name,
				price_formatted,
				listing_type: l_type,
				built_up_area: r.built_up_area,
				bedroom_count: r.bedroom_count,
				bathroom_count: r.bathroom_count,
				hero_thumb_url,
				project_name: proj.name.clone(),
				project_slug: proj.slug.clone(),
				location_name: proj.location_name.clone().unwrap_or_default(),
			}
		})
		.collect();

	let description_html = proj
		.description
		.as_ref()
		.map(|desc| render_markdown(desc))
		.unwrap_or_default();

	let project_detail_data = PublicProjectDetail {
		id: proj.id,
		name: proj.name.clone(),
		slug: proj.slug.clone(),
		category: proj.category,
		location_name: proj.location_name.unwrap_or_else(|| "Prime Location".to_string()),
		start_date: format_opt_date(proj.start_date),
		launch_date: format_opt_date(proj.launch_date),
		possession_date: format_opt_date(proj.possession_date),
	};

	let canonical_url = format!("{}/projects/{}", CONFIG.site_base_url.trim_end_matches('/'), proj.slug);
	let meta_title = format!("{} — Verified Real Estate Project | MyPlace", proj.name);
	let meta_description = format!(
		"Explore {} by {}. View verified floor plans, amenities, available properties, and project approvals.",
		proj.name, owner_summary.name
	);

	let enquiry_success = feedback.enquiry.as_deref() == Some("success");
	let enquiry_error = if feedback.enquiry.as_deref() == Some("error") {
		Some("Please provide a valid name and phone number.".to_string())
	} else {
		None
	};

	let template = ProjectDetailTemplate {
		meta_title: &meta_title,
		meta_description: &meta_description,
		canonical_url: &canonical_url,
		og_image: og_image.as_deref(),
		project: project_detail_data,
		owner: owner_summary,
		description_html,
		gallery,
		documents,
		properties,
		amenities,
		enquiry_success,
		enquiry_error,
	};

	let html = template.render().map_err(|e| {
		tracing::error!(error = ?e, "failed to render project detail template");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	if feedback.enquiry.is_none() {
		state.page_cache.insert(cache_key, html.clone()).await;
	}

	Ok(Html(html).into_response())
}

// ---------------------------------------------------------------------------
// 4. PROJECT ENQUIRY SUBMISSION: POST /projects/{id}/enquire
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct EnquiryForm {
	name: String,
	phone: String,
	email: Option<String>,
	message: Option<String>,
}

async fn project_enquire(
	Path(id): Path<i32>,
	State(state): State<AppState>,
	Form(form): Form<EnquiryForm>,
) -> Result<Redirect, StatusCode> {
	let p = sqlx::query!("SELECT slug FROM projects WHERE id = $1 AND deleted_at IS NULL", id)
		.fetch_optional(&state.model.db)
		.await
		.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
		.ok_or(StatusCode::NOT_FOUND)?;

	let name = form.name.trim();
	let phone = form.phone.trim();
	if name.is_empty() || phone.is_empty() {
		return Ok(Redirect::to(&format!("/projects/{}?enquiry=error", p.slug)));
	}

	let email = form.email.as_deref().map(str::trim).filter(|e| !e.is_empty());
	let message = form.message.as_deref().map(str::trim).filter(|m| !m.is_empty());

	sqlx::query!(
		r#"
		INSERT INTO project_enquiries (project_id, name, phone, email, message)
		VALUES ($1, $2, $3, $4, $5)
		"#,
		id,
		name,
		phone,
		email,
		message
	)
	.execute(&state.model.db)
	.await
	.map_err(|e| {
		tracing::error!(error = ?e, "db error saving project enquiry");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	Ok(Redirect::to(&format!("/projects/{}?enquiry=success", p.slug)))
}

// ---------------------------------------------------------------------------
// 5. OWNER SHOWCASE: GET /owners/{slug}
// ---------------------------------------------------------------------------

async fn owner_detail(
	Path(slug): Path<String>,
	State(state): State<AppState>,
) -> Result<Response, StatusCode> {
	let cache_key = format!("/owners/{}", slug);
	if let Some(cached_html) = state.page_cache.get(&cache_key).await {
		return Ok(Html(cached_html).into_response());
	}

	struct DbOwner {
		id: i32,
		name: String,
		slug: String,
		phone: Option<String>,
		email: Option<String>,
		website: Option<String>,
		profile_img_key: Option<String>,
		bio: Option<String>,
	}

	let owner_opt = sqlx::query_as!(
		DbOwner,
		r#"
		SELECT id, name, slug, phone, email, website, profile_img_key, bio
		FROM project_owners
		WHERE LOWER(slug) = LOWER($1) AND active = TRUE AND deleted_at IS NULL
		"#,
		slug
	)
	.fetch_optional(&state.model.db)
	.await
	.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

	let owner = match owner_opt {
		Some(o) => {
			if o.slug != slug {
				return Ok(Redirect::permanent(&format!("/owners/{}", o.slug)).into_response());
			}
			o
		}
		None => {
			if let Ok(id) = slug.parse::<i32>() {
				let canonical = sqlx::query!(
					"SELECT slug FROM project_owners WHERE id = $1 AND active = TRUE AND deleted_at IS NULL",
					id
				)
				.fetch_optional(&state.model.db)
				.await
				.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

				if let Some(can) = canonical {
					return Ok(Redirect::permanent(&format!("/owners/{}", can.slug)).into_response());
				}
			}
			return Err(StatusCode::NOT_FOUND);
		}
	};

	// Fetch projects by this developer
	struct DbProjRow {
		id: i32,
		name: String,
		slug: String,
		category: String,
		location: Option<String>,
		properties_count: i64,
		possession_date: Option<time::Date>,
		hero_thumb_key: Option<String>,
	}

	let proj_rows = sqlx::query_as!(
		DbProjRow,
		r#"
		SELECT
			p.id, p.name, p.slug, p.category,
			loc.formatted_address as "location?",
			COUNT(DISTINCT prop.id) as "properties_count!: i64",
			p.possession_date,
			(
				SELECT m.thumbnail_key
				FROM project_media pm
				JOIN media m ON m.id = pm.media_id
				WHERE pm.project_id = p.id
				ORDER BY pm.sequence ASC
				LIMIT 1
			) as hero_thumb_key
		FROM projects p
			LEFT JOIN locations loc ON loc.id = p.location_id
			LEFT JOIN properties prop ON prop.project_id = p.id AND prop.deleted_at IS NULL
		WHERE p.project_owner_id = $1 AND p.deleted_at IS NULL
		GROUP BY p.id, loc.formatted_address
		ORDER BY p.id DESC
		"#,
		owner.id
	)
	.fetch_all(&state.model.db)
	.await
	.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

	let projects = proj_rows
		.into_iter()
		.map(|r| {
			let hero_thumb_url = r.hero_thumb_key.as_ref().map(|k| CONFIG.asset_url(k));
			PublicProjectCard {
				id: r.id,
				name: r.name,
				slug: r.slug,
				category: r.category,
				location_name: r.location.unwrap_or_else(|| "Unspecified".to_string()),
				hero_thumb_url,
				property_count: r.properties_count,
				possession_date: format_opt_date(r.possession_date),
				owner_name: owner.name.clone(),
				owner_slug: owner.slug.clone(),
			}
		})
		.collect();

	let bio_html = owner
		.bio
		.as_ref()
		.map(|b| render_markdown(b))
		.unwrap_or_default();

	let image_url = owner.profile_img_key.as_ref().map(|k| CONFIG.asset_url(k));
	let owner_detail_data = PublicOwnerDetail {
		id: owner.id,
		name: owner.name.clone(),
		slug: owner.slug.clone(),
		phone: owner.phone,
		email: owner.email,
		website: owner.website,
		image_url: image_url.clone(),
	};

	let canonical_url = format!("{}/owners/{}", CONFIG.site_base_url.trim_end_matches('/'), owner.slug);
	let meta_title = format!("{} — Real Estate Developer Portfolio | MyPlace", owner.name);
	let meta_description = format!(
		"View portfolio, residential developments, and commercial projects delivered by {} on MyPlace.",
		owner.name
	);

	let template = OwnerDetailTemplate {
		meta_title: &meta_title,
		meta_description: &meta_description,
		canonical_url: &canonical_url,
		og_image: image_url.as_deref(),
		owner: owner_detail_data,
		bio_html,
		projects,
	};

	let html = template.render().map_err(|e| {
		tracing::error!(error = ?e, "failed to render owner detail template");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	state.page_cache.insert(cache_key, html.clone()).await;
	Ok(Html(html).into_response())
}

// ---------------------------------------------------------------------------
// 6. PROPERTIES LIST: GET /properties
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct PropertiesFilter {
	category: Option<String>,
	listing_type: Option<String>,
	q: Option<String>,
}

async fn properties_list(
	Query(filter): Query<PropertiesFilter>,
	State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
	let is_unfiltered = filter.category.is_none() && filter.listing_type.is_none() && filter.q.is_none();
	let cache_key = "/properties";
	if is_unfiltered {
		if let Some(cached_html) = state.page_cache.get(cache_key).await {
			return Ok(Html(cached_html));
		}
	}

	struct DbPropRow {
		id: i32,
		slug: String,
		category: String,
		property_type_name: String,
		built_up_area: Option<f64>,
		bedroom_count: Option<f64>,
		bathroom_count: Option<i16>,
		price: Option<f64>,
		currency_code: Option<String>,
		listing_type: Option<String>,
		billing_period: Option<String>,
		project_name: String,
		project_slug: String,
		location: Option<String>,
		hero_thumb_key: Option<String>,
	}

	let search_pattern = filter.q.as_ref().map(|q| format!("%{q}%"));

	let rows = sqlx::query_as!(
		DbPropRow,
		r#"
		SELECT
			p.id, p.slug,
			pt.category, pt.name as property_type_name,
			p.built_up_area::float8 as "built_up_area?",
			rpd.bedroom_count::float8 as "bedroom_count?",
			rpd.bathroom_count as "bathroom_count?",
			pl.price::float8 as "price?",
			pl.currency_code as "currency_code?",
			pl.listing_type as "listing_type?",
			pl.billing_period as "billing_period?",
			proj.name as project_name,
			proj.slug as project_slug,
			loc.formatted_address as "location?",
			(
				SELECT m.thumbnail_key
				FROM property_media pm
				JOIN media m ON m.id = pm.media_id
				WHERE pm.property_id = p.id
				ORDER BY pm.sequence ASC
				LIMIT 1
			) as hero_thumb_key
		FROM properties p
			JOIN property_types pt ON pt.id = p.property_type_id
			JOIN projects proj ON proj.id = p.project_id AND proj.deleted_at IS NULL
			JOIN project_owners po ON po.id = proj.project_owner_id AND po.active = TRUE AND po.deleted_at IS NULL
			LEFT JOIN locations loc ON loc.id = proj.location_id
			LEFT JOIN residential_property_details rpd ON rpd.property_id = p.id
			LEFT JOIN property_listings pl ON pl.property_id = p.id AND pl.status = 'active'
		WHERE p.deleted_at IS NULL
		  AND ($1::text IS NULL OR pt.category = $1)
		  AND ($2::text IS NULL OR pl.listing_type = $2)
		  AND ($3::text IS NULL OR proj.name ILIKE $3 OR loc.formatted_address ILIKE $3 OR p.unit_number ILIKE $3)
		ORDER BY p.id DESC
		"#,
		filter.category.as_deref(),
		filter.listing_type.as_deref(),
		search_pattern.as_deref()
	)
	.fetch_all(&state.model.db)
	.await
	.map_err(|e| {
		tracing::error!(error = ?e, "db error fetching properties list");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	let properties = rows
		.into_iter()
		.map(|r| {
			let hero_thumb_url = r.hero_thumb_key.as_ref().map(|k| CONFIG.asset_url(k));
			let currency = r.currency_code.unwrap_or_else(|| "INR".to_string());
			let l_type = r.listing_type.unwrap_or_else(|| "sale".to_string());
			let price_formatted = format_price(r.price, &currency, &l_type, r.billing_period.as_deref());

			let title = if let Some(bhk) = r.bedroom_count {
				format!("{bhk} BHK {} in {}", r.property_type_name, r.project_name)
			} else {
				format!("{} in {}", r.property_type_name, r.project_name)
			};

			PublicPropertyCard {
				id: r.id,
				slug: r.slug,
				title,
				category: r.category,
				property_type_name: r.property_type_name,
				price_formatted,
				listing_type: l_type,
				built_up_area: r.built_up_area,
				bedroom_count: r.bedroom_count,
				bathroom_count: r.bathroom_count,
				hero_thumb_url,
				project_name: r.project_name,
				project_slug: r.project_slug,
				location_name: r.location.unwrap_or_else(|| "Prime Location".to_string()),
			}
		})
		.collect();

	let canonical_url = format!("{}/properties", CONFIG.site_base_url.trim_end_matches('/'));
	let template = PropertiesListTemplate {
		meta_title: "Properties for Sale & Rent - MyPlace",
		meta_description: "Search verified apartments, penthouses, villas, commercial office spaces, and land plots.",
		canonical_url: &canonical_url,
		og_image: None,
		category_filter: filter.category.as_deref(),
		listing_type_filter: filter.listing_type.as_deref(),
		search_query: filter.q.as_deref(),
		properties,
	};

	let html = template.render().map_err(|e| {
		tracing::error!(error = ?e, "failed to render properties list template");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	if is_unfiltered {
		state.page_cache.insert(cache_key.to_string(), html.clone()).await;
	}

	Ok(Html(html))
}

// ---------------------------------------------------------------------------
// 7. PROPERTY DETAIL: GET /properties/{slug}
// ---------------------------------------------------------------------------

async fn property_detail(
	Path(slug): Path<String>,
	Query(feedback): Query<EnquiryFeedback>,
	State(state): State<AppState>,
) -> Result<Response, StatusCode> {
	let cache_key = format!("/properties/{}", slug);
	if feedback.enquiry.is_none() {
		if let Some(cached_html) = state.page_cache.get(&cache_key).await {
			return Ok(Html(cached_html).into_response());
		}
	}

	struct DbProp {
		id: i32,
		project_id: i32,
		slug: String,
		unit_number: Option<String>,
		building: Option<String>,
		floor_number: Option<i32>,
		total_floors: Option<i32>,
		built_up_area: Option<f64>,
		usable_area: Option<f64>,
		description: Option<String>,
		category: String,
		property_type_name: String,
		price: Option<f64>,
		currency_code: Option<String>,
		listing_type: Option<String>,
		billing_period: Option<String>,
		bedroom_count: Option<f64>,
		bathroom_count: Option<i16>,
		balcony_count: Option<i16>,
		is_duplex: Option<bool>,
		res_parking: Option<String>,
		com_parking: Option<String>,
		parcel_number: Option<String>,
		zoning: Option<String>,
		approval_status: Option<String>,
		development_status: Option<String>,
	}

	let prop_opt = sqlx::query_as!(
		DbProp,
		r#"
		SELECT
			p.id, p.project_id, p.slug, p.unit_number, p.building,
			p.floor_number, p.total_floors,
			p.built_up_area::float8 as "built_up_area?",
			p.usable_area::float8 as "usable_area?",
			p.description,
			pt.category, pt.name as property_type_name,
			pl.price::float8 as "price?",
			pl.currency_code as "currency_code?",
			pl.listing_type as "listing_type?",
			pl.billing_period as "billing_period?",
			rpd.bedroom_count::float8 as "bedroom_count?",
			rpd.bathroom_count as "bathroom_count?",
			rpd.balcony_count as "balcony_count?",
			rpd.is_duplex as "is_duplex?",
			rpd.parking as "res_parking?",
			cpd.parking as "com_parking?",
			lpd.parcel_number as "parcel_number?",
			lpd.zoning as "zoning?",
			lpd.approval_status as "approval_status?",
			lpd.development_status as "development_status?"
		FROM properties p
			JOIN property_types pt ON pt.id = p.property_type_id
			JOIN projects proj ON proj.id = p.project_id AND proj.deleted_at IS NULL
			JOIN project_owners po ON po.id = proj.project_owner_id AND po.active = TRUE AND po.deleted_at IS NULL
			LEFT JOIN property_listings pl ON pl.property_id = p.id AND pl.status = 'active'
			LEFT JOIN residential_property_details rpd ON rpd.property_id = p.id
			LEFT JOIN commercial_property_details cpd ON cpd.property_id = p.id
			LEFT JOIN land_property_details lpd ON lpd.property_id = p.id
		WHERE LOWER(p.slug) = LOWER($1) AND p.deleted_at IS NULL
		"#,
		slug
	)
	.fetch_optional(&state.model.db)
	.await
	.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

	let prop = match prop_opt {
		Some(p) => {
			if p.slug != slug {
				return Ok(Redirect::permanent(&format!("/properties/{}", p.slug)).into_response());
			}
			p
		}
		None => {
			if let Ok(id) = slug.parse::<i32>() {
				let canonical = sqlx::query!("SELECT slug FROM properties WHERE id = $1 AND deleted_at IS NULL", id)
					.fetch_optional(&state.model.db)
					.await
					.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

				if let Some(can) = canonical {
					return Ok(Redirect::permanent(&format!("/properties/{}", can.slug)).into_response());
				}
			}
			return Err(StatusCode::NOT_FOUND);
		}
	};

	// Fetch parent project & developer
	struct DbProjOwner {
		id: i32,
		name: String,
		slug: String,
		category: String,
		location: Option<String>,
		owner_id: i32,
		owner_name: String,
		owner_slug: String,
		owner_image_key: Option<String>,
	}

	let po_row = sqlx::query_as!(
		DbProjOwner,
		r#"
		SELECT
			p.id, p.name, p.slug, p.category,
			loc.formatted_address as "location?",
			po.id as owner_id, po.name as owner_name, po.slug as owner_slug, po.profile_img_thumb_key as owner_image_key
		FROM projects p
			JOIN project_owners po ON po.id = p.project_owner_id
			LEFT JOIN locations loc ON loc.id = p.location_id
		WHERE p.id = $1
		"#,
		prop.project_id
	)
	.fetch_one(&state.model.db)
	.await
	.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

	let project_summary = PublicProjectSummary {
		id: po_row.id,
		name: po_row.name,
		slug: po_row.slug,
		category: po_row.category,
		location_name: po_row.location.unwrap_or_else(|| "Prime Location".to_string()),
	};

	let owner_summary = PublicOwnerSummary {
		id: po_row.owner_id,
		name: po_row.owner_name,
		slug: po_row.owner_slug,
		image_url: po_row.owner_image_key.as_ref().map(|k| CONFIG.asset_url(k)),
	};

	// Fetch Property Gallery
	struct DbMedia {
		s3_key: String,
		thumbnail_key: Option<String>,
		sequence: i16,
	}

	let media_rows = sqlx::query_as!(
		DbMedia,
		r#"
		SELECT m.s3_key, m.thumbnail_key, pm.sequence
		FROM property_media pm
		JOIN media m ON m.id = pm.media_id
		WHERE pm.property_id = $1
		ORDER BY pm.sequence ASC
		"#,
		prop.id
	)
	.fetch_all(&state.model.db)
	.await
	.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

	let mut og_image = None;
	let gallery: Vec<PublicMediaItem> = media_rows
		.into_iter()
		.map(|r| {
			let url = CONFIG.asset_url(&r.s3_key);
			let thumbnail_url = r
				.thumbnail_key
				.as_ref()
				.map(|k| CONFIG.asset_url(k))
				.unwrap_or_else(|| url.clone());
			if og_image.is_none() {
				og_image = Some(url.clone());
			}
			PublicMediaItem {
				url,
				thumbnail_url,
				sequence: r.sequence,
			}
		})
		.collect();

	// Fetch Amenities
	let amenity_rows = sqlx::query!(
		r#"
		SELECT a.name
		FROM property_amenities pa
		JOIN amenities a ON a.id = pa.amenity_id
		WHERE pa.property_id = $1 AND a.is_active = TRUE
		ORDER BY a.name ASC
		"#,
		prop.id
	)
	.fetch_all(&state.model.db)
	.await
	.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

	let amenities = amenity_rows.into_iter().map(|r| r.name).collect();

	let currency = prop.currency_code.unwrap_or_else(|| "INR".to_string());
	let l_type = prop.listing_type.unwrap_or_else(|| "sale".to_string());
	let price_formatted = format_price(prop.price, &currency, &l_type, prop.billing_period.as_deref());

	let parking = prop.res_parking.or(prop.com_parking);
	let property_detail_data = PublicPropertyDetail {
		id: prop.id,
		slug: prop.slug.clone(),
		unit_number: prop.unit_number,
		building: prop.building,
		floor_number: prop.floor_number,
		total_floors: prop.total_floors,
		built_up_area: prop.built_up_area,
		usable_area: prop.usable_area,
		category: prop.category,
		property_type_name: prop.property_type_name.clone(),
		price_formatted,
		listing_type: l_type,
		billing_period: prop.billing_period,
		bedroom_count: prop.bedroom_count,
		bathroom_count: prop.bathroom_count,
		balcony_count: prop.balcony_count,
		is_duplex: prop.is_duplex.unwrap_or(false),
		parking,
		parcel_number: prop.parcel_number,
		zoning: prop.zoning,
		approval_status: prop.approval_status,
		development_status: prop.development_status,
	};

	let description_html = prop
		.description
		.as_ref()
		.map(|desc| render_markdown(desc))
		.unwrap_or_default();

	let canonical_url = format!("{}/properties/{}", CONFIG.site_base_url.trim_end_matches('/'), prop.slug);
	let meta_title = format!(
		"{} in {} | MyPlace",
		property_detail_data.property_type_name, project_summary.name
	);
	let meta_description = format!(
		"Verified property listing for {} in {}. Price: {}. View amenities, floor plan specifications, and enquire directly.",
		property_detail_data.property_type_name, project_summary.name, property_detail_data.price_formatted
	);

	let enquiry_success = feedback.enquiry.as_deref() == Some("success");
	let enquiry_error = if feedback.enquiry.as_deref() == Some("error") {
		Some("Please provide a valid name and phone number.".to_string())
	} else {
		None
	};

	let template = PropertyDetailTemplate {
		meta_title: &meta_title,
		meta_description: &meta_description,
		canonical_url: &canonical_url,
		og_image: og_image.as_deref(),
		property: property_detail_data,
		project: project_summary,
		owner: owner_summary,
		description_html,
		gallery,
		amenities,
		enquiry_success,
		enquiry_error,
	};

	let html = template.render().map_err(|e| {
		tracing::error!(error = ?e, "failed to render property detail template");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	if feedback.enquiry.is_none() {
		state.page_cache.insert(cache_key, html.clone()).await;
	}

	Ok(Html(html).into_response())
}

// ---------------------------------------------------------------------------
// 8. PROPERTY ENQUIRY SUBMISSION: POST /properties/{id}/enquire
// ---------------------------------------------------------------------------

async fn property_enquire(
	Path(id): Path<i32>,
	State(state): State<AppState>,
	Form(form): Form<EnquiryForm>,
) -> Result<Redirect, StatusCode> {
	let prop = sqlx::query!("SELECT slug FROM properties WHERE id = $1 AND deleted_at IS NULL", id)
		.fetch_optional(&state.model.db)
		.await
		.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
		.ok_or(StatusCode::NOT_FOUND)?;

	let name = form.name.trim();
	let phone = form.phone.trim();
	if name.is_empty() || phone.is_empty() {
		return Ok(Redirect::to(&format!("/properties/{}?enquiry=error", prop.slug)));
	}

	let email = form.email.as_deref().map(str::trim).filter(|e| !e.is_empty());
	let message = form.message.as_deref().map(str::trim).filter(|m| !m.is_empty());

	sqlx::query!(
		r#"
		INSERT INTO property_enquiries (property_id, name, phone, email, message)
		VALUES ($1, $2, $3, $4, $5)
		"#,
		id,
		name,
		phone,
		email,
		message
	)
	.execute(&state.model.db)
	.await
	.map_err(|e| {
		tracing::error!(error = ?e, "db error saving property enquiry");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	Ok(Redirect::to(&format!("/properties/{}?enquiry=success", prop.slug)))
}

// ---------------------------------------------------------------------------
// 9. SITEMAP XML: GET /sitemap.xml
// ---------------------------------------------------------------------------

async fn sitemap_xml(State(state): State<AppState>) -> Result<Response, StatusCode> {
	let cache_key = "/sitemap.xml";
	if let Some(cached_xml) = state.page_cache.get(cache_key).await {
		return Ok(([(header::CONTENT_TYPE, "application/xml; charset=utf-8")], cached_xml).into_response());
	}

	let base = CONFIG.site_base_url.trim_end_matches('/');
	let mut urls = Vec::new();

	// Static routes
	urls.push(SitemapUrl {
		loc: format!("{base}/"),
		lastmod: None,
		changefreq: "daily",
		priority: "1.0",
	});
	urls.push(SitemapUrl {
		loc: format!("{base}/projects"),
		lastmod: None,
		changefreq: "daily",
		priority: "0.9",
	});
	urls.push(SitemapUrl {
		loc: format!("{base}/properties"),
		lastmod: None,
		changefreq: "hourly",
		priority: "0.9",
	});

	// Active Projects
	struct ProjSitemap {
		slug: String,
		updated_at: time::OffsetDateTime,
	}
	let projs = sqlx::query_as!(
		ProjSitemap,
		"SELECT slug, updated_at FROM projects WHERE deleted_at IS NULL ORDER BY id DESC"
	)
	.fetch_all(&state.model.db)
	.await
	.unwrap_or_default();

	let date_fmt = time::macros::format_description!("[year]-[month]-[day]");
	for p in projs {
		urls.push(SitemapUrl {
			loc: format!("{base}/projects/{}", p.slug),
			lastmod: p.updated_at.format(&date_fmt).ok(),
			changefreq: "weekly",
			priority: "0.8",
		});
	}

	// Active Owners
	struct OwnerSitemap {
		slug: String,
		updated_at: time::OffsetDateTime,
	}
	let owners = sqlx::query_as!(
		OwnerSitemap,
		"SELECT slug, updated_at FROM project_owners WHERE active = TRUE AND deleted_at IS NULL ORDER BY id DESC"
	)
	.fetch_all(&state.model.db)
	.await
	.unwrap_or_default();

	for o in owners {
		urls.push(SitemapUrl {
			loc: format!("{base}/owners/{}", o.slug),
			lastmod: o.updated_at.format(&date_fmt).ok(),
			changefreq: "weekly",
			priority: "0.7",
		});
	}

	// Active Properties
	struct PropSitemap {
		slug: String,
		updated_at: time::OffsetDateTime,
	}
	let props = sqlx::query_as!(
		PropSitemap,
		"SELECT slug, updated_at FROM properties WHERE deleted_at IS NULL ORDER BY id DESC"
	)
	.fetch_all(&state.model.db)
	.await
	.unwrap_or_default();

	for pr in props {
		urls.push(SitemapUrl {
			loc: format!("{base}/properties/{}", pr.slug),
			lastmod: pr.updated_at.format(&date_fmt).ok(),
			changefreq: "daily",
			priority: "0.8",
		});
	}

	let template = SitemapTemplate { urls };
	let xml = template.render().map_err(|e| {
		tracing::error!(error = ?e, "failed to render sitemap template");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	state.page_cache.insert(cache_key.to_string(), xml.clone()).await;
	Ok(([(header::CONTENT_TYPE, "application/xml; charset=utf-8")], xml).into_response())
}

// ---------------------------------------------------------------------------
// 10. ROBOTS TXT: GET /robots.txt
// ---------------------------------------------------------------------------

async fn robots_txt() -> Response {
	let body = format!(
		"User-agent: *\nAllow: /\nDisallow: /admin/\nDisallow: /owner/\nDisallow: /auth/\n\nSitemap: {}/sitemap.xml\n",
		CONFIG.site_base_url.trim_end_matches('/')
	);
	([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], body).into_response()
}

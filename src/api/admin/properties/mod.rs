use askama::Template;
use axum::{
	Extension, Form, Router,
	extract::{Multipart, Path, Query, State},
	http::StatusCode,
	response::{Html, Redirect},
	routing::get,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
	AppState,
	config::CONFIG,
	session_store::AuthUser,
	templates::admin::{
		AdminPage, AmenityOptionItem, ProjectMediaItem, ProjectOptionItem, PropertyAllListTemplate,
		PropertyEditData, PropertyFormTemplate, PropertyGlobalItem, PropertyListItem,
		PropertyListTemplate, PropertyMediaTemplate, PropertyTypeOption,
	},
	utils::sql::slugify,
};

pub mod error;
use error::PropertyError;

/// Router mounted at `/admin/properties`
pub fn router() -> Router<AppState> {
	Router::new().route("/", get(all_properties_list))
}

/// Project-scoped router mounted at `/admin/projects/{pid}/properties`
pub fn project_router() -> Router<AppState> {
	Router::new()
		.route("/residential", get(list_residential))
		.route("/commercial", get(list_commercial))
		.route("/land", get(list_land))
		.route(
			"/residential/new",
			get(create_residential_form).post(create_residential_property),
		)
		.route(
			"/commercial/new",
			get(create_commercial_form).post(create_commercial_property),
		)
		.route(
			"/land/new",
			get(create_land_form).post(create_land_property),
		)
		.route(
			"/{id}/edit",
			get(property_edit_form).post(update_property),
		)
		.route("/{id}", axum::routing::delete(delete_property))
		.route(
			"/{id}/media",
			get(property_media_page).post(upload_property_media),
		)
		.route(
			"/{id}/media/{mid}",
			axum::routing::delete(delete_property_media),
		)
}

// ---------------------------------------------------------------------------
// Payloads
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct PropertyAllQuery {
	project_id: Option<i32>,
	category: Option<String>,
	q: Option<String>,
}

#[derive(Deserialize)]
pub struct PropertyFormPayload {
	pub property_type_id: i32,
	pub unit_number: Option<String>,
	pub building: Option<String>,
	pub floor_number: Option<i32>,
	pub total_floors: Option<i32>,
	pub built_up_area: Option<f64>,
	pub usable_area: Option<f64>,
	pub description: Option<String>,
	pub listing_type: String,
	pub status: String,
	pub price: f64,
	pub billing_period: Option<String>,
	// residential
	pub bedroom_count: Option<f64>,
	pub bathroom_count: Option<i16>,
	pub balcony_count: Option<i16>,
	pub is_duplex: Option<String>,
	pub parking: Option<String>,
	// land
	pub parcel_number: Option<String>,
	pub zoning: Option<String>,
	pub approval_status: Option<String>,
	pub development_status: Option<String>,
	#[serde(default)]
	pub amenities: Vec<i32>,
}

fn format_inr(price: Option<f64>) -> String {
	match price {
		Some(p) => {
			if p >= 10_000_000.0 {
				format!("₹{:.2} Cr", p / 10_000_000.0)
			} else if p >= 100_000.0 {
				format!("₹{:.2} L", p / 100_000.0)
			} else {
				format!("₹{:.0}", p)
			}
		}
		None => "Price on Request".to_string(),
	}
}

// ---------------------------------------------------------------------------
// Global Overview Handler
// ---------------------------------------------------------------------------

async fn all_properties_list(
	State(state): State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
	Query(query): Query<PropertyAllQuery>,
) -> Result<Html<String>, PropertyError> {
	struct DbRow {
		id: i32,
		project_id: i32,
		project_name: String,
		unit_number: Option<String>,
		building: Option<String>,
		category: String,
		type_name: String,
		price: Option<f64>,
		status: Option<String>,
	}

	let projects_rows = sqlx::query!(
		"SELECT id, name FROM projects WHERE deleted_at IS NULL ORDER BY name ASC"
	)
	.fetch_all(&state.model.db)
	.await?;

	let projects = projects_rows
		.into_iter()
		.map(|r| ProjectOptionItem {
			id: r.id,
			name: r.name,
		})
		.collect();

	let rows = sqlx::query_as!(
		DbRow,
		r#"
		SELECT
			p.id, p.project_id, pr.name as project_name,
			p.unit_number, p.building,
			pt.category, pt.name as type_name,
			pl.price::float8 as "price?",
			pl.status as "status?"
		FROM properties p
			JOIN projects pr ON pr.id = p.project_id
			JOIN property_types pt ON pt.id = p.property_type_id
			LEFT JOIN property_listings pl ON pl.property_id = p.id
		WHERE p.deleted_at IS NULL
			AND ($1::int IS NULL OR p.project_id = $1)
			AND ($2::text IS NULL OR pt.category = $2)
			AND ($3::text IS NULL OR (p.unit_number ILIKE $3 OR p.building ILIKE $3))
		ORDER BY p.id DESC
		LIMIT 50
		"#,
		query.project_id,
		query.category.as_deref(),
		query.q.as_ref().map(|q| format!("%{}%", q.trim())),
	)
	.fetch_all(&state.model.db)
	.await?;

	let properties = rows
		.into_iter()
		.map(|r| {
			let unit_title = match (r.unit_number, r.building) {
				(Some(u), Some(b)) => format!("Unit {u}, {b}"),
				(Some(u), None) => format!("Unit {u}"),
				(None, Some(b)) => b,
				(None, None) => format!("Unit #{}", r.id),
			};
			PropertyGlobalItem {
				id: r.id,
				project_id: r.project_id,
				project_name: r.project_name,
				unit_title,
				category: r.category,
				type_name: r.type_name,
				price_formatted: format_inr(r.price),
				status: r.status.unwrap_or_else(|| "active".to_string()),
			}
		})
		.collect();

	Ok(Html(
		PropertyAllListTemplate {
			admin_name: &auth_user.username,
			page: AdminPage::Properties,
			properties,
			projects,
			selected_project_id: query.project_id,
			selected_category: query.category,
			q: query.q.as_deref(),
		}
		.render()?,
	))
}

// ---------------------------------------------------------------------------
// Project-Scoped Property Handlers
// ---------------------------------------------------------------------------

async fn list_properties_by_category(
	state: &AppState,
	auth_user: &AuthUser,
	project_id: i32,
	category_enum: &str,
	category_display: &str,
) -> Result<Html<String>, PropertyError> {
	let project = sqlx::query!(
		"SELECT id, name FROM projects WHERE id = $1 AND deleted_at IS NULL",
		project_id
	)
	.fetch_one(&state.model.db)
	.await?;

	struct DbRow {
		id: i32,
		unit_number: Option<String>,
		building: Option<String>,
		type_name: String,
		built_up_area: Option<f64>,
		listing_type: Option<String>,
		status: Option<String>,
		price: Option<f64>,
		media_count: i64,
		// Subtype fields
		bedroom_count: Option<f64>,
		bathroom_count: Option<i16>,
		parcel_number: Option<String>,
		zoning: Option<String>,
	}

	let rows = sqlx::query_as!(
		DbRow,
		r#"
		SELECT
			p.id, p.unit_number, p.building,
			pt.name as type_name,
			p.built_up_area::float8 as "built_up_area?",
			pl.listing_type as "listing_type?",
			pl.status as "status?",
			pl.price::float8 as "price?",
			COUNT(DISTINCT pm.media_id) as "media_count!: i64",
			rpd.bedroom_count::float8 as "bedroom_count?",
			rpd.bathroom_count as "bathroom_count?",
			lpd.parcel_number as "parcel_number?",
			lpd.zoning as "zoning?"
		FROM properties p
			JOIN property_types pt ON pt.id = p.property_type_id
			LEFT JOIN property_listings pl ON pl.property_id = p.id
			LEFT JOIN property_media pm ON pm.property_id = p.id
			LEFT JOIN residential_property_details rpd ON rpd.property_id = p.id
			LEFT JOIN land_property_details lpd ON lpd.property_id = p.id
		WHERE p.project_id = $1 AND pt.category = $2 AND p.deleted_at IS NULL
		GROUP BY p.id, pt.name, pl.listing_type, pl.status, pl.price,
		         rpd.bedroom_count, rpd.bathroom_count, lpd.parcel_number, lpd.zoning
		ORDER BY p.id DESC
		"#,
		project_id,
		category_enum,
	)
	.fetch_all(&state.model.db)
	.await?;

	let properties = rows
		.into_iter()
		.map(|r| {
			let details_summary = match category_enum {
				"RESIDENTIAL" => {
					let bhk = r
						.bedroom_count
						.map(|b| format!("{b} BHK"))
						.unwrap_or_default();
					let bath = r
						.bathroom_count
						.map(|b| format!("{b} Bath"))
						.unwrap_or_default();
					format!("{bhk} {bath}").trim().to_string()
				}
				"LAND" => {
					let parcel = r
						.parcel_number
						.map(|p| format!("Parcel {p}"))
						.unwrap_or_default();
					let zone = r.zoning.unwrap_or_default();
					format!("{parcel} ({zone})").trim().to_string()
				}
				_ => "Commercial Unit".to_string(),
			};

			PropertyListItem {
				id: r.id,
				unit_number: r.unit_number,
				building: r.building,
				type_name: r.type_name,
				area_sqft: r.built_up_area,
				listing_type: r.listing_type.unwrap_or_else(|| "sale".to_string()),
				status: r.status.unwrap_or_else(|| "active".to_string()),
				price_formatted: format_inr(r.price),
				media_count: r.media_count,
				details_summary,
			}
		})
		.collect();

	Ok(Html(
		PropertyListTemplate {
			admin_name: &auth_user.username,
			page: AdminPage::Properties,
			project_id: project.id,
			project_name: project.name,
			category: category_display.to_string(),
			properties,
		}
		.render()?,
	))
}

async fn list_residential(
	State(state): State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
	Path(pid): Path<i32>,
) -> Result<Html<String>, PropertyError> {
	list_properties_by_category(&state, &auth_user, pid, "RESIDENTIAL", "Residential").await
}

async fn list_commercial(
	State(state): State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
	Path(pid): Path<i32>,
) -> Result<Html<String>, PropertyError> {
	list_properties_by_category(&state, &auth_user, pid, "COMMERCIAL", "Commercial").await
}

async fn list_land(
	State(state): State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
	Path(pid): Path<i32>,
) -> Result<Html<String>, PropertyError> {
	list_properties_by_category(&state, &auth_user, pid, "LAND", "Land").await
}

// ---------------------------------------------------------------------------
// Create Forms & Handlers
// ---------------------------------------------------------------------------

async fn show_create_form(
	state: &AppState,
	auth_user: &AuthUser,
	project_id: i32,
	category_enum: &str,
	category_display: &str,
) -> Result<Html<String>, PropertyError> {
	let project = sqlx::query!(
		"SELECT id, name FROM projects WHERE id = $1 AND deleted_at IS NULL",
		project_id
	)
	.fetch_one(&state.model.db)
	.await?;

	let property_types = sqlx::query!(
		"SELECT id, name FROM property_types WHERE category = $1 AND is_active = TRUE ORDER BY name ASC",
		category_enum
	)
	.fetch_all(&state.model.db)
	.await?
	.into_iter()
	.map(|r| PropertyTypeOption {
		id: r.id,
		name: r.name,
	})
	.collect();

	let amenities = sqlx::query!(
		"SELECT id, name FROM amenities WHERE is_active = TRUE ORDER BY name ASC"
	)
	.fetch_all(&state.model.db)
	.await?
	.into_iter()
	.map(|r| AmenityOptionItem {
		id: r.id,
		name: r.name,
		selected: false,
	})
	.collect();

	Ok(Html(
		PropertyFormTemplate {
			admin_name: &auth_user.username,
			page: AdminPage::Properties,
			project_id: project.id,
			project_name: project.name,
			category: category_display.to_string(),
			property_types,
			amenities,
			edit_property: None,
		}
		.render()?,
	))
}

async fn create_residential_form(
	State(state): State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
	Path(pid): Path<i32>,
) -> Result<Html<String>, PropertyError> {
	show_create_form(&state, &auth_user, pid, "RESIDENTIAL", "Residential").await
}

async fn create_commercial_form(
	State(state): State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
	Path(pid): Path<i32>,
) -> Result<Html<String>, PropertyError> {
	show_create_form(&state, &auth_user, pid, "COMMERCIAL", "Commercial").await
}

async fn create_land_form(
	State(state): State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
	Path(pid): Path<i32>,
) -> Result<Html<String>, PropertyError> {
	show_create_form(&state, &auth_user, pid, "LAND", "Land").await
}

async fn insert_property(
	state: &AppState,
	project_id: i32,
	category_enum: &str,
	form: PropertyFormPayload,
) -> Result<Redirect, PropertyError> {
	let slug_raw = form
		.unit_number
		.as_deref()
		.unwrap_or_else(|| form.building.as_deref().unwrap_or("unit"));
	let slug = format!("{}-{}", slugify(slug_raw), Uuid::now_v7().simple());

	let mut tx = state.model.db.begin().await?;

	// 1. Insert unified property
	let row = sqlx::query!(
		r#"
		INSERT INTO properties (
			project_id, property_type_id, unit_number, building, floor_number,
			total_floors, built_up_area, usable_area, description, slug
		)
		VALUES ($1, $2, $3, $4, $5, $6, ($7::float8)::numeric, ($8::float8)::numeric, $9, $10)
		RETURNING id
		"#,
		project_id,
		form.property_type_id,
		form.unit_number.as_deref(),
		form.building.as_deref(),
		form.floor_number,
		form.total_floors,
		form.built_up_area,
		form.usable_area,
		form.description.as_deref(),
		slug,
	)
	.fetch_one(&mut *tx)
	.await?;

	let prop_id = row.id;

	// 2. Insert subtype specific details
	match category_enum {
		"RESIDENTIAL" => {
			let is_duplex = form.is_duplex.as_deref() == Some("on");
			sqlx::query!(
				r#"
				INSERT INTO residential_property_details (
					property_id, bedroom_count, bathroom_count, balcony_count, is_duplex, parking
				)
				VALUES ($1, ($2::float8)::numeric, $3, $4, $5, $6)
				"#,
				prop_id,
				form.bedroom_count,
				form.bathroom_count,
				form.balcony_count,
				is_duplex,
				form.parking.as_deref(),
			)
			.execute(&mut *tx)
			.await?;
		}
		"COMMERCIAL" => {
			sqlx::query!(
				r#"
				INSERT INTO commercial_property_details (property_id, parking)
				VALUES ($1, $2)
				"#,
				prop_id,
				form.parking.as_deref(),
			)
			.execute(&mut *tx)
			.await?;
		}
		"LAND" => {
			sqlx::query!(
				r#"
				INSERT INTO land_property_details (
					property_id, parcel_number, zoning, approval_status, development_status
				)
				VALUES ($1, $2, $3, $4, $5)
				"#,
				prop_id,
				form.parcel_number.as_deref(),
				form.zoning.as_deref(),
				form.approval_status.as_deref(),
				form.development_status.as_deref(),
			)
			.execute(&mut *tx)
			.await?;
		}
		_ => {}
	}

	// 3. Insert listing
	sqlx::query!(
		r#"
		INSERT INTO property_listings (
			property_id, listing_type, status, currency_code, price, billing_period
		)
		VALUES ($1, $2, $3, 'INR', ($4::float8)::numeric, $5)
		"#,
		prop_id,
		form.listing_type,
		form.status,
		form.price,
		form.billing_period.as_deref(),
	)
	.execute(&mut *tx)
	.await?;

	// 4. Insert amenities
	for amenity_id in form.amenities {
		sqlx::query!(
			"INSERT INTO property_amenities (property_id, amenity_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
			prop_id,
			amenity_id,
		)
		.execute(&mut *tx)
		.await?;
	}

	tx.commit().await?;

	let redirect_cat = category_enum.to_lowercase();
	tracing::info!(property_id = prop_id, project_id, "created property unit");
	state.page_cache.invalidate(crate::cache::InvalidationEvent::PropertyUpdated(prop_id)).await;
	Ok(Redirect::to(&format!(
		"/admin/projects/{project_id}/properties/{redirect_cat}"
	)))
}

async fn create_residential_property(
	State(state): State<AppState>,
	Path(pid): Path<i32>,
	Form(form): Form<PropertyFormPayload>,
) -> Result<Redirect, PropertyError> {
	insert_property(&state, pid, "RESIDENTIAL", form).await
}

async fn create_commercial_property(
	State(state): State<AppState>,
	Path(pid): Path<i32>,
	Form(form): Form<PropertyFormPayload>,
) -> Result<Redirect, PropertyError> {
	insert_property(&state, pid, "COMMERCIAL", form).await
}

async fn create_land_property(
	State(state): State<AppState>,
	Path(pid): Path<i32>,
	Form(form): Form<PropertyFormPayload>,
) -> Result<Redirect, PropertyError> {
	insert_property(&state, pid, "LAND", form).await
}

// ---------------------------------------------------------------------------
// Edit & Update
// ---------------------------------------------------------------------------

async fn property_edit_form(
	State(state): State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
	Path((pid, id)): Path<(i32, i32)>,
) -> Result<Html<String>, PropertyError> {
	let project = sqlx::query!(
		"SELECT id, name FROM projects WHERE id = $1 AND deleted_at IS NULL",
		pid
	)
	.fetch_one(&state.model.db)
	.await?;

	struct DbProp {
		id: i32,
		property_type_id: i32,
		category: String,
		unit_number: Option<String>,
		building: Option<String>,
		floor_number: Option<i32>,
		total_floors: Option<i32>,
		built_up_area: Option<f64>,
		usable_area: Option<f64>,
		description: Option<String>,
		listing_type: Option<String>,
		status: Option<String>,
		price: Option<f64>,
		billing_period: Option<String>,
		bedroom_count: Option<f64>,
		bathroom_count: Option<i16>,
		balcony_count: Option<i16>,
		is_duplex: Option<bool>,
		parking_res: Option<String>,
		parking_com: Option<String>,
		parcel_number: Option<String>,
		zoning: Option<String>,
		approval_status: Option<String>,
		development_status: Option<String>,
	}

	let r = sqlx::query_as!(
		DbProp,
		r#"
		SELECT
			p.id, p.property_type_id, pt.category,
			p.unit_number, p.building, p.floor_number, p.total_floors,
			p.built_up_area::float8 as "built_up_area?",
			p.usable_area::float8 as "usable_area?",
			p.description,
			pl.listing_type as "listing_type?",
			pl.status as "status?",
			pl.price::float8 as "price?",
			pl.billing_period,
			rpd.bedroom_count::float8 as "bedroom_count?",
			rpd.bathroom_count as "bathroom_count?",
			rpd.balcony_count as "balcony_count?",
			rpd.is_duplex as "is_duplex?",
			rpd.parking as "parking_res?",
			cpd.parking as "parking_com?",
			lpd.parcel_number as "parcel_number?",
			lpd.zoning as "zoning?",
			lpd.approval_status as "approval_status?",
			lpd.development_status as "development_status?"
		FROM properties p
			JOIN property_types pt ON pt.id = p.property_type_id
			LEFT JOIN property_listings pl ON pl.property_id = p.id
			LEFT JOIN residential_property_details rpd ON rpd.property_id = p.id
			LEFT JOIN commercial_property_details cpd ON cpd.property_id = p.id
			LEFT JOIN land_property_details lpd ON lpd.property_id = p.id
		WHERE p.id = $1 AND p.project_id = $2 AND p.deleted_at IS NULL
		"#,
		id,
		pid
	)
	.fetch_one(&state.model.db)
	.await?;

	let property_types = sqlx::query!(
		"SELECT id, name FROM property_types WHERE category = $1 AND is_active = TRUE ORDER BY name ASC",
		r.category
	)
	.fetch_all(&state.model.db)
	.await?
	.into_iter()
	.map(|t| PropertyTypeOption {
		id: t.id,
		name: t.name,
	})
	.collect();

	let linked_amenities: std::collections::HashSet<i32> = sqlx::query!(
		"SELECT amenity_id FROM property_amenities WHERE property_id = $1",
		id
	)
	.fetch_all(&state.model.db)
	.await?
	.into_iter()
	.map(|row| row.amenity_id)
	.collect();

	let amenities = sqlx::query!(
		"SELECT id, name FROM amenities WHERE is_active = TRUE ORDER BY name ASC"
	)
	.fetch_all(&state.model.db)
	.await?
	.into_iter()
	.map(|a| AmenityOptionItem {
		selected: linked_amenities.contains(&a.id),
		id: a.id,
		name: a.name,
	})
	.collect();

	let parking = r.parking_res.or(r.parking_com);

	let edit_property = PropertyEditData {
		id: r.id,
		property_type_id: r.property_type_id,
		unit_number: r.unit_number,
		building: r.building,
		floor_number: r.floor_number,
		total_floors: r.total_floors,
		built_up_area: r.built_up_area,
		usable_area: r.usable_area,
		description: r.description,
		listing_type: r.listing_type.unwrap_or_else(|| "sale".to_string()),
		status: r.status.unwrap_or_else(|| "active".to_string()),
		price: r.price,
		billing_period: r.billing_period,
		bedroom_count: r.bedroom_count,
		bathroom_count: r.bathroom_count,
		balcony_count: r.balcony_count,
		is_duplex: r.is_duplex.unwrap_or(false),
		parking,
		parcel_number: r.parcel_number,
		zoning: r.zoning,
		approval_status: r.approval_status,
		development_status: r.development_status,
	};

	let cat_display = match r.category.as_str() {
		"RESIDENTIAL" => "Residential",
		"COMMERCIAL" => "Commercial",
		"LAND" => "Land",
		_ => "Residential",
	};

	Ok(Html(
		PropertyFormTemplate {
			admin_name: &auth_user.username,
			page: AdminPage::Properties,
			project_id: project.id,
			project_name: project.name,
			category: cat_display.to_string(),
			property_types,
			amenities,
			edit_property: Some(edit_property),
		}
		.render()?,
	))
}

async fn update_property(
	State(state): State<AppState>,
	Path((pid, id)): Path<(i32, i32)>,
	Form(form): Form<PropertyFormPayload>,
) -> Result<Redirect, PropertyError> {
	let mut tx = state.model.db.begin().await?;

	sqlx::query!(
		r#"
		UPDATE properties
		SET property_type_id = $1, unit_number = $2, building = $3, floor_number = $4,
		    total_floors = $5, built_up_area = ($6::float8)::numeric, usable_area = ($7::float8)::numeric, description = $8,
		    updated_at = NOW()
		WHERE id = $9 AND project_id = $10 AND deleted_at IS NULL
		"#,
		form.property_type_id,
		form.unit_number.as_deref(),
		form.building.as_deref(),
		form.floor_number,
		form.total_floors,
		form.built_up_area,
		form.usable_area,
		form.description.as_deref(),
		id,
		pid,
	)
	.execute(&mut *tx)
	.await?;

	// Update listing
	sqlx::query!(
		r#"
		UPDATE property_listings
		SET listing_type = $1, status = $2, price = ($3::float8)::numeric, billing_period = $4, updated_at = NOW()
		WHERE property_id = $5
		"#,
		form.listing_type,
		form.status,
		form.price,
		form.billing_period.as_deref(),
		id,
	)
	.execute(&mut *tx)
	.await?;

	// Update subtype details
	let is_duplex = form.is_duplex.as_deref() == Some("on");
	let _ = sqlx::query!(
		r#"
		UPDATE residential_property_details
		SET bedroom_count = ($1::float8)::numeric, bathroom_count = $2, balcony_count = $3, is_duplex = $4, parking = $5
		WHERE property_id = $6
		"#,
		form.bedroom_count,
		form.bathroom_count,
		form.balcony_count,
		is_duplex,
		form.parking.as_deref(),
		id,
	)
	.execute(&mut *tx)
	.await;

	let _ = sqlx::query!(
		"UPDATE commercial_property_details SET parking = $1 WHERE property_id = $2",
		form.parking.as_deref(),
		id,
	)
	.execute(&mut *tx)
	.await;

	let _ = sqlx::query!(
		r#"
		UPDATE land_property_details
		SET parcel_number = $1, zoning = $2, approval_status = $3, development_status = $4
		WHERE property_id = $5
		"#,
		form.parcel_number.as_deref(),
		form.zoning.as_deref(),
		form.approval_status.as_deref(),
		form.development_status.as_deref(),
		id,
	)
	.execute(&mut *tx)
	.await;

	// Sync amenities
	sqlx::query!("DELETE FROM property_amenities WHERE property_id = $1", id)
		.execute(&mut *tx)
		.await?;

	for amenity_id in form.amenities {
		sqlx::query!(
			"INSERT INTO property_amenities (property_id, amenity_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
			id,
			amenity_id,
		)
		.execute(&mut *tx)
		.await?;
	}

	tx.commit().await?;

	tracing::info!(property_id = id, project_id = pid, "updated property");
	state.page_cache.invalidate(crate::cache::InvalidationEvent::PropertyUpdated(id)).await;
	Ok(Redirect::to(&format!(
		"/admin/projects/{pid}/properties/residential"
	)))
}

// ---------------------------------------------------------------------------
// Delete Property
// ---------------------------------------------------------------------------

async fn delete_property(
	State(state): State<AppState>,
	Path((pid, id)): Path<(i32, i32)>,
) -> Result<StatusCode, PropertyError> {
	// 1. Find all media linked to this property and clean up
	let media_rows = sqlx::query!(
		"SELECT media_id FROM property_media WHERE property_id = $1",
		id
	)
	.fetch_all(&state.model.db)
	.await?;

	sqlx::query!("DELETE FROM property_media WHERE property_id = $1", id)
		.execute(&state.model.db)
		.await?;

	for m in media_rows {
		let _ = crate::media::delete_media(&state.model.db, &state.s3_client, m.media_id).await;
	}

	// 2. Soft delete property
	sqlx::query!(
		"UPDATE properties SET deleted_at = NOW() WHERE id = $1 AND project_id = $2",
		id,
		pid
	)
	.execute(&state.model.db)
	.await?;

	tracing::info!(property_id = id, project_id = pid, "soft-deleted property");
	state.page_cache.invalidate(crate::cache::InvalidationEvent::PropertyUpdated(id)).await;
	Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Property Media Handlers
// ---------------------------------------------------------------------------

async fn property_media_page(
	State(state): State<AppState>,
	Extension(auth_user): Extension<AuthUser>,
	Path((pid, id)): Path<(i32, i32)>,
) -> Result<Html<String>, PropertyError> {
	let project = sqlx::query!(
		"SELECT id, name FROM projects WHERE id = $1 AND deleted_at IS NULL",
		pid
	)
	.fetch_one(&state.model.db)
	.await?;

	let prop = sqlx::query!(
		"SELECT id, unit_number, building FROM properties WHERE id = $1 AND project_id = $2 AND deleted_at IS NULL",
		id,
		pid
	)
	.fetch_one(&state.model.db)
	.await?;

	struct DbMedia {
		media_id: Uuid,
		s3_key: String,
		thumbnail_key: Option<String>,
		sequence: i16,
	}

	let media_rows = sqlx::query_as!(
		DbMedia,
		r#"
		SELECT pm.media_id, m.s3_key, m.thumbnail_key, pm.sequence
		FROM property_media pm
		JOIN media m ON m.id = pm.media_id
		WHERE pm.property_id = $1
		ORDER BY pm.sequence ASC
		"#,
		id
	)
	.fetch_all(&state.model.db)
	.await?;

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
				url,
				thumbnail_url,
				sequence: r.sequence,
			}
		})
		.collect();

	let unit_title = match (prop.unit_number, prop.building) {
		(Some(u), Some(b)) => format!("Unit {u}, {b}"),
		(Some(u), None) => format!("Unit {u}"),
		(None, Some(b)) => b,
		(None, None) => format!("Unit #{}", prop.id),
	};

	Ok(Html(
		PropertyMediaTemplate {
			admin_name: &auth_user.username,
			page: AdminPage::Properties,
			project_id: project.id,
			project_name: project.name,
			property_id: prop.id,
			unit_title,
			media,
		}
		.render()?,
	))
}

async fn upload_property_media(
	State(state): State<AppState>,
	Path((pid, id)): Path<(i32, i32)>,
	mut multipart: Multipart,
) -> Result<Redirect, PropertyError> {
	while let Ok(Some(field)) = multipart.next_field().await {
		if field.name() != Some("image") {
			continue;
		}

		let bytes = field
			.bytes()
			.await
			.map_err(|e| PropertyError::Internal(e.to_string()))?;

		let uploaded =
			crate::media::process_and_upload_image(&state.model.db, &state.s3_client, bytes.to_vec())
				.await?;

		let next_seq_row = sqlx::query!(
			"SELECT COALESCE(MAX(sequence), 0) + 1 as \"seq!: i16\" FROM property_media WHERE property_id = $1",
			id
		)
		.fetch_one(&state.model.db)
		.await?;

		sqlx::query!(
			"INSERT INTO property_media (property_id, media_id, sequence) VALUES ($1, $2, $3)",
			id,
			uploaded.media_id,
			next_seq_row.seq,
		)
		.execute(&state.model.db)
		.await?;

		tracing::info!(property_id = id, media_id = %uploaded.media_id, "attached media to property");
		state.page_cache.invalidate(crate::cache::InvalidationEvent::PropertyUpdated(id)).await;
		return Ok(Redirect::to(&format!(
			"/admin/projects/{pid}/properties/{id}/media"
		)));
	}

	Err(PropertyError::Validation("No image field provided in multipart form".to_string()))
}

async fn delete_property_media(
	State(state): State<AppState>,
	Path((_pid, id, mid)): Path<(i32, i32, Uuid)>,
) -> Result<StatusCode, PropertyError> {
	sqlx::query!(
		"DELETE FROM property_media WHERE property_id = $1 AND media_id = $2",
		id,
		mid
	)
	.execute(&state.model.db)
	.await?;

	crate::media::delete_media(&state.model.db, &state.s3_client, mid).await?;
	state.page_cache.invalidate(crate::cache::InvalidationEvent::PropertyUpdated(id)).await;

	Ok(StatusCode::NO_CONTENT)
}

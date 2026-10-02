use askama::Template;
use axum::{
	Extension, Router,
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
	utils::{form::Form, sql::slugify},
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
		.route("/{id}/edit", get(property_edit_form).post(update_property))
		.route("/{id}", axum::routing::delete(delete_property))
		.route(
			"/{id}/media",
			get(property_media_page).post(upload_property_media),
		)
		.route(
			"/{id}/media/reorder",
			axum::routing::post(reorder_property_media),
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
	pub bathroom_count: Option<f64>,
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

fn validate_property_payload(form: &PropertyFormPayload) -> Result<(), PropertyError> {
	if form.property_type_id <= 0 {
		return Err(PropertyError::Validation(
			"Valid property type is required".to_string(),
		));
	}
	if !form.price.is_finite() || form.price < 0.0 {
		return Err(PropertyError::Validation(
			"Price must be a non-negative number".to_string(),
		));
	}
	if form.price > 100_000_000_000.0 {
		return Err(PropertyError::Validation(
			"Price exceeds maximum limit".to_string(),
		));
	}
	if !["sale", "rent", "lease"].contains(&form.listing_type.as_str()) {
		return Err(PropertyError::Validation(
			"Invalid listing type".to_string(),
		));
	}
	if !["active", "closed", "sold", "withdrawn"].contains(&form.status.as_str()) {
		return Err(PropertyError::Validation("Invalid status".to_string()));
	}
	if let Some(built) = form.built_up_area {
		if !built.is_finite() || built < 0.0 || built > 10_000_000.0 {
			return Err(PropertyError::Validation(
				"Built up area must be between 0 and 10,000,000 sq ft".to_string(),
			));
		}
	}
	if let Some(usable) = form.usable_area {
		if !usable.is_finite() || usable < 0.0 || usable > 10_000_000.0 {
			return Err(PropertyError::Validation(
				"Usable area must be between 0 and 10,000,000 sq ft".to_string(),
			));
		}
	}
	if let Some(floor) = form.floor_number {
		if floor < -50 || floor > 500 {
			return Err(PropertyError::Validation(
				"Floor number must be between -50 and 500".to_string(),
			));
		}
	}
	if let Some(total) = form.total_floors {
		if total < 0 || total > 500 {
			return Err(PropertyError::Validation(
				"Total floors must be between 0 and 500".to_string(),
			));
		}
	}
	if let Some(beds) = form.bedroom_count {
		if !beds.is_finite() || beds < 0.0 || beds > 999.5 {
			return Err(PropertyError::Validation(
				"Bedroom count must be between 0 and 999.5".to_string(),
			));
		}
	}
	if let Some(baths) = form.bathroom_count {
		if !baths.is_finite() || baths < 0.0 || baths > 999.5 {
			return Err(PropertyError::Validation(
				"Bathroom count must be between 0 and 999.5".to_string(),
			));
		}
	}
	if let Some(balconies) = form.balcony_count {
		if balconies < 0 || balconies > 999 {
			return Err(PropertyError::Validation(
				"Balcony count must be between 0 and 999".to_string(),
			));
		}
	}
	Ok(())
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

	let projects_rows = sqlx::query_file!("queries/projects/list_project_options.sql")
		.fetch_all(&state.model.db)
		.await?;

	let projects = projects_rows
		.into_iter()
		.map(|r| ProjectOptionItem {
			id: r.id,
			name: r.name,
		})
		.collect();

	let rows = sqlx::query_file_as!(
		DbRow,
		"queries/properties/list_properties.sql",
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
	let project = sqlx::query_file!("queries/projects/check_project_exists.sql", project_id)
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
		bathroom_count: Option<f64>,
		parcel_number: Option<String>,
		zoning: Option<String>,
	}

	let rows = sqlx::query_file_as!(
		DbRow,
		"queries/properties/list_project_properties_page.sql",
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
	let project = sqlx::query_file!("queries/projects/check_project_exists.sql", project_id)
		.fetch_one(&state.model.db)
		.await?;

	let property_types = sqlx::query_file!(
		"queries/properties/list_active_property_types.sql",
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

	let amenities = sqlx::query_file!("queries/amenities/list_active_amenities.sql")
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
	validate_property_payload(&form)?;

	let slug_raw = form
		.unit_number
		.as_deref()
		.unwrap_or_else(|| form.building.as_deref().unwrap_or("unit"));
	let slug = format!("{}-{}", slugify(slug_raw), Uuid::now_v7().simple());

	let mut tx = state.model.db.begin().await?;

	// 1. Insert unified property
	let row = sqlx::query_file!(
		"queries/properties/insert_property.sql",
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
			sqlx::query_file!(
				"queries/properties/insert_residential_details.sql",
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
			sqlx::query_file!(
				"queries/properties/insert_commercial_details.sql",
				prop_id,
				form.parking.as_deref(),
			)
			.execute(&mut *tx)
			.await?;
		}
		"LAND" => {
			sqlx::query_file!(
				"queries/properties/insert_land_details.sql",
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
	sqlx::query_file!(
		"queries/properties/insert_property_listing.sql",
		prop_id,
		form.listing_type,
		form.status,
		form.price,
		form.billing_period.as_deref(),
	)
	.execute(&mut *tx)
	.await?;

	// 4. Insert amenities (deduplicating duplicate IDs)
	let mut seen = std::collections::HashSet::new();
	for amenity_id in form.amenities {
		if seen.insert(amenity_id) {
			sqlx::query_file!(
				"queries/properties/insert_property_amenity.sql",
				prop_id,
				amenity_id,
			)
			.execute(&mut *tx)
			.await?;
		}
	}

	tx.commit().await?;

	let redirect_cat = category_enum.to_lowercase();
	tracing::info!(property_id = prop_id, project_id, "created property unit");
	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::PropertyUpdated(prop_id))
		.await;
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
	let project = sqlx::query_file!("queries/projects/check_project_exists.sql", pid)
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
		bathroom_count: Option<f64>,
		balcony_count: Option<i16>,
		is_duplex: Option<bool>,
		parking_res: Option<String>,
		parking_com: Option<String>,
		parcel_number: Option<String>,
		zoning: Option<String>,
		approval_status: Option<String>,
		development_status: Option<String>,
	}

	let r = sqlx::query_file_as!(
		DbProp,
		"queries/properties/get_property_detail.sql",
		id,
		pid
	)
	.fetch_one(&state.model.db)
	.await?;

	let property_types = sqlx::query_file!(
		"queries/properties/list_active_property_types.sql",
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

	let linked_amenities: std::collections::HashSet<i32> =
		sqlx::query_file!("queries/properties/get_property_linked_amenity_ids.sql", id)
			.fetch_all(&state.model.db)
			.await?
			.into_iter()
			.map(|row| row.amenity_id)
			.collect();

	let amenities = sqlx::query_file!("queries/amenities/list_active_amenities.sql")
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
	validate_property_payload(&form)?;

	let mut tx = state.model.db.begin().await?;

	sqlx::query_file!(
		"queries/properties/update_property.sql",
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
	sqlx::query_file!(
		"queries/properties/update_property_listing.sql",
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
	let _ = sqlx::query_file!(
		"queries/properties/update_residential_details.sql",
		form.bedroom_count,
		form.bathroom_count,
		form.balcony_count,
		is_duplex,
		form.parking.as_deref(),
		id,
	)
	.execute(&mut *tx)
	.await;

	let _ = sqlx::query_file!(
		"queries/properties/update_commercial_details.sql",
		form.parking.as_deref(),
		id,
	)
	.execute(&mut *tx)
	.await;

	let _ = sqlx::query_file!(
		"queries/properties/update_land_details.sql",
		form.parcel_number.as_deref(),
		form.zoning.as_deref(),
		form.approval_status.as_deref(),
		form.development_status.as_deref(),
		id,
	)
	.execute(&mut *tx)
	.await;

	// Sync amenities
	sqlx::query_file!("queries/properties/delete_property_amenities.sql", id)
		.execute(&mut *tx)
		.await?;

	let mut seen = std::collections::HashSet::new();
	for amenity_id in form.amenities {
		if seen.insert(amenity_id) {
			sqlx::query_file!(
				"queries/properties/insert_property_amenity.sql",
				id,
				amenity_id,
			)
			.execute(&mut *tx)
			.await?;
		}
	}

	tx.commit().await?;

	tracing::info!(property_id = id, project_id = pid, "updated property");
	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::PropertyUpdated(id))
		.await;
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
	let media_rows = sqlx::query_file!("queries/properties/get_property_media_ids.sql", id)
		.fetch_all(&state.model.db)
		.await?;

	sqlx::query_file!("queries/properties/delete_all_property_media.sql", id)
		.execute(&state.model.db)
		.await?;

	for m in media_rows {
		let _ = crate::media::delete_media(&state.model.db, &state.s3_client, m.media_id).await;
	}

	// 2. Soft delete property
	sqlx::query_file!("queries/properties/delete_property.sql", id, pid)
		.execute(&state.model.db)
		.await?;

	tracing::info!(property_id = id, project_id = pid, "soft-deleted property");
	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::PropertyUpdated(id))
		.await;
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
	let project = sqlx::query_file!("queries/projects/check_project_exists.sql", pid)
		.fetch_one(&state.model.db)
		.await?;

	let prop = sqlx::query_file!("queries/properties/check_property_exists.sql", id, pid)
		.fetch_one(&state.model.db)
		.await?;

	struct DbMedia {
		media_id: Uuid,
		media_type: String,
		s3_key: String,
		thumbnail_key: Option<String>,
		sequence: i16,
	}

	let media_rows = sqlx::query_file_as!(DbMedia, "queries/properties/get_property_media.sql", id)
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
				media_type: r.media_type,
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
	// Verify property exists
	sqlx::query_file!("queries/properties/check_property_exists.sql", id, pid)
		.fetch_one(&state.model.db)
		.await?;

	let mut uploaded_s3_keys: Vec<String> = Vec::new();
	let mut uploaded_media_ids: Vec<Uuid> = Vec::new();

	let next_seq_row = sqlx::query_file!(
		"queries/properties/get_next_property_media_sequence.sql",
		id
	)
	.fetch_one(&state.model.db)
	.await?;
	let mut current_seq = next_seq_row.seq;

	let mut field_found = false;

	while let Ok(Some(field)) = multipart.next_field().await {
		let name = field.name().unwrap_or_default().to_string();
		if !matches!(
			name.as_str(),
			"image" | "video" | "media" | "file" | "files" | "files[]"
		) {
			continue;
		}

		let filename = field.file_name().unwrap_or_default().to_string();
		let content_type = field.content_type().map(|s| s.to_string());

		let bytes = match field.bytes().await {
			Ok(b) if !b.is_empty() => b,
			_ => continue,
		};

		field_found = true;
		let is_video =
			crate::media::is_video_upload(&name, content_type.as_deref(), Some(&filename));

		let upload_res: Result<(Uuid, Vec<String>), PropertyError> = if is_video {
			match crate::media::process_and_upload_video(
				&state.model.db,
				&state.s3_client,
				bytes.to_vec(),
				&filename,
				content_type.as_deref(),
			)
			.await
			{
				Ok(res) => {
					let mut keys = vec![res.s3_key];
					if let Some(tk) = res.thumbnail_key {
						keys.push(tk);
					}
					Ok((res.media_id, keys))
				}
				Err(e) => Err(PropertyError::Media(e)),
			}
		} else {
			match crate::media::process_and_upload_image(
				&state.model.db,
				&state.s3_client,
				bytes.to_vec(),
			)
			.await
			{
				Ok(res) => Ok((res.media_id, vec![res.original_key, res.thumbnail_key])),
				Err(e) => Err(PropertyError::Media(e)),
			}
		};

		match upload_res {
			Ok((mid, s3_keys)) => {
				uploaded_s3_keys.extend(s3_keys);
				uploaded_media_ids.push(mid);

				let insert_res = sqlx::query_file!(
					"queries/properties/insert_property_media.sql",
					id,
					mid,
					current_seq,
				)
				.execute(&state.model.db)
				.await;

				if let Err(e) = insert_res {
					for key in &uploaded_s3_keys {
						let _ = state
							.s3_client
							.delete_object()
							.bucket(&CONFIG.s3_bucket)
							.key(key)
							.send()
							.await;
					}
					for m_id in &uploaded_media_ids {
						let _ = sqlx::query_file!(
							"queries/properties/delete_property_media.sql",
							id,
							m_id
						)
						.execute(&state.model.db)
						.await;
						let _ = sqlx::query!("DELETE FROM media WHERE id = $1", m_id)
							.execute(&state.model.db)
							.await;
					}
					return Err(PropertyError::from(e));
				}

				current_seq += 1;
			}
			Err(e) => {
				for key in &uploaded_s3_keys {
					let _ = state
						.s3_client
						.delete_object()
						.bucket(&CONFIG.s3_bucket)
						.key(key)
						.send()
						.await;
				}
				for m_id in &uploaded_media_ids {
					let _ =
						sqlx::query_file!("queries/properties/delete_property_media.sql", id, m_id)
							.execute(&state.model.db)
							.await;
					let _ = sqlx::query!("DELETE FROM media WHERE id = $1", m_id)
						.execute(&state.model.db)
						.await;
				}
				return Err(e);
			}
		}
	}

	if !field_found || uploaded_media_ids.is_empty() {
		return Err(PropertyError::Validation(
			"No media file provided in upload".to_string(),
		));
	}

	tracing::info!(
		property_id = id,
		count = uploaded_media_ids.len(),
		"attached media to property"
	);
	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::PropertyUpdated(id))
		.await;
	Ok(Redirect::to(&format!(
		"/admin/projects/{pid}/properties/{id}/media"
	)))
}

#[derive(serde::Deserialize)]
pub struct ReorderPropertyMediaPayload {
	pub order: Vec<Uuid>,
}

async fn reorder_property_media(
	State(state): State<AppState>,
	Path((_pid, id)): Path<(i32, i32)>,
	axum::Json(payload): axum::Json<ReorderPropertyMediaPayload>,
) -> Result<StatusCode, PropertyError> {
	let mut tx = state.model.db.begin().await?;

	// 1. Shift existing sequences to negative
	sqlx::query_file!(
		"queries/properties/shift_property_media_sequences_negative.sql",
		id
	)
	.execute(&mut *tx)
	.await?;

	// 2. Set new sequences
	for (idx, mid) in payload.order.into_iter().enumerate() {
		let seq = (idx + 1) as i16;
		sqlx::query_file!(
			"queries/properties/update_property_media_sequence.sql",
			id,
			mid,
			seq
		)
		.execute(&mut *tx)
		.await?;
	}

	tx.commit().await?;
	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::PropertyUpdated(id))
		.await;
	Ok(StatusCode::OK)
}

async fn delete_property_media(
	State(state): State<AppState>,
	Path((_pid, id, mid)): Path<(i32, i32, Uuid)>,
) -> Result<StatusCode, PropertyError> {
	sqlx::query_file!("queries/properties/delete_property_media.sql", id, mid)
		.execute(&state.model.db)
		.await?;

	crate::media::delete_media(&state.model.db, &state.s3_client, mid).await?;
	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::PropertyUpdated(id))
		.await;

	Ok(StatusCode::NO_CONTENT)
}

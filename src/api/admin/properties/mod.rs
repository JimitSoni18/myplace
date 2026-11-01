use axum::{
	Json, Router,
	extract::{Multipart, Path, Query, State},
	http::StatusCode,
	routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
	AppState,
	config::CONFIG,
	session_store::AuthUser,
	utils::sql::slugify,
};

pub mod error;
use error::PropertyError;

/// Router mounted at `/api/admin/properties`
pub fn router() -> Router<AppState> {
	Router::new()
		.route("/", get(list_all_properties))
		.route(
			"/{id}",
			get(get_property_global)
				.put(update_property_global)
				.delete(delete_property_global),
		)
		.route("/{id}/media", axum::routing::post(upload_property_media_global))
		.route(
			"/{id}/media/reorder",
			axum::routing::post(reorder_property_media_global),
		)
		.route(
			"/{id}/media/{mid}",
			axum::routing::delete(delete_property_media_global),
		)
}

/// Project-scoped router mounted at `/api/admin/projects/{pid}/properties`
pub fn project_router() -> Router<AppState> {
	Router::new()
		.route("/", get(list_project_properties).post(create_property))
		.route(
			"/{id}",
			get(get_property_project)
				.put(update_property_project)
				.delete(delete_property_project),
		)
		.route(
			"/{id}/media",
			get(get_property_media_project).post(upload_property_media_project),
		)
		.route(
			"/{id}/media/reorder",
			axum::routing::post(reorder_property_media_project),
		)
		.route(
			"/{id}/media/{mid}",
			axum::routing::delete(delete_property_media_project),
		)
}

#[derive(Deserialize, ts_rs::TS)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct PropertyAllQuery {
	#[serde(default)]
	pub project_id: Option<i32>,
	#[serde(default)]
	pub category: Option<String>,
	#[serde(default)]
	pub q: Option<String>,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "admin/")]
pub struct PropertyMediaItem {
	pub id: Uuid,
	pub media_type: String,
	pub url: String,
	pub thumbnail_url: String,
	pub sequence: i16,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct PropertyGlobalItem {
	pub id: i32,
	pub project_id: i32,
	pub project_name: String,
	pub unit_number: Option<String>,
	pub building: Option<String>,
	pub unit_title: String,
	pub category: String,
	pub type_name: String,
	pub price: f64,
	pub status: String,
	pub built_up_area: Option<f64>,
}

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct PropertyDetailResponse {
	pub id: i32,
	pub project_id: i32,
	pub property_type_id: i32,
	pub category: String,
	pub type_name: String,
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
	pub bedroom_count: Option<f64>,
	pub bathroom_count: Option<f64>,
	pub balcony_count: Option<i16>,
	pub is_duplex: Option<bool>,
	pub parking: Option<String>,
	pub parcel_number: Option<String>,
	pub zoning: Option<String>,
	pub approval_status: Option<String>,
	pub development_status: Option<String>,
	pub amenities: Vec<i32>,
	pub media: Vec<PropertyMediaItem>,
}

#[derive(Deserialize, ts_rs::TS)]
#[ts(export, export_to = "admin/", optional_fields = nullable)]
pub struct PropertyFormPayload {
	pub property_type_id: i32,
	pub category: Option<String>,
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
	pub is_duplex: Option<bool>,
	pub parking: Option<String>,
	// land
	pub parcel_number: Option<String>,
	pub zoning: Option<String>,
	pub approval_status: Option<String>,
	pub development_status: Option<String>,
	#[serde(default)]
	pub amenities: Vec<i32>,
}

fn validate_property_payload(p: &PropertyFormPayload) -> Result<(), PropertyError> {
	if p.property_type_id <= 0 {
		return Err(PropertyError::Validation("Valid property type is required".to_string()));
	}
	if p.price < 0.0 {
		return Err(PropertyError::Validation("Price cannot be negative".to_string()));
	}
	if let Some(bua) = p.built_up_area {
		if bua < 0.0 {
			return Err(PropertyError::Validation("Built-up area cannot be negative".to_string()));
		}
	}
	if let Some(ua) = p.usable_area {
		if ua < 0.0 {
			return Err(PropertyError::Validation("Usable area cannot be negative".to_string()));
		}
	}
	if let Some(bc) = p.bedroom_count {
		if bc < 0.0 {
			return Err(PropertyError::Validation("Bedroom count cannot be negative".to_string()));
		}
	}
	if let Some(bc) = p.bathroom_count {
		if bc < 0.0 {
			return Err(PropertyError::Validation("Bathroom count cannot be negative".to_string()));
		}
	}
	if let Some(bc) = p.balcony_count {
		if bc < 0 {
			return Err(PropertyError::Validation("Balcony count cannot be negative".to_string()));
		}
	}
	Ok(())
}

async fn list_all_properties(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Query(query): Query<PropertyAllQuery>,
) -> Result<Json<Vec<PropertyGlobalItem>>, PropertyError> {
	let q_param = query.q.as_deref().filter(|s| !s.trim().is_empty()).map(|s| format!("%{}%", s.trim()));
	let cat_param = query.category.as_deref().filter(|s| !s.trim().is_empty());

	let rows = sqlx::query_file!(
		"queries/properties/list_properties.sql",
		query.project_id,
		cat_param,
		q_param.as_deref(),
	)
	.fetch_all(&state.model.db)
	.await?;

	let properties = rows
		.into_iter()
		.map(|r| {
			let unit_title = match (&r.unit_number, &r.building) {
				(Some(u), Some(b)) => format!("Unit {u}, {b}"),
				(Some(u), None) => format!("Unit {u}"),
				(None, Some(b)) => b.clone(),
				(None, None) => format!("Unit #{}", r.id),
			};
			PropertyGlobalItem {
				id: r.id,
				project_id: r.project_id,
				project_name: r.project_name,
				unit_number: r.unit_number,
				building: r.building,
				unit_title,
				category: r.category,
				type_name: r.type_name,
				price: r.price.unwrap_or(0.0),
				status: r.status.unwrap_or_else(|| "available".to_string()),
				built_up_area: None,
			}
		})
		.collect();

	Ok(Json(properties))
}

async fn list_project_properties(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(pid): Path<i32>,
	Query(query): Query<PropertyAllQuery>,
) -> Result<Json<Vec<PropertyGlobalItem>>, PropertyError> {
	let q_param = query.q.as_deref().filter(|s| !s.trim().is_empty()).map(|s| format!("%{}%", s.trim()));
	let cat_param = query.category.as_deref().filter(|s| !s.trim().is_empty());

	let rows = sqlx::query_file!(
		"queries/properties/list_properties.sql",
		Some(pid),
		cat_param,
		q_param.as_deref(),
	)
	.fetch_all(&state.model.db)
	.await?;

	let properties = rows
		.into_iter()
		.map(|r| {
			let unit_title = match (&r.unit_number, &r.building) {
				(Some(u), Some(b)) => format!("Unit {u}, {b}"),
				(Some(u), None) => format!("Unit {u}"),
				(None, Some(b)) => b.clone(),
				(None, None) => format!("Unit #{}", r.id),
			};
			PropertyGlobalItem {
				id: r.id,
				project_id: r.project_id,
				project_name: r.project_name,
				unit_number: r.unit_number,
				building: r.building,
				unit_title,
				category: r.category,
				type_name: r.type_name,
				price: r.price.unwrap_or(0.0),
				status: r.status.unwrap_or_else(|| "available".to_string()),
				built_up_area: None,
			}
		})
		.collect();

	Ok(Json(properties))
}

async fn fetch_property_detail(state: &AppState, id: i32, pid_opt: Option<i32>) -> Result<PropertyDetailResponse, PropertyError> {
	let pid = match pid_opt {
		Some(p) => p,
		None => {
			sqlx::query_scalar::<_, i32>(
				"SELECT project_id FROM properties WHERE id = $1 AND deleted_at IS NULL",
			)
			.bind(id)
			.fetch_one(&state.model.db)
			.await?
		}
	};

	let r = sqlx::query_file!(
		"queries/properties/get_property_detail.sql",
		id,
		pid
	)
	.fetch_one(&state.model.db)
	.await?;

	let linked_amenities: Vec<i32> = sqlx::query_file!("queries/properties/get_property_linked_amenity_ids.sql", id)
		.fetch_all(&state.model.db)
		.await?
		.into_iter()
		.map(|a| a.amenity_id)
		.collect();

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
		.map(|m| {
			let url = CONFIG.asset_url(&m.s3_key);
			let thumbnail_url = m.thumbnail_key.as_ref().map(|k| CONFIG.asset_url(k)).unwrap_or_else(|| url.clone());
			PropertyMediaItem {
				id: m.media_id,
				media_type: m.media_type,
				url,
				thumbnail_url,
				sequence: m.sequence,
			}
		})
		.collect();

	let parking = r.parking_res.or(r.parking_com);

	Ok(PropertyDetailResponse {
		id: r.id,
		project_id: pid,
		property_type_id: r.property_type_id,
		category: r.category.clone(),
		type_name: r.category,
		unit_number: r.unit_number,
		building: r.building,
		floor_number: r.floor_number,
		total_floors: r.total_floors,
		built_up_area: r.built_up_area,
		usable_area: r.usable_area,
		description: r.description,
		listing_type: r.listing_type.unwrap_or_else(|| "sale".to_string()),
		status: r.status.unwrap_or_else(|| "available".to_string()),
		price: r.price.unwrap_or(0.0),
		billing_period: r.billing_period,
		bedroom_count: r.bedroom_count,
		bathroom_count: r.bathroom_count,
		balcony_count: r.balcony_count,
		is_duplex: r.is_duplex,
		parking,
		parcel_number: r.parcel_number,
		zoning: r.zoning,
		approval_status: r.approval_status,
		development_status: r.development_status,
		amenities: linked_amenities,
		media,
	})
}

async fn get_property_global(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
) -> Result<Json<PropertyDetailResponse>, PropertyError> {
	let detail = fetch_property_detail(&state, id, None).await?;
	Ok(Json(detail))
}

async fn get_property_project(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path((pid, id)): Path<(i32, i32)>,
) -> Result<Json<PropertyDetailResponse>, PropertyError> {
	let detail = fetch_property_detail(&state, id, Some(pid)).await?;
	Ok(Json(detail))
}

async fn create_property(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(project_id): Path<i32>,
	Json(form): Json<PropertyFormPayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), PropertyError> {
	validate_property_payload(&form)?;

	// Determine category from property_type if not passed
	let category_enum = sqlx::query_scalar::<_, String>("SELECT category FROM property_types WHERE id = $1")
		.bind(form.property_type_id)
		.fetch_one(&state.model.db)
		.await?;

	let slug_raw = form
		.unit_number
		.as_deref()
		.unwrap_or_else(|| form.building.as_deref().unwrap_or("unit"));
	let slug = format!("{}-{}", slugify(slug_raw), Uuid::now_v7().simple());

	let mut tx = state.model.db.begin().await?;

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

	match category_enum.as_str() {
		"RESIDENTIAL" => {
			let is_duplex = form.is_duplex.unwrap_or(false);
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

	let listing_type = match form.listing_type.trim().to_lowercase().as_str() {

		"rent" => "rent",
		"lease" => "lease",
		_ => "sale",
	};
	let status = match form.status.trim().to_lowercase().as_str() {
		"active" | "available" => "active",
		"sold" => "sold",
		"rented" | "leased" | "closed" => "closed",
		"reserved" | "withdrawn" => "withdrawn",
		_ => "active",
	};

	sqlx::query_file!(
		"queries/properties/insert_property_listing.sql",
		prop_id,
		listing_type,
		status,
		form.price,
		form.billing_period.as_deref(),
	)
	.execute(&mut *tx)
	.await?;


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

	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::PropertyUpdated(prop_id))
		.await;

	Ok((
		StatusCode::CREATED,
		Json(json!({ "id": prop_id, "project_id": project_id, "slug": slug })),
	))
}

async fn execute_update_property(state: &AppState, pid: i32, id: i32, form: PropertyFormPayload) -> Result<(), PropertyError> {
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

	let listing_type = match form.listing_type.trim().to_lowercase().as_str() {
		"rent" => "rent",
		"lease" => "lease",
		_ => "sale",
	};
	let status = match form.status.trim().to_lowercase().as_str() {
		"active" | "available" => "active",
		"sold" => "sold",
		"rented" | "leased" | "closed" => "closed",
		"reserved" | "withdrawn" => "withdrawn",
		_ => "active",
	};

	sqlx::query_file!(
		"queries/properties/update_property_listing.sql",
		listing_type,
		status,
		form.price,
		form.billing_period.as_deref(),
		id,
	)
	.execute(&mut *tx)
	.await?;


	let is_duplex = form.is_duplex.unwrap_or(false);
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

	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::PropertyUpdated(id))
		.await;

	Ok(())
}

async fn update_property_global(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
	Json(form): Json<PropertyFormPayload>,
) -> Result<Json<serde_json::Value>, PropertyError> {
	let project_id = sqlx::query_scalar::<_, i32>("SELECT project_id FROM properties WHERE id = $1 AND deleted_at IS NULL")
		.bind(id)
		.fetch_one(&state.model.db)
		.await?;

	execute_update_property(&state, project_id, id, form).await?;
	Ok(Json(json!({ "success": true, "id": id })))
}

async fn update_property_project(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path((pid, id)): Path<(i32, i32)>,
	Json(form): Json<PropertyFormPayload>,
) -> Result<Json<serde_json::Value>, PropertyError> {
	execute_update_property(&state, pid, id, form).await?;
	Ok(Json(json!({ "success": true, "id": id })))
}

async fn execute_delete_property(state: &AppState, pid: i32, id: i32) -> Result<(), PropertyError> {
	let media_rows = sqlx::query_file!("queries/properties/get_property_media_ids.sql", id)
		.fetch_all(&state.model.db)
		.await?;

	sqlx::query_file!("queries/properties/delete_all_property_media.sql", id)
		.execute(&state.model.db)
		.await?;

	for m in media_rows {
		let _ = crate::media::delete_media(&state.model.db, &state.s3_client, m.media_id).await;
	}

	sqlx::query_file!("queries/properties/delete_property.sql", id, pid)
		.execute(&state.model.db)
		.await?;

	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::PropertyUpdated(id))
		.await;

	Ok(())
}

async fn delete_property_global(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
) -> Result<StatusCode, PropertyError> {
	let project_id = sqlx::query_scalar::<_, i32>("SELECT project_id FROM properties WHERE id = $1 AND deleted_at IS NULL")
		.bind(id)
		.fetch_one(&state.model.db)
		.await?;

	execute_delete_property(&state, project_id, id).await?;
	Ok(StatusCode::NO_CONTENT)
}

async fn delete_property_project(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path((pid, id)): Path<(i32, i32)>,
) -> Result<StatusCode, PropertyError> {
	execute_delete_property(&state, pid, id).await?;
	Ok(StatusCode::NO_CONTENT)
}

async fn get_property_media_project(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path((_pid, id)): Path<(i32, i32)>,
) -> Result<Json<Vec<PropertyMediaItem>>, PropertyError> {
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
			let thumbnail_url = r.thumbnail_key.as_ref().map(|k| CONFIG.asset_url(k)).unwrap_or_else(|| url.clone());
			PropertyMediaItem {
				id: r.media_id,
				media_type: r.media_type,
				url,
				thumbnail_url,
				sequence: r.sequence,
			}
		})
		.collect();

	Ok(Json(media))
}

async fn handle_upload_property_media(
	state: &AppState,
	pid: i32,
	id: i32,
	mut multipart: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>), PropertyError> {
	sqlx::query_file!("queries/properties/check_property_exists.sql", id, pid)
		.fetch_one(&state.model.db)
		.await?;

	let mut uploaded_s3_keys: Vec<String> = Vec::new();
	let mut uploaded_media_ids: Vec<Uuid> = Vec::new();

	let next_seq_row = sqlx::query_file!("queries/properties/get_next_property_media_sequence.sql", id)
		.fetch_one(&state.model.db)
		.await?;
	let mut current_seq = next_seq_row.seq;

	loop {
		let field = match multipart.next_field().await {
			Ok(Some(f)) => f,
			Ok(None) => break,
			Err(e) => {
				return Err(PropertyError::Validation(format!("Upload error: {e}")));
			}
		};

		let name = field.name().unwrap_or_default().to_string();
		let filename = field.file_name().unwrap_or_default().to_string();
		let content_type = field.content_type().map(|s| s.to_string());

		let bytes = field.bytes().await.map_err(|e| {
			PropertyError::Validation(format!("Failed to read file payload: {e}"))
		})?;

		if bytes.is_empty() {
			continue;
		}

		let is_video = crate::media::is_video_upload(&name, content_type.as_deref(), Some(&filename));

		let lower_name = filename.to_lowercase();
		if lower_name.ends_with(".svg") || content_type.as_deref() == Some("image/svg+xml") {
			return Err(PropertyError::Validation(
				"SVG format is not supported for gallery media. Please upload JPG, PNG, or WebP images.".to_string(),
			));
		}

		const MAX_IMAGE_BYTES: usize = 5 * 1024 * 1024; // 5 MB limit requested by user
		if !is_video && bytes.len() > MAX_IMAGE_BYTES {
			return Err(PropertyError::Validation(format!(
				"Image '{}' exceeds the 5 MB size limit ({:.2} MB uploaded).",
				filename,
				bytes.len() as f64 / 1_048_576.0
			)));
		}

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
						let _ = sqlx::query_file!("queries/properties/delete_property_media.sql", id, m_id)
							.execute(&state.model.db)
							.await;
					}
					return Err(e.into());
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
					let _ = sqlx::query_file!("queries/properties/delete_property_media.sql", id, m_id)
						.execute(&state.model.db)
						.await;
				}
				return Err(e);
			}
		}
	}

	if uploaded_media_ids.is_empty() {
		return Err(PropertyError::Validation("No media file provided in upload".to_string()));
	}

	state
		.page_cache
		.invalidate(crate::cache::InvalidationEvent::PropertyUpdated(id))
		.await;

	Ok((
		StatusCode::CREATED,
		Json(json!({ "success": true, "media_ids": uploaded_media_ids })),
	))
}

async fn upload_property_media_global(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
	multipart: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>), PropertyError> {
	let project_id = sqlx::query_scalar::<_, i32>("SELECT project_id FROM properties WHERE id = $1 AND deleted_at IS NULL")
		.bind(id)
		.fetch_one(&state.model.db)
		.await?;
	handle_upload_property_media(&state, project_id, id, multipart).await
}

async fn upload_property_media_project(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path((pid, id)): Path<(i32, i32)>,
	multipart: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>), PropertyError> {
	handle_upload_property_media(&state, pid, id, multipart).await
}

pub use crate::api::admin::projects::ReorderMediaPayload;

async fn handle_reorder_property_media(state: &AppState, id: i32, payload: ReorderMediaPayload) -> Result<Json<serde_json::Value>, PropertyError> {
	let mut tx = state.model.db.begin().await?;

	sqlx::query_file!(
		"queries/properties/shift_property_media_sequences_negative.sql",
		id
	)
	.execute(&mut *tx)
	.await?;

	for (idx, mid) in payload.order.into_iter().enumerate() {
		let seq = idx as i16;
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

	Ok(Json(json!({ "success": true })))
}

async fn reorder_property_media_global(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path(id): Path<i32>,
	Json(payload): Json<ReorderMediaPayload>,
) -> Result<Json<serde_json::Value>, PropertyError> {
	handle_reorder_property_media(&state, id, payload).await
}

async fn reorder_property_media_project(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path((_pid, id)): Path<(i32, i32)>,
	Json(payload): Json<ReorderMediaPayload>,
) -> Result<Json<serde_json::Value>, PropertyError> {
	handle_reorder_property_media(&state, id, payload).await
}

async fn delete_property_media_global(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path((id, mid)): Path<(i32, Uuid)>,
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

async fn delete_property_media_project(
	State(state): State<AppState>,
	_auth_user: AuthUser,
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

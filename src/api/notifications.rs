use axum::{
	Json, Router,
	extract::{Path, State},
	http::StatusCode,
	response::sse::{Event, KeepAlive, Sse},
	routing::{get, post},
};
use futures_util::StreamExt as _;
use serde::Serialize;
use tokio_stream::wrappers::BroadcastStream;

use crate::{
	AppState, EnquiryNotification,
	middlewares::owner_authenticate::OwnerUser,
	session_store::AuthUser,
};

#[derive(Serialize, ts_rs::TS)]
#[ts(export, export_to = "notifications/")]
pub struct NotificationsListResponse {
	pub items: Vec<EnquiryNotification>,
	pub unread_count: i64,
}

#[derive(sqlx::FromRow)]
struct DbEnquiryRow {
	id: i32,
	enquiry_type: String,
	entity_id: i32,
	entity_name: String,
	owner_id: i32,
	name: String,
	phone: String,
	email: Option<String>,
	message: Option<String>,
	is_read: bool,
	created_at: time::OffsetDateTime,
}

impl DbEnquiryRow {
	fn into_notification(self) -> EnquiryNotification {
		let created_at_str = self
			.created_at
			.format(&time::format_description::well_known::Rfc3339)
			.unwrap_or_else(|_| self.created_at.to_string());
		EnquiryNotification {
			id: self.id,
			enquiry_type: self.enquiry_type,
			entity_id: self.entity_id,
			entity_name: self.entity_name,
			owner_id: self.owner_id,
			name: self.name,
			phone: self.phone,
			email: self.email,
			message: self.message,
			is_read: self.is_read,
			created_at: created_at_str,
		}
	}
}

pub fn admin_router() -> Router<AppState> {
	Router::new()
		.route("/", get(admin_list_notifications))
		.route("/stream", get(admin_sse_stream))
		.route("/read-all", post(admin_read_all))
		.route("/{enquiry_type}/{id}/read", post(admin_mark_read))
}

pub fn owner_router() -> Router<AppState> {
	Router::new()
		.route("/", get(owner_list_notifications))
		.route("/stream", get(owner_sse_stream))
		.route("/read-all", post(owner_read_all))
		.route("/{enquiry_type}/{id}/read", post(owner_mark_read))
}

// --- ADMIN HANDLERS ---

async fn admin_sse_stream(
	State(state): State<AppState>,
	_auth_user: AuthUser,
) -> Sse<impl futures_util::Stream<Item = Result<Event, std::convert::Infallible>>> {
	let rx = state.notification_tx.subscribe();
	let stream = BroadcastStream::new(rx).filter_map(|res| async move {
		match res {
			Ok(item) => {
				let json_str = serde_json::to_string(&item).ok()?;
				Some(Ok(Event::default().event("notification").data(json_str)))
			}
			Err(_) => None,
		}
	});


	Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn admin_list_notifications(
	State(state): State<AppState>,
	_auth_user: AuthUser,
) -> Result<Json<NotificationsListResponse>, (StatusCode, Json<serde_json::Value>)> {
	let db = &state.model.db;

	let sql = r#"
		SELECT pe.id, 'project' as enquiry_type, pe.project_id as entity_id,
		       p.name as entity_name, p.project_owner_id as owner_id,
		       pe.name, pe.phone, pe.email, pe.message, pe.is_read, pe.created_at
		FROM project_enquiries pe
		JOIN projects p ON p.id = pe.project_id
		UNION ALL
		SELECT pe.id, 'property' as enquiry_type, pe.property_id as entity_id,
		       COALESCE(prop.unit_number || ' - ' || p.name, p.name) as entity_name,
		       p.project_owner_id as owner_id,
		       pe.name, pe.phone, pe.email, pe.message, pe.is_read, pe.created_at
		FROM property_enquiries pe
		JOIN properties prop ON prop.id = pe.property_id
		JOIN projects p ON p.id = prop.project_id
		ORDER BY created_at DESC
		LIMIT 50
	"#;

	let rows = sqlx::query_as::<_, DbEnquiryRow>(sql)
		.fetch_all(db)
		.await
		.unwrap_or_default();

	let unread_sql = r#"
		SELECT (
			(SELECT COUNT(*) FROM project_enquiries WHERE is_read = false) +
			(SELECT COUNT(*) FROM property_enquiries WHERE is_read = false)
		) as unread_count
	"#;

	let unread_count = sqlx::query_scalar::<_, Option<i64>>(unread_sql)
		.fetch_one(db)
		.await
		.unwrap_or_default()
		.unwrap_or(0);

	let items = rows.into_iter().map(DbEnquiryRow::into_notification).collect();

	Ok(Json(NotificationsListResponse {
		items,
		unread_count,
	}))
}

async fn admin_mark_read(
	State(state): State<AppState>,
	_auth_user: AuthUser,
	Path((enquiry_type, id)): Path<(String, i32)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
	let db = &state.model.db;
	if enquiry_type == "project" {
		let _ = sqlx::query("UPDATE project_enquiries SET is_read = true WHERE id = $1")
			.bind(id)
			.execute(db)
			.await;
	} else if enquiry_type == "property" {
		let _ = sqlx::query("UPDATE property_enquiries SET is_read = true WHERE id = $1")
			.bind(id)
			.execute(db)
			.await;
	}
	Ok(StatusCode::NO_CONTENT)
}

async fn admin_read_all(
	State(state): State<AppState>,
	_auth_user: AuthUser,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
	let db = &state.model.db;
	let _ = sqlx::query("UPDATE project_enquiries SET is_read = true WHERE is_read = false")
		.execute(db)
		.await;
	let _ = sqlx::query("UPDATE property_enquiries SET is_read = true WHERE is_read = false")
		.execute(db)
		.await;
	Ok(StatusCode::NO_CONTENT)
}

// --- OWNER HANDLERS ---

async fn owner_sse_stream(
	State(state): State<AppState>,
	owner_user: OwnerUser,
) -> Sse<impl futures_util::Stream<Item = Result<Event, std::convert::Infallible>>> {
	let rx = state.notification_tx.subscribe();
	let owner_id = owner_user.owner_id;
	let stream = BroadcastStream::new(rx).filter_map(move |res| async move {
		match res {
			Ok(item) if item.owner_id == owner_id => {
				let json_str = serde_json::to_string(&item).ok()?;
				Some(Ok(Event::default().event("notification").data(json_str)))
			}

			_ => None,
		}
	});

	Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn owner_list_notifications(
	State(state): State<AppState>,
	owner_user: OwnerUser,
) -> Result<Json<NotificationsListResponse>, (StatusCode, Json<serde_json::Value>)> {
	let db = &state.model.db;
	let owner_id = owner_user.owner_id;

	let sql = r#"
		SELECT pe.id, 'project' as enquiry_type, pe.project_id as entity_id,
		       p.name as entity_name, p.project_owner_id as owner_id,
		       pe.name, pe.phone, pe.email, pe.message, pe.is_read, pe.created_at
		FROM project_enquiries pe
		JOIN projects p ON p.id = pe.project_id
		WHERE p.project_owner_id = $1
		UNION ALL
		SELECT pe.id, 'property' as enquiry_type, pe.property_id as entity_id,
		       COALESCE(prop.unit_number || ' - ' || p.name, p.name) as entity_name,
		       p.project_owner_id as owner_id,
		       pe.name, pe.phone, pe.email, pe.message, pe.is_read, pe.created_at
		FROM property_enquiries pe
		JOIN properties prop ON prop.id = pe.property_id
		JOIN projects p ON p.id = prop.project_id
		WHERE p.project_owner_id = $1
		ORDER BY created_at DESC
		LIMIT 50
	"#;

	let rows = sqlx::query_as::<_, DbEnquiryRow>(sql)
		.bind(owner_id)
		.fetch_all(db)
		.await
		.unwrap_or_default();

	let unread_sql = r#"
		SELECT (
			(SELECT COUNT(*) FROM project_enquiries pe JOIN projects p ON p.id = pe.project_id WHERE p.project_owner_id = $1 AND pe.is_read = false) +
			(SELECT COUNT(*) FROM property_enquiries pe JOIN properties prop ON prop.id = pe.property_id JOIN projects p ON p.id = prop.project_id WHERE p.project_owner_id = $1 AND pe.is_read = false)
		) as unread_count
	"#;

	let unread_count = sqlx::query_scalar::<_, Option<i64>>(unread_sql)
		.bind(owner_id)
		.fetch_one(db)
		.await
		.unwrap_or_default()
		.unwrap_or(0);

	let items = rows.into_iter().map(DbEnquiryRow::into_notification).collect();

	Ok(Json(NotificationsListResponse {
		items,
		unread_count,
	}))
}

async fn owner_mark_read(
	State(state): State<AppState>,
	owner_user: OwnerUser,
	Path((enquiry_type, id)): Path<(String, i32)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
	let db = &state.model.db;
	let owner_id = owner_user.owner_id;

	if enquiry_type == "project" {
		let _ = sqlx::query(
			"UPDATE project_enquiries SET is_read = true WHERE id = $1 AND project_id IN (SELECT id FROM projects WHERE project_owner_id = $2)",
		)
		.bind(id)
		.bind(owner_id)
		.execute(db)
		.await;
	} else if enquiry_type == "property" {
		let _ = sqlx::query(
			"UPDATE property_enquiries SET is_read = true WHERE id = $1 AND property_id IN (SELECT prop.id FROM properties prop JOIN projects p ON p.id = prop.project_id WHERE p.project_owner_id = $2)",
		)
		.bind(id)
		.bind(owner_id)
		.execute(db)
		.await;
	}

	Ok(StatusCode::NO_CONTENT)
}

async fn owner_read_all(
	State(state): State<AppState>,
	owner_user: OwnerUser,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
	let db = &state.model.db;
	let owner_id = owner_user.owner_id;

	let _ = sqlx::query(
		"UPDATE project_enquiries SET is_read = true WHERE is_read = false AND project_id IN (SELECT id FROM projects WHERE project_owner_id = $1)",
	)
	.bind(owner_id)
	.execute(db)
	.await;

	let _ = sqlx::query(
		"UPDATE property_enquiries SET is_read = true WHERE is_read = false AND property_id IN (SELECT prop.id FROM properties prop JOIN projects p ON p.id = prop.project_id WHERE p.project_owner_id = $1)",
	)
	.bind(owner_id)
	.execute(db)
	.await;

	Ok(StatusCode::NO_CONTENT)
}

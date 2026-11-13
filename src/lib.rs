use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{Client, config::Credentials};
use std::sync::Arc;

pub mod api;
pub mod cache;
pub mod config;
pub mod constants;
pub mod crypto;
pub mod media;
pub mod middlewares;
pub mod model;
pub mod response_types;
pub mod servers;
pub mod session_store;
pub mod sitemap;
pub mod templates;
pub mod utils;

use crate::{config::CONFIG, model::Model, session_store::SessionStore};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, ts_rs::TS, utoipa::ToSchema)]
#[ts(export, export_to = "notifications/", optional_fields = nullable)]
pub struct EnquiryNotification {
	pub id: i32,
	pub enquiry_type: String, // "project" | "property"
	pub entity_id: i32,
	pub entity_name: String,
	pub owner_id: i32,
	pub name: String,
	pub phone: String,
	pub email: Option<String>,
	pub message: Option<String>,
	pub is_read: bool,
	pub created_at: String,
}

pub struct AppStateStruct {
	pub model: Model,
	pub session_store: SessionStore,
	pub s3_client: aws_sdk_s3::Client,
	pub page_cache: crate::cache::PageCache,
	pub notification_tx: tokio::sync::broadcast::Sender<EnquiryNotification>,
}

pub type AppState = Arc<AppStateStruct>;

/// Build an S3 client pointing at the Garage instance configured via environment.
pub async fn build_s3_client() -> Client {
	let credentials = Credentials::new(
		&CONFIG.s3_access_key_id,
		&CONFIG.s3_secret_access_key,
		None,
		None,
		"myplace-config",
	);
	let aws_cfg = aws_config::defaults(BehaviorVersion::latest())
		.credentials_provider(credentials)
		.region(Region::new(CONFIG.s3_region.clone()))
		.endpoint_url(&CONFIG.s3_endpoint)
		.load()
		.await;
	let s3_cfg = aws_sdk_s3::config::Builder::from(&aws_cfg)
		.force_path_style(true)
		.build();
	Client::from_conf(s3_cfg)
}

pub async fn build_app_state() -> AppState {
	let model = Model::new().await;
	let session_store = SessionStore::default();
	let s3_client = build_s3_client().await;
	let page_cache = crate::cache::PageCache::with_db(CONFIG.page_cache_capacity, model.db.clone());
	let (notification_tx, _) = tokio::sync::broadcast::channel(256);

	Arc::new(AppStateStruct {
		model,
		session_store,
		s3_client,
		page_cache,
		notification_tx,
	})
}

pub async fn build_app_state_lazy() -> AppState {
	let model = Model::new_lazy();
	let session_store = SessionStore::default();
	let s3_client = build_s3_client().await;
	let page_cache = crate::cache::PageCache::with_db(CONFIG.page_cache_capacity, model.db.clone());
	let (notification_tx, _) = tokio::sync::broadcast::channel(256);

	Arc::new(AppStateStruct {
		model,
		session_store,
		s3_client,
		page_cache,
		notification_tx,
	})
}

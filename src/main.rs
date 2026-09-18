use std::{
	net::{Ipv4Addr, SocketAddrV4},
	sync::Arc,
	time::Duration,
};

use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{Client, config::Credentials};
use axum::{Router, middleware};
use tower_http::services::ServeDir;
use tracing::info;

use crate::{
	config::CONFIG,
	model::Model,
	session_store::{SessionStore, SessionStoreTrait as _},
};

pub mod config;
pub mod crypto;
pub mod session_store;
pub mod middlewares;
pub mod response_types;
pub mod model;
pub mod constants;
pub mod templates;
pub mod api;
pub mod pages;
pub mod utils;
pub mod media;
pub mod cache;

pub struct AppStateStruct {
	pub model: Model,
	pub session_store: SessionStore,
	pub s3_client: aws_sdk_s3::Client,
	pub page_cache: crate::cache::PageCache,
}

pub type AppState = Arc<AppStateStruct>;

/// Build an S3 client pointing at the Garage instance configured via environment.
async fn build_s3_client() -> Client {
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

#[tokio::main]
async fn main() {
	// --- Logging ---
	tracing_subscriber::fmt()
		.with_env_filter(
			tracing_subscriber::EnvFilter::try_from_default_env()
				.unwrap_or_else(|_| "myplace=debug,tower_http=info".into()),
		)
		.init();

	info!("starting myplace server");

	// --- State ---
	let model = Model::new().await;
	let session_store = SessionStore::default();
	let s3_client = build_s3_client().await;
	let page_cache = crate::cache::PageCache::new(CONFIG.page_cache_capacity);

	let state = Arc::new(AppStateStruct {
		model,
		session_store,
		s3_client,
		page_cache,
	});

	// --- Background: session cleanup every 10 minutes ---
	{
		let cleanup_state = state.clone();
		tokio::spawn(async move {
			let mut interval = tokio::time::interval(Duration::from_secs(10 * 60));
			loop {
				interval.tick().await;
				let removed = cleanup_state.session_store.clean_expired().await;
				if removed > 0 {
					info!(count = removed, "cleaned up expired sessions");
				}
			}
		});
	}

	// --- Router ---
	let api_routes = Router::new()
		.nest("/admin", api::admin::router())
		.layer(middleware::from_extractor_with_state::<
			crate::session_store::AuthUser,
			AppState,
		>(state.clone()))
		.nest("/owner", api::owners::router())
		.nest("/auth", api::auth::router())
		.merge(api::public::router());

	let static_asset_server = ServeDir::new("static");

	let router = Router::new()
		.merge(api_routes)
		.with_state(state)
		.fallback_service(static_asset_server);

	let listener =
		tokio::net::TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), CONFIG.port))
			.await
			.unwrap();

	info!(port = CONFIG.port, "listening");

	axum::serve(listener, router).await.unwrap();
}

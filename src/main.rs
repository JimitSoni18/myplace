use std::{
	net::{Ipv4Addr, SocketAddrV4},
	time::Duration,
};

use myplace::{build_app_state, config::CONFIG, servers, session_store::SessionStoreTrait as _};
use tracing::info;

#[tokio::main]
async fn main() {
	// --- Logging ---
	tracing_subscriber::fmt()
		.with_env_filter(
			tracing_subscriber::EnvFilter::try_from_default_env()
				.unwrap_or_else(|_| "myplace=debug,tower_http=info".into()),
		)
		.init();

	info!("starting myplace servers");


	// --- Shared State ---
	let state = build_app_state().await;

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

	let app = servers::main_router(state);
	let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), CONFIG.public_port);
	let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

	info!(
		port = CONFIG.public_port,
		"MyPlace server listening on port {} (public: /, api: /api)",
		CONFIG.public_port
	);

	axum::serve(listener, app).await.unwrap();
}

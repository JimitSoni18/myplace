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

	// --- Build 3 separate servers ---
	let admin_app = servers::admin_router(state.clone());
	let owner_app = servers::owner_router(state.clone());
	let public_app = servers::public_router(state.clone());

	let admin_addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), CONFIG.admin_port);
	let owner_addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), CONFIG.owner_port);
	let public_addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), CONFIG.public_port);

	let admin_listener = tokio::net::TcpListener::bind(admin_addr).await.unwrap();
	let owner_listener = tokio::net::TcpListener::bind(owner_addr).await.unwrap();
	let public_listener = tokio::net::TcpListener::bind(public_addr).await.unwrap();

	info!(port = CONFIG.admin_port, "Admin server listening");
	info!(port = CONFIG.owner_port, "Owner server listening");
	info!(port = CONFIG.public_port, "Public server listening");

	let admin_handle = tokio::spawn(async move {
		axum::serve(admin_listener, admin_app).await.unwrap();
	});
	let owner_handle = tokio::spawn(async move {
		axum::serve(owner_listener, owner_app).await.unwrap();
	});
	let public_handle = tokio::spawn(async move {
		axum::serve(public_listener, public_app).await.unwrap();
	});

	let (res_admin, res_owner, res_public) =
		tokio::join!(admin_handle, owner_handle, public_handle);
	res_admin.unwrap();
	res_owner.unwrap();
	res_public.unwrap();
}

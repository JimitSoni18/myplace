use std::sync::LazyLock;

pub struct Config {
	/// Main public port (backwards compatible with `port`).
	pub port: u16,
	pub public_port: u16,
	pub admin_port: u16,
	pub owner_port: u16,

	/// Canonical origins for the 3 separate servers.
	pub public_site_origin: String,
	pub admin_site_origin: String,
	pub owner_site_origin: String,

	pub max_db_connections: u32,
	pub db_url: String,
	pub cookie_signing_secret: String,

	// Garage / S3 — backend (container-to-container)
	pub s3_endpoint: String,
	pub s3_bucket: String,
	pub s3_access_key_id: String,
	pub s3_secret_access_key: String,
	pub s3_region: String,

	/// Browser-facing asset base URL (e.g. http://myplace.web.docker.localhost:3903).
	/// Templates build image URLs as: `{asset_base_url}/{s3_key}`.
	/// The database stores only the S3 key, never the full URL.
	pub asset_base_url: String,

	/// TinyLFU page cache capacity (default 50)
	pub page_cache_capacity: u64,

	/// Backwards compatibility alias for `public_site_origin`
	pub site_base_url: String,

	/// Sitemap partition range for projects (default 1000)
	pub sitemap_projects_range: i32,
	/// Sitemap partition range for project owners (default 500)
	pub sitemap_owners_range: i32,
	/// Sitemap partition range for properties (default 2000)
	pub sitemap_properties_range: i32,
}

impl Config {
	pub fn load_from_env() -> Self {
		let _ = dotenvy::dotenv();

		let public_port: u16 = dotenvy::var("PUBLIC_PORT")
			.or_else(|_| dotenvy::var("PORT"))
			.unwrap_or_else(|_| "8080".to_string())
			.parse()
			.expect("error: invalid public port");

		let admin_port: u16 = dotenvy::var("ADMIN_PORT")
			.unwrap_or_else(|_| "8081".to_string())
			.parse()
			.expect("error: invalid admin port");

		let owner_port: u16 = dotenvy::var("OWNER_PORT")
			.unwrap_or_else(|_| "8082".to_string())
			.parse()
			.expect("error: invalid owner port");

		let public_site_origin = dotenvy::var("PUBLIC_SITE_ORIGIN")
			.or_else(|_| dotenvy::var("SITE_BASE_URL"))
			.unwrap_or_else(|_| format!("http://localhost:{}", public_port))
			.trim_end_matches('/')
			.to_string();

		let admin_site_origin = dotenvy::var("ADMIN_SITE_ORIGIN")
			.unwrap_or_else(|_| format!("http://localhost:{}", admin_port))
			.trim_end_matches('/')
			.to_string();

		let owner_site_origin = dotenvy::var("OWNER_SITE_ORIGIN")
			.unwrap_or_else(|_| format!("http://localhost:{}", owner_port))
			.trim_end_matches('/')
			.to_string();

		Self {
			port: public_port,
			public_port,
			admin_port,
			owner_port,
			public_site_origin: public_site_origin.clone(),
			admin_site_origin,
			owner_site_origin,
			max_db_connections: dotenvy::var("MAX_DB_CONNECTIONS")
				.expect("error: `MAX_DB_CONNECTIONS` environment variable is not set")
				.parse()
				.expect("error: `MAX_DB_CONNECTIONS` environment variable should be a number"),
			db_url: dotenvy::var("DATABASE_URL")
				.expect("error: `DATABASE_URL` environment variable is not set"),
			cookie_signing_secret: dotenvy::var("COOKIE_SIGNING_SECRET")
				.expect("error: `COOKIE_SIGNING_SECRET` environment variable is not set"),
			s3_endpoint: dotenvy::var("S3_ENDPOINT")
				.expect("error: `S3_ENDPOINT` environment variable is not set"),
			s3_bucket: dotenvy::var("S3_BUCKET")
				.expect("error: `S3_BUCKET` environment variable is not set"),
			s3_access_key_id: dotenvy::var("AWS_ACCESS_KEY_ID")
				.expect("error: `AWS_ACCESS_KEY_ID` environment variable is not set"),
			s3_secret_access_key: dotenvy::var("AWS_SECRET_ACCESS_KEY")
				.expect("error: `AWS_SECRET_ACCESS_KEY` environment variable is not set"),
			s3_region: dotenvy::var("AWS_DEFAULT_REGION").unwrap_or_else(|_| "garage".to_string()),
			asset_base_url: dotenvy::var("ASSET_BASE_URL")
				.expect("error: `ASSET_BASE_URL` environment variable is not set"),
			page_cache_capacity: dotenvy::var("PAGE_CACHE_CAPACITY")
				.ok()
				.and_then(|v| v.parse().ok())
				.unwrap_or(50),
			site_base_url: public_site_origin,
			sitemap_projects_range: dotenvy::var("SITEMAP_PROJECTS_RANGE")
				.ok()
				.and_then(|v| v.parse().ok())
				.unwrap_or(1000),
			sitemap_owners_range: dotenvy::var("SITEMAP_OWNERS_RANGE")
				.ok()
				.and_then(|v| v.parse().ok())
				.unwrap_or(500),
			sitemap_properties_range: dotenvy::var("SITEMAP_PROPERTIES_RANGE")
				.ok()
				.and_then(|v| v.parse().ok())
				.unwrap_or(2000),
		}
	}

	/// Build an asset URL from a stored S3 key.
	/// Keys are stored without a leading slash; this adds one.
	pub fn asset_url(&self, key: &str) -> String {
		format!("{}/{}", self.asset_base_url.trim_end_matches('/'), key)
	}

	pub fn public_url(&self, path: &str) -> String {
		format!(
			"{}/{}",
			self.public_site_origin.trim_end_matches('/'),
			path.trim_start_matches('/')
		)
	}

	pub fn admin_url(&self, path: &str) -> String {
		format!(
			"{}/{}",
			self.admin_site_origin.trim_end_matches('/'),
			path.trim_start_matches('/')
		)
	}

	pub fn owner_url(&self, path: &str) -> String {
		format!(
			"{}/{}",
			self.owner_site_origin.trim_end_matches('/'),
			path.trim_start_matches('/')
		)
	}
}

pub static CONFIG: LazyLock<Config> = LazyLock::new(Config::load_from_env);

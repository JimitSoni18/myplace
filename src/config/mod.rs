use std::sync::LazyLock;

pub struct Config {
	pub port: u16,
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

	/// Canonical website base URL for sitemap and OpenGraph (default http://localhost:3000)
	pub site_base_url: String,
}

impl Config {
	pub fn load_from_env() -> Self {
		Self {
			port: dotenvy::var("PORT")
				.expect("error: `PORT` environment variable is not set")
				.parse()
				.expect("error: `PORT` environment variable should be a valid port number"),
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
			s3_region: dotenvy::var("AWS_DEFAULT_REGION")
				.unwrap_or_else(|_| "garage".to_string()),
			asset_base_url: dotenvy::var("ASSET_BASE_URL")
				.expect("error: `ASSET_BASE_URL` environment variable is not set"),
			page_cache_capacity: dotenvy::var("PAGE_CACHE_CAPACITY")
				.ok()
				.and_then(|v| v.parse().ok())
				.unwrap_or(50),
			site_base_url: dotenvy::var("SITE_BASE_URL")
				.unwrap_or_else(|_| "http://localhost:3000".to_string()),
		}
	}

	/// Build an asset URL from a stored S3 key.
	/// Keys are stored without a leading slash; this adds one.
	pub fn asset_url(&self, key: &str) -> String {
		format!("{}/{}", self.asset_base_url.trim_end_matches('/'), key)
	}
}

pub static CONFIG: LazyLock<Config> = LazyLock::new(Config::load_from_env);


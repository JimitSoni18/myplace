use std::sync::LazyLock;

/// Explicit environment variable loading based on `APP_ENV` (or `ENVIRONMENT`).
/// - "test": loads `.env.test`, falls back to `.env`.
/// - "production": relies on system/platform environment variables (Render), only loading `.env.production` if explicitly present locally.
/// - "development" (default): loads `.env.development`, falls back to `.env`.
pub fn load_dotenv() {
	let env_name = std::env::var("APP_ENV")
		.or_else(|_| std::env::var("ENVIRONMENT"))
		.or_else(|_| std::env::var("RUST_ENV"))
		.unwrap_or_else(|_| {
			if std::env::args().any(|arg| arg.contains("test"))
				|| std::env::current_exe()
					.map(|p| p.to_string_lossy().contains("test"))
					.unwrap_or(false)
			{
				"test".to_string()
			} else {
				"development".to_string()
			}
		});

	match env_name.to_lowercase().as_str() {
		"test" => {
			let _ = dotenvy::from_filename(".env.test");
			let _ = dotenvy::dotenv();
		}
		"production" => {
			// In production, system environment variables take precedence.
			// Only try .env.production if it exists on disk locally.
			let _ = dotenvy::from_filename(".env.production");
		}
		_ => {
			// Development mode
			let _ = dotenvy::from_filename(".env.development");
			let _ = dotenvy::dotenv();
		}
	}
}

/// Automatically ensures remote database URLs (e.g. on Render) have `sslmode=require`
/// if no sslmode is explicitly specified, while preserving plaintext local connections.
pub fn normalize_db_url(url: &str) -> String {
	let mut s = url.trim().to_string();
	if !s.contains("sslmode=")
		&& !s.contains("localhost")
		&& !s.contains("127.0.0.1")
		&& !s.contains("@db:")
		&& !s.contains("@db/")
	{
		if s.contains('?') {
			s.push_str("&sslmode=require");
		} else {
			s.push_str("?sslmode=require");
		}
	}
	s
}

pub struct Config {
	/// Main public port (backwards compatible with `port`).
	pub port: u16,
	pub public_port: u16,
	pub admin_port: u16,
	pub owner_port: u16,

	/// When `true` (default), all front-ends (public, admin, owner) are multiplexed
	/// onto the single public port via path prefixes (`/`, `/admin`, `/owner`).
	/// When `false`, port-based separation is used (admin and owner listeners on their own ports).
	pub route_multiplexing: bool,

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
		load_dotenv();

		let route_multiplexing = std::env::var("ENABLE_ROUTE_MULTIPLEXING")
			.or_else(|_| std::env::var("ROUTE_MULTIPLEXING"))
			.map(|v| v.eq_ignore_ascii_case("true") || v == "1")
			.unwrap_or(cfg!(feature = "route-multiplexing"));

		let public_port: u16 = std::env::var("PUBLIC_PORT")
			.or_else(|_| std::env::var("PORT"))
			.unwrap_or_else(|_| "8080".to_string())
			.parse()
			.expect("error: invalid public port");

		let admin_port: u16 = std::env::var("ADMIN_PORT")
			.unwrap_or_else(|_| "8081".to_string())
			.parse()
			.expect("error: invalid admin port");

		let owner_port: u16 = std::env::var("OWNER_PORT")
			.unwrap_or_else(|_| "8082".to_string())
			.parse()
			.expect("error: invalid owner port");

		let public_site_origin = std::env::var("PUBLIC_SITE_ORIGIN")
			.or_else(|_| std::env::var("SITE_BASE_URL"))
			.unwrap_or_else(|_| format!("http://localhost:{}", public_port))
			.trim_end_matches('/')
			.to_string();

		let admin_site_origin = std::env::var("ADMIN_SITE_ORIGIN")
			.unwrap_or_else(|_| {
				if route_multiplexing {
					public_site_origin.clone()
				} else {
					format!("http://localhost:{}", admin_port)
				}
			})
			.trim_end_matches('/')
			.to_string();

		let owner_site_origin = std::env::var("OWNER_SITE_ORIGIN")
			.unwrap_or_else(|_| {
				if route_multiplexing {
					public_site_origin.clone()
				} else {
					format!("http://localhost:{}", owner_port)
				}
			})
			.trim_end_matches('/')
			.to_string();

		Self {
			port: public_port,
			public_port,
			admin_port,
			owner_port,
			route_multiplexing,
			public_site_origin: public_site_origin.clone(),
			admin_site_origin,
			owner_site_origin,
			max_db_connections: std::env::var("MAX_DB_CONNECTIONS")
				.unwrap_or_else(|_| "10".to_string())
				.parse()
				.expect("error: `MAX_DB_CONNECTIONS` environment variable should be a number"),
			db_url: normalize_db_url(
				&std::env::var("DATABASE_URL")
					.expect("error: `DATABASE_URL` environment variable is not set"),
			),
			cookie_signing_secret: std::env::var("COOKIE_SIGNING_SECRET")
				.expect("error: `COOKIE_SIGNING_SECRET` environment variable is not set"),
			s3_endpoint: std::env::var("S3_ENDPOINT")
				.expect("error: `S3_ENDPOINT` environment variable is not set"),
			s3_bucket: std::env::var("S3_BUCKET")
				.expect("error: `S3_BUCKET` environment variable is not set"),
			s3_access_key_id: std::env::var("AWS_ACCESS_KEY_ID")
				.expect("error: `AWS_ACCESS_KEY_ID` environment variable is not set"),
			s3_secret_access_key: std::env::var("AWS_SECRET_ACCESS_KEY")
				.expect("error: `AWS_SECRET_ACCESS_KEY` environment variable is not set"),
			s3_region: std::env::var("AWS_DEFAULT_REGION")
				.or_else(|_| std::env::var("AWS_REGION"))
				.unwrap_or_else(|_| "garage".to_string()),
			asset_base_url: std::env::var("ASSET_BASE_URL")
				.expect("error: `ASSET_BASE_URL` environment variable is not set"),
			page_cache_capacity: std::env::var("PAGE_CACHE_CAPACITY")
				.ok()
				.and_then(|v| v.parse().ok())
				.unwrap_or(50),
			site_base_url: public_site_origin,
			sitemap_projects_range: std::env::var("SITEMAP_PROJECTS_RANGE")
				.ok()
				.and_then(|v| v.parse().ok())
				.unwrap_or(1000),
			sitemap_owners_range: std::env::var("SITEMAP_OWNERS_RANGE")
				.ok()
				.and_then(|v| v.parse().ok())
				.unwrap_or(500),
			sitemap_properties_range: std::env::var("SITEMAP_PROPERTIES_RANGE")
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
		let clean = path.trim_start_matches('/');
		let subpath = if self.route_multiplexing {
			if clean.starts_with("admin/") || clean == "admin" {
				clean.to_string()
			} else {
				format!("admin/{clean}").trim_end_matches('/').to_string()
			}
		} else if clean.starts_with("admin/") {
			clean.strip_prefix("admin/").unwrap_or("").to_string()
		} else if clean == "admin" {
			"".to_string()
		} else {
			clean.to_string()
		};

		if subpath.is_empty() {
			self.admin_site_origin.clone()
		} else {
			format!("{}/{}", self.admin_site_origin.trim_end_matches('/'), subpath)
		}
	}

	pub fn owner_url(&self, path: &str) -> String {
		let clean = path.trim_start_matches('/');
		let subpath = if self.route_multiplexing {
			if clean.starts_with("owner/") || clean == "owner" {
				clean.to_string()
			} else {
				format!("owner/{clean}").trim_end_matches('/').to_string()
			}
		} else if clean.starts_with("owner/") {
			clean.strip_prefix("owner/").unwrap_or("").to_string()
		} else if clean == "owner" {
			"".to_string()
		} else {
			clean.to_string()
		};

		if subpath.is_empty() {
			self.owner_site_origin.clone()
		} else {
			format!("{}/{}", self.owner_site_origin.trim_end_matches('/'), subpath)
		}
	}
}

pub static CONFIG: LazyLock<Config> = LazyLock::new(Config::load_from_env);

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_feature_flag_default() {
		// By default, route-multiplexing feature is enabled in Cargo.toml
		assert!(cfg!(feature = "route-multiplexing"));
	}

	#[test]
	fn test_admin_and_owner_url_multiplexed() {
		let mut cfg = Config {
			port: 8080,
			public_port: 8080,
			admin_port: 8081,
			owner_port: 8082,
			route_multiplexing: true,
			public_site_origin: "https://myplace.com".to_string(),
			admin_site_origin: "https://myplace.com".to_string(),
			owner_site_origin: "https://myplace.com".to_string(),
			max_db_connections: 10,
			db_url: "postgres://localhost/test".to_string(),
			cookie_signing_secret: "secret".to_string(),
			s3_endpoint: "http://localhost:3900".to_string(),
			s3_bucket: "bucket".to_string(),
			s3_access_key_id: "id".to_string(),
			s3_secret_access_key: "key".to_string(),
			s3_region: "garage".to_string(),
			asset_base_url: "http://localhost:3900/bucket".to_string(),
			page_cache_capacity: 50,
			site_base_url: "https://myplace.com".to_string(),
			sitemap_projects_range: 1000,
			sitemap_owners_range: 500,
			sitemap_properties_range: 2000,
		};

		// Multiplexed mode: URLs have /admin and /owner prefixes
		assert_eq!(cfg.admin_url("/projects"), "https://myplace.com/admin/projects");
		assert_eq!(cfg.admin_url("/admin/projects"), "https://myplace.com/admin/projects");
		assert_eq!(cfg.admin_url("/"), "https://myplace.com/admin");
		assert_eq!(cfg.owner_url("/projects"), "https://myplace.com/owner/projects");
		assert_eq!(cfg.owner_url("/owner/projects"), "https://myplace.com/owner/projects");
		assert_eq!(cfg.owner_url("/"), "https://myplace.com/owner");

		// Non-multiplexed (port-isolated) mode: URLs do NOT have prefixes
		cfg.route_multiplexing = false;
		cfg.admin_site_origin = "http://localhost:8081".to_string();
		cfg.owner_site_origin = "http://localhost:8082".to_string();

		assert_eq!(cfg.admin_url("/projects"), "http://localhost:8081/projects");
		assert_eq!(cfg.admin_url("/admin/projects"), "http://localhost:8081/projects");
		assert_eq!(cfg.admin_url("/"), "http://localhost:8081");
		assert_eq!(cfg.owner_url("/projects"), "http://localhost:8082/projects");
		assert_eq!(cfg.owner_url("/owner/projects"), "http://localhost:8082/projects");
		assert_eq!(cfg.owner_url("/"), "http://localhost:8082");
	}
}


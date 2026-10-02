use askama::Template;
use axum::http::StatusCode;
use time::macros::format_description;
use tracing::{error, info};

use crate::{
	cache::PageCache,
	config::{CONFIG, Config},
	templates::public::{SitemapIndexItem, SitemapIndexTemplate, SitemapTemplate, SitemapUrl},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SitemapEntity {
	Projects,
	Owners,
	Properties,
}

impl SitemapEntity {
	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Projects => "projects",
			Self::Owners => "owners",
			Self::Properties => "properties",
		}
	}

	pub fn from_str(s: &str) -> Option<Self> {
		match s {
			"projects" => Some(Self::Projects),
			"owners" => Some(Self::Owners),
			"properties" => Some(Self::Properties),
			_ => None,
		}
	}
}

/// Calculate the inclusive [start_id, end_id] range for a given ID and range size.
/// For range 1000:
/// 1..=1000 -> (1, 1000)
/// 1001..=2000 -> (1001, 2000)
pub fn calculate_range(id: i32, range_size: i32) -> (i32, i32) {
	let range_size = range_size.max(1);
	let id_val = id.max(1);
	let start_id = ((id_val - 1) / range_size) * range_size + 1;
	let end_id = start_id + range_size - 1;
	(start_id, end_id)
}

/// Parse shard route parameter such as "projects-1-1000.xml" into (SitemapEntity, start_id, end_id).
pub fn parse_shard_param(param: &str) -> Option<(SitemapEntity, i32, i32)> {
	let name = param.strip_suffix(".xml")?;
	let mut parts = name.split('-');
	let entity_str = parts.next()?;
	let start_str = parts.next()?;
	let end_str = parts.next()?;
	if parts.next().is_some() {
		return None;
	}
	let entity = SitemapEntity::from_str(entity_str)?;
	let start = start_str.parse::<i32>().ok()?;
	let end = end_str.parse::<i32>().ok()?;
	if start <= 0 || end < start {
		return None;
	}
	Some((entity, start, end))
}

pub fn shard_cache_key(entity: SitemapEntity, start_id: i32, end_id: i32) -> String {
	format!("/sitemaps/{}-{}-{}.xml", entity.as_str(), start_id, end_id)
}

/// Generate XML for static pages: /sitemaps/pages.xml
pub fn generate_pages_sitemap(base_url: &str) -> Result<String, StatusCode> {
	let urls = vec![
		SitemapUrl {
			loc: format!("{base_url}/"),
			lastmod: None,
			changefreq: "daily",
			priority: "1.0",
		},
		SitemapUrl {
			loc: format!("{base_url}/projects"),
			lastmod: None,
			changefreq: "daily",
			priority: "0.9",
		},
		SitemapUrl {
			loc: format!("{base_url}/properties"),
			lastmod: None,
			changefreq: "hourly",
			priority: "0.9",
		},
	];
	SitemapTemplate { urls }.render().map_err(|e| {
		error!(error = ?e, "failed to render pages sitemap");
		StatusCode::INTERNAL_SERVER_ERROR
	})
}

struct ShardRow {
	id: i32,
	slug: String,
	updated_at: time::OffsetDateTime,
}

/// Query and generate XML for a specific projects shard [start_id, end_id].
/// Returns Ok(Some(xml)) if records found, Ok(None) if empty.
pub async fn generate_projects_shard(
	db: &sqlx::PgPool,
	base_url: &str,
	start_id: i32,
	end_id: i32,
) -> Result<Option<String>, StatusCode> {
	let projs = sqlx::query_file_as!(
		ShardRow,
		"queries/sitemap/get_projects_shard.sql",
		start_id,
		end_id
	)
	.fetch_all(db)
	.await
	.map_err(|e| {
		error!(error = ?e, "failed to query projects shard");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	if projs.is_empty() {
		return Ok(None);
	}

	let date_fmt = format_description!("[year]-[month]-[day]");
	let urls = projs
		.into_iter()
		.map(|p| SitemapUrl {
			loc: format!("{base_url}/projects/{}-{}", p.id, p.slug),
			lastmod: p.updated_at.format(&date_fmt).ok(),
			changefreq: "weekly",
			priority: "0.8",
		})
		.collect();

	let xml = SitemapTemplate { urls }.render().map_err(|e| {
		error!(error = ?e, "failed to render projects shard template");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	Ok(Some(xml))
}

/// Query and generate XML for a specific owners shard [start_id, end_id].
/// Returns Ok(Some(xml)) if records found, Ok(None) if empty.
pub async fn generate_owners_shard(
	db: &sqlx::PgPool,
	base_url: &str,
	start_id: i32,
	end_id: i32,
) -> Result<Option<String>, StatusCode> {
	let owners = sqlx::query_file_as!(
		ShardRow,
		"queries/sitemap/get_owners_shard.sql",
		start_id,
		end_id
	)
	.fetch_all(db)
	.await
	.map_err(|e| {
		error!(error = ?e, "failed to query owners shard");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	if owners.is_empty() {
		return Ok(None);
	}

	let date_fmt = format_description!("[year]-[month]-[day]");
	let urls = owners
		.into_iter()
		.map(|o| SitemapUrl {
			loc: format!("{base_url}/owners/{}-{}", o.id, o.slug),
			lastmod: o.updated_at.format(&date_fmt).ok(),
			changefreq: "weekly",
			priority: "0.7",
		})
		.collect();

	let xml = SitemapTemplate { urls }.render().map_err(|e| {
		error!(error = ?e, "failed to render owners shard template");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	Ok(Some(xml))
}

/// Query and generate XML for a specific properties shard [start_id, end_id].
/// Returns Ok(Some(xml)) if records found, Ok(None) if empty.
pub async fn generate_properties_shard(
	db: &sqlx::PgPool,
	base_url: &str,
	start_id: i32,
	end_id: i32,
) -> Result<Option<String>, StatusCode> {
	let props = sqlx::query_file_as!(
		ShardRow,
		"queries/sitemap/get_properties_shard.sql",
		start_id,
		end_id
	)
	.fetch_all(db)
	.await
	.map_err(|e| {
		error!(error = ?e, "failed to query properties shard");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	if props.is_empty() {
		return Ok(None);
	}

	let date_fmt = format_description!("[year]-[month]-[day]");
	let urls = props
		.into_iter()
		.map(|pr| SitemapUrl {
			loc: format!("{base_url}/properties/{}-{}", pr.id, pr.slug),
			lastmod: pr.updated_at.format(&date_fmt).ok(),
			changefreq: "daily",
			priority: "0.8",
		})
		.collect();

	let xml = SitemapTemplate { urls }.render().map_err(|e| {
		error!(error = ?e, "failed to render properties shard template");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	Ok(Some(xml))
}

struct RangeRow {
	range_start: i32,
	lastmod: time::OffsetDateTime,
}

/// Query distinct active shard ranges and generate root sitemap index XML.
pub async fn generate_sitemap_index(
	db: &sqlx::PgPool,
	base_url: &str,
	cfg: &Config,
) -> Result<String, StatusCode> {
	let date_fmt = format_description!("[year]-[month]-[day]");
	let mut sitemaps = Vec::new();

	// 1. Static pages sitemap
	sitemaps.push(SitemapIndexItem {
		loc: format!("{base_url}/sitemaps/pages.xml"),
		lastmod: None,
	});

	// 2. Projects shards
	let proj_ranges = sqlx::query_file_as!(
		RangeRow,
		"queries/sitemap/get_projects_ranges.sql",
		cfg.sitemap_projects_range
	)
	.fetch_all(db)
	.await
	.map_err(|e| {
		error!(error = ?e, "failed to query project ranges for sitemap index");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	for r in proj_ranges {
		let start = r.range_start;
		let end = start + cfg.sitemap_projects_range - 1;
		sitemaps.push(SitemapIndexItem {
			loc: format!("{base_url}/sitemaps/projects-{start}-{end}.xml"),
			lastmod: r.lastmod.format(&date_fmt).ok(),
		});
	}

	// 3. Owners shards
	let owner_ranges = sqlx::query_file_as!(
		RangeRow,
		"queries/sitemap/get_owners_ranges.sql",
		cfg.sitemap_owners_range
	)
	.fetch_all(db)
	.await
	.map_err(|e| {
		error!(error = ?e, "failed to query owner ranges for sitemap index");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	for r in owner_ranges {
		let start = r.range_start;
		let end = start + cfg.sitemap_owners_range - 1;
		sitemaps.push(SitemapIndexItem {
			loc: format!("{base_url}/sitemaps/owners-{start}-{end}.xml"),
			lastmod: r.lastmod.format(&date_fmt).ok(),
		});
	}

	// 4. Properties shards
	let prop_ranges = sqlx::query_file_as!(
		RangeRow,
		"queries/sitemap/get_properties_ranges.sql",
		cfg.sitemap_properties_range
	)
	.fetch_all(db)
	.await
	.map_err(|e| {
		error!(error = ?e, "failed to query property ranges for sitemap index");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	for r in prop_ranges {
		let start = r.range_start;
		let end = start + cfg.sitemap_properties_range - 1;
		sitemaps.push(SitemapIndexItem {
			loc: format!("{base_url}/sitemaps/properties-{start}-{end}.xml"),
			lastmod: r.lastmod.format(&date_fmt).ok(),
		});
	}

	SitemapIndexTemplate { sitemaps }.render().map_err(|e| {
		error!(error = ?e, "failed to render sitemap index template");
		StatusCode::INTERNAL_SERVER_ERROR
	})
}

/// Update only the specific shard containing the given entity ID.
/// If records exist, the shard cache is updated; if empty, it is removed from the cache.
/// The sitemap index is invalidated to reflect any changes.
pub async fn update_entity_shard(
	db: &sqlx::PgPool,
	page_cache: &PageCache,
	entity: SitemapEntity,
	id: i32,
) {
	let (start_id, end_id) = match entity {
		SitemapEntity::Projects => calculate_range(id, CONFIG.sitemap_projects_range),
		SitemapEntity::Owners => calculate_range(id, CONFIG.sitemap_owners_range),
		SitemapEntity::Properties => calculate_range(id, CONFIG.sitemap_properties_range),
	};

	let shard_key = shard_cache_key(entity, start_id, end_id);
	let base = CONFIG.public_site_origin.as_str();

	let shard_result = match entity {
		SitemapEntity::Projects => generate_projects_shard(db, base, start_id, end_id).await,
		SitemapEntity::Owners => generate_owners_shard(db, base, start_id, end_id).await,
		SitemapEntity::Properties => generate_properties_shard(db, base, start_id, end_id).await,
	};

	match shard_result {
		Ok(Some(xml)) => {
			info!(key = %shard_key, "updating sitemap shard in cache");
			page_cache.insert(shard_key, xml).await;
		}
		Ok(None) => {
			info!(key = %shard_key, "removing empty sitemap shard from cache");
			page_cache.invalidate_key(&shard_key).await;
		}
		Err(e) => {
			error!(error = ?e, key = %shard_key, "error regenerating sitemap shard");
		}
	}

	// Invalidate sitemap index so it reflects updated shards
	page_cache.invalidate_key("/sitemap.xml").await;
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_range_calculation_project_default() {
		// Range 1000
		assert_eq!(calculate_range(1, 1000), (1, 1000));
		assert_eq!(calculate_range(500, 1000), (1, 1000));
		assert_eq!(calculate_range(1000, 1000), (1, 1000));
		assert_eq!(calculate_range(1001, 1000), (1001, 2000));
		assert_eq!(calculate_range(2000, 1000), (1001, 2000));
		assert_eq!(calculate_range(2001, 1000), (2001, 3000));
	}

	#[test]
	fn test_range_calculation_owner_default() {
		// Range 500
		assert_eq!(calculate_range(1, 500), (1, 500));
		assert_eq!(calculate_range(250, 500), (1, 500));
		assert_eq!(calculate_range(500, 500), (1, 500));
		assert_eq!(calculate_range(501, 500), (501, 1000));
		assert_eq!(calculate_range(1000, 500), (501, 1000));
		assert_eq!(calculate_range(1001, 500), (1001, 1500));
	}

	#[test]
	fn test_range_calculation_property_default() {
		// Range 2000
		assert_eq!(calculate_range(1, 2000), (1, 2000));
		assert_eq!(calculate_range(1500, 2000), (1, 2000));
		assert_eq!(calculate_range(2000, 2000), (1, 2000));
		assert_eq!(calculate_range(2001, 2000), (2001, 4000));
		assert_eq!(calculate_range(4000, 2000), (2001, 4000));
	}

	#[test]
	fn test_range_calculation_edge_cases() {
		// 0 or negative ID
		assert_eq!(calculate_range(0, 1000), (1, 1000));
		assert_eq!(calculate_range(-5, 1000), (1, 1000));

		// range_size <= 0
		assert_eq!(calculate_range(1, 0), (1, 1));
	}

	#[test]
	fn test_parse_shard_param() {
		assert_eq!(
			parse_shard_param("projects-1-1000.xml"),
			Some((SitemapEntity::Projects, 1, 1000))
		);
		assert_eq!(
			parse_shard_param("owners-501-1000.xml"),
			Some((SitemapEntity::Owners, 501, 1000))
		);
		assert_eq!(
			parse_shard_param("properties-2001-4000.xml"),
			Some((SitemapEntity::Properties, 2001, 4000))
		);

		// Invalid cases
		assert_eq!(parse_shard_param("pages.xml"), None);
		assert_eq!(parse_shard_param("unknown-1-1000.xml"), None);
		assert_eq!(parse_shard_param("projects-abc-1000.xml"), None);
		assert_eq!(parse_shard_param("projects-1000-100.xml"), None); // end < start
		assert_eq!(parse_shard_param("projects-1-1000"), None); // missing .xml
	}
}

use askama::Template;
use axum::http::StatusCode;
use time::macros::format_description;
use tracing::{error, info};

use crate::{
	cache::PageCache,
	config::Config,
	templates::public::{SitemapIndexItem, SitemapIndexTemplate, SitemapTemplate, SitemapUrl},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SitemapEntity {
	Projects,
	Owners,
}

impl SitemapEntity {
	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Projects => "projects",
			Self::Owners => "owners",
		}
	}

	pub fn from_str(s: &str) -> Option<Self> {
		match s {
			"projects" => Some(Self::Projects),
			"owners" => Some(Self::Owners),
			_ => None,
		}
	}
}

/// Calculate the inclusive [start_id, end_id] range for a given ID and range size.
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
	];
	SitemapTemplate { urls }.render().map_err(|e| {
		error!(error = ?e, "failed to render pages sitemap");
		StatusCode::INTERNAL_SERVER_ERROR
	})
}

struct ShardRow {
	#[allow(dead_code)]
	id: i32,
	slug: String,
	updated_at: time::OffsetDateTime,
}

/// Query and generate XML for a specific projects shard [start_id, end_id].
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
			loc: format!("{base_url}/projects/{}", p.slug),
			lastmod: p.updated_at.format(&date_fmt).ok(),
			changefreq: "daily",
			priority: "0.9",
		})
		.collect();

	let xml = SitemapTemplate { urls }.render().map_err(|e| {
		error!(error = ?e, "failed to render projects shard template");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	Ok(Some(xml))
}

/// Query and generate XML for a specific owners shard [start_id, end_id].
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
			loc: format!("{base_url}/owners/{}", o.slug),
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

	SitemapIndexTemplate { sitemaps }.render().map_err(|e| {
		error!(error = ?e, "failed to render sitemap index template");
		StatusCode::INTERNAL_SERVER_ERROR
	})
}

/// Generic shard generator that dispatches by entity type.
pub async fn generate_shard(
	db: &sqlx::PgPool,
	base_url: &str,
	entity: SitemapEntity,
	start_id: i32,
	end_id: i32,
) -> Result<Option<String>, StatusCode> {
	match entity {
		SitemapEntity::Projects => generate_projects_shard(db, base_url, start_id, end_id).await,
		SitemapEntity::Owners => generate_owners_shard(db, base_url, start_id, end_id).await,
	}
}

pub async fn build_and_cache_sitemap_index(
	db: &sqlx::PgPool,
	base_url: &str,
	cfg: &Config,
	cache: &PageCache,
) -> Result<String, StatusCode> {
	let xml = generate_sitemap_index(db, base_url, cfg).await?;
	cache.insert("/sitemap.xml", &xml).await;
	Ok(xml)
}

pub async fn build_entity_sitemap_shard(
	db: &sqlx::PgPool,
	base_url: &str,
	cfg: &Config,
	entity: SitemapEntity,
	entity_id: i32,
	cache: &PageCache,
) -> Result<(), StatusCode> {
	let range_size = match entity {
		SitemapEntity::Projects => cfg.sitemap_projects_range,
		SitemapEntity::Owners => cfg.sitemap_owners_range,
	};
	let (start_id, end_id) = calculate_range(entity_id, range_size);
	let cache_key = shard_cache_key(entity, start_id, end_id);

	if let Some(xml) = generate_shard(db, base_url, entity, start_id, end_id).await? {
		cache.insert(&cache_key, &xml).await;
		info!(%cache_key, "cached updated entity sitemap shard");
	} else {
		cache.remove(&cache_key).await;
	}

	build_and_cache_sitemap_index(db, base_url, cfg, cache).await?;
	Ok(())
}

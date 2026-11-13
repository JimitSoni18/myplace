use moka::future::Cache;
use tracing::info;

#[derive(Debug, Clone)]
pub enum InvalidationEvent {
	ProjectUpdated(i32),
	OwnerUpdated(i32),
	All,
}

#[derive(Clone)]
pub struct PageCache {
	cache: Cache<String, String>,
	db: Option<sqlx::PgPool>,
}

impl PageCache {
	pub fn new(capacity: u64) -> Self {
		let cache = Cache::builder().max_capacity(capacity).build();
		Self { cache, db: None }
	}

	pub fn with_db(capacity: u64, db: sqlx::PgPool) -> Self {
		let cache = Cache::builder().max_capacity(capacity).build();
		Self {
			cache,
			db: Some(db),
		}
	}

	pub async fn get(&self, key: &str) -> Option<String> {
		self.cache.get(key).await
	}

	pub async fn insert(&self, key: impl Into<String>, html: impl Into<String>) {
		self.cache.insert(key.into(), html.into()).await;
	}

	pub async fn remove(&self, key: &str) {
		self.cache.invalidate(key).await;
	}

	pub async fn invalidate_key(&self, key: &str) {
		self.cache.invalidate(key).await;
	}

	pub async fn invalidate(&self, event: InvalidationEvent) {
		match event {
			InvalidationEvent::ProjectUpdated(id) => {
				info!(project_id = id, "invalidating project cache entries");
				self.cache.invalidate("/").await;
				self.cache.invalidate("/projects").await;
				self.cache.invalidate("/sitemap.xml").await;
				// Invalidate all /projects/ pages
				self.cache
					.invalidate_entries_if(|k, _| k.starts_with("/projects"))
					.ok();

				if let Some(db) = &self.db {
					let _ = crate::sitemap::build_entity_sitemap_shard(
						db,
						&crate::config::CONFIG.public_site_origin,
						&crate::config::CONFIG,
						crate::sitemap::SitemapEntity::Projects,
						id,
						self,
					)
					.await;
				}
			}
			InvalidationEvent::OwnerUpdated(id) => {
				info!(owner_id = id, "invalidating owner cache entries");
				self.cache.invalidate("/").await;
				self.cache.invalidate("/projects").await;
				self.cache.invalidate("/sitemap.xml").await;
				self.cache
					.invalidate_entries_if(|k, _| k.starts_with("/owners"))
					.ok();

				if let Some(db) = &self.db {
					let _ = crate::sitemap::build_entity_sitemap_shard(
						db,
						&crate::config::CONFIG.public_site_origin,
						&crate::config::CONFIG,
						crate::sitemap::SitemapEntity::Owners,
						id,
						self,
					)
					.await;
				}
			}
			InvalidationEvent::All => {
				info!("invalidating all page cache entries");
				self.cache.invalidate_all();
			}
		}
	}

	/// Dispatch invalidation in the background without blocking the caller.
	pub fn dispatch_invalidation(&self, event: InvalidationEvent) {
		let this = self.clone();
		tokio::spawn(async move {
			this.invalidate(event).await;
		});
	}
}

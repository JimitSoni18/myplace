use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

use crate::config::CONFIG;

pub type Db = Pool<Postgres>;

#[derive(Clone)]
pub struct Model {
	pub db: Db,
}

impl Model {
	pub async fn new() -> Self {
		let raw_db_url = CONFIG.db_url.as_str();
		let database_url = crate::config::normalize_db_url(raw_db_url);
		let mut attempts = 0;
		let db = loop {
			match PgPoolOptions::new()
				.max_connections(CONFIG.max_db_connections)
				.acquire_timeout(std::time::Duration::from_secs(3))
				.connect(&database_url)
				.await
			{
				Ok(pool) => break pool,
				Err(e) => {
					attempts += 1;
					if attempts >= 30 {
						panic!("failed to connect to database after 30 attempts: {e}");
					}
					tokio::time::sleep(std::time::Duration::from_secs(1)).await;
				}
			}
		};
		Model { db }
	}
}

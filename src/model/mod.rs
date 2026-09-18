use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

use crate::config::CONFIG;

pub type Db = Pool<Postgres>;

#[derive(Clone)]
pub struct Model {
	pub db: Db,
}

impl Model {
	pub async fn new() -> Self {
		Model {
			db: PgPoolOptions::new()
				.max_connections(CONFIG.max_db_connections)
				.connect(CONFIG.db_url.as_str())
				.await
				.unwrap(),
		}
	}
}

use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	myplace::config::load_dotenv();

	let raw_db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
	let database_url = myplace::config::normalize_db_url(&raw_db_url);

	println!("[migrate] connecting to database...");
	let mut attempts = 0;
	let pool = loop {
		match PgPoolOptions::new()
			.max_connections(2)
			.acquire_timeout(std::time::Duration::from_secs(3))
			.connect(&database_url)
			.await
		{
			Ok(p) => break p,
			Err(e) => {
				attempts += 1;
				if attempts >= 30 {
					eprintln!("[migrate] failed to connect to database after 30 attempts: {e}");
					return Err(e.into());
				}
				println!("[migrate] waiting for database to be ready (attempt {attempts}/30)...");
				tokio::time::sleep(std::time::Duration::from_secs(1)).await;
			}
		}
	};

	println!("[migrate] applying database migrations...");
	sqlx::migrate!("./migrations").run(&pool).await?;

	println!("[migrate] database migrations applied successfully");
	Ok(())
}

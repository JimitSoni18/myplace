use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	myplace::config::load_dotenv();

	let raw_db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
	let database_url = myplace::config::normalize_db_url(&raw_db_url);

	println!("[migrate] connecting to database...");
	let pool = PgPoolOptions::new()
		.max_connections(2)
		.connect(&database_url)
		.await?;

	println!("[migrate] applying database migrations...");
	sqlx::migrate!("./migrations").run(&pool).await?;

	println!("[migrate] database migrations applied successfully");
	Ok(())
}

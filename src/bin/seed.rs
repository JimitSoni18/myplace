/// Seed binary: creates the initial super-admin account.
///
/// Reads ADMIN_USERNAME and ADMIN_PASSWORD from the environment (.env is loaded
/// automatically). Safe to run multiple times — will report "already seeded" and
/// exit 0 if the account already exists. Never overwrites an existing password.
///
/// Usage:
///   ./seed
///   # or via Dockerfile CMD: sqlx migrate run && ./seed && ./myplace

use argon2::{
	Argon2,
	password_hash::{PasswordHasher as _, SaltString, rand_core::OsRng},
};
use sqlx::{PgPool, postgres::PgPoolOptions};

#[tokio::main]
async fn main() {
	// Load .env so local dev works the same as Docker
	let _ = dotenvy::dotenv();

	let admin_username = std::env::var("ADMIN_USERNAME")
		.expect("ADMIN_USERNAME must be set for the seed binary");
	let admin_password = std::env::var("ADMIN_PASSWORD")
		.expect("ADMIN_PASSWORD must be set for the seed binary");
	let database_url = std::env::var("DATABASE_URL")
		.expect("DATABASE_URL must be set");

	if admin_username.is_empty() || admin_password.is_empty() {
		eprintln!("error: ADMIN_USERNAME and ADMIN_PASSWORD must not be empty");
		std::process::exit(1);
	}

	// Connect to Postgres
	let pool: PgPool = PgPoolOptions::new()
		.max_connections(2)
		.connect(&database_url)
		.await
		.expect("failed to connect to database");

	// Check whether the admin already exists
	let existing = sqlx::query_scalar!(
		"SELECT 1 FROM profiles p
		 INNER JOIN admin_users a ON a.profile_id = p.id
		 WHERE p.username = $1",
		admin_username,
	)
	.fetch_optional(&pool)
	.await
	.expect("database query failed");

	if existing.is_some() {
		println!(
			"[seed] admin '{}' already exists — skipping",
			admin_username
		);
		return;
	}

	// Hash the password with Argon2id
	let argon2 = Argon2::default();
	let salt = SaltString::generate(&mut OsRng);
	let password_hash = argon2
		.hash_password(admin_password.as_bytes(), &salt)
		.expect("failed to hash password")
		.to_string();

	// Create profile + admin_user in a single transaction
	let mut txn = pool.begin().await.expect("failed to begin transaction");

	let profile_id = sqlx::query_scalar!(
		"INSERT INTO profiles (username, password) VALUES ($1, $2) RETURNING id",
		admin_username,
		password_hash,
	)
	.fetch_one(&mut *txn)
	.await
	.expect("failed to insert profile");

	sqlx::query!(
		"INSERT INTO admin_users (profile_id) VALUES ($1)",
		profile_id,
	)
	.execute(&mut *txn)
	.await
	.expect("failed to insert admin_user");

	txn.commit().await.expect("failed to commit transaction");

	println!("[seed] admin '{}' created successfully", admin_username);
}

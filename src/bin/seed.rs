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

	let admin_username =
		std::env::var("ADMIN_USERNAME").expect("ADMIN_USERNAME must be set for the seed binary");
	let admin_password =
		std::env::var("ADMIN_PASSWORD").expect("ADMIN_PASSWORD must be set for the seed binary");
	let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

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

	if existing.is_none() {
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
	} else {
		println!("[seed] admin '{}' already exists", admin_username);
	}

	// -----------------------------------------------------------------------
	// Seed Amenities
	// -----------------------------------------------------------------------
	let amenity_count = sqlx::query_scalar!("SELECT COUNT(*) FROM amenities")
		.fetch_one(&pool)
		.await
		.unwrap()
		.unwrap_or(0);

	if amenity_count == 0 {
		sqlx::query!(
			r#"
			INSERT INTO amenities (name, slug, is_active) VALUES
			('Swimming Pool', 'swimming-pool', true),
			('Gymnasium', 'gymnasium', true),
			('Clubhouse', 'clubhouse', true),
			('24/7 Security', '24-7-security', true),
			('Children Play Area', 'children-play-area', true)
			ON CONFLICT DO NOTHING
			"#
		)
		.execute(&pool)
		.await
		.expect("failed to seed amenities");
		println!("[seed] amenities seeded");
	}

	// -----------------------------------------------------------------------
	// Seed Location
	// -----------------------------------------------------------------------
	let location_id = match sqlx::query_scalar!("SELECT id FROM locations LIMIT 1")
		.fetch_optional(&pool)
		.await
		.unwrap()
	{
		Some(id) => id,
		None => {
			let loc_id = sqlx::query_scalar!(
				r#"
				INSERT INTO locations (formatted_address, city, state_or_province, country_name)
				VALUES ('100 Feet Rd, Indiranagar, Bengaluru, Karnataka, 560038', 'Bengaluru', 'Karnataka', 'India')
				RETURNING id
				"#
			)
			.fetch_one(&pool)
			.await
			.expect("failed to seed location");
			println!("[seed] location seeded");
			loc_id
		}
	};

	// -----------------------------------------------------------------------
	// Seed Project Owner
	// -----------------------------------------------------------------------
	let owner_id =
		match sqlx::query_scalar!("SELECT id FROM project_owners WHERE deleted_at IS NULL LIMIT 1")
			.fetch_optional(&pool)
			.await
			.unwrap()
		{
			Some(id) => id,
			None => {
				let argon2 = Argon2::default();
				let salt = SaltString::generate(&mut OsRng);
				let owner_pw_hash = argon2
					.hash_password(b"ownerpassword123", &salt)
					.expect("failed to hash owner password")
					.to_string();

				let owner_profile_id = sqlx::query_scalar!(
					"INSERT INTO profiles (username, password) VALUES ('prestige_owner', $1) RETURNING id",
					owner_pw_hash
				)
				.fetch_one(&pool)
				.await
				.expect("failed to insert owner profile");

				let oid = sqlx::query_scalar!(
					r#"
				INSERT INTO project_owners (profile_id, name, slug, email, phone, active)
				VALUES ($1, 'Prestige Estates', 'prestige-estates', 'contact@prestige.com', '9876543210', true)
				RETURNING id
				"#,
					owner_profile_id
				)
				.fetch_one(&pool)
				.await
				.expect("failed to seed project owner");
				println!("[seed] project owner seeded");
				oid
			}
		};

	// -----------------------------------------------------------------------
	// Seed Project
	// -----------------------------------------------------------------------
	let project_id = match sqlx::query_scalar!(
		"SELECT id FROM projects WHERE deleted_at IS NULL LIMIT 1"
	)
	.fetch_optional(&pool)
	.await
	.unwrap()
	{
		Some(id) => id,
		None => {
			let pid = sqlx::query_scalar!(
				r#"
				INSERT INTO projects (project_owner_id, location_id, name, slug, description, category)
				VALUES ($1, $2, 'Prestige Falcon City', 'prestige-falcon-city', 'Premium residential apartments in Bengaluru.', 'Residential')
				RETURNING id
				"#,
				owner_id,
				location_id
			)
			.fetch_one(&pool)
			.await
			.expect("failed to seed project");

			sqlx::query!(
				r#"
				INSERT INTO project_amenities (project_id, amenity_id)
				SELECT $1, id FROM amenities LIMIT 2
				"#,
				pid
			)
			.execute(&pool)
			.await
			.ok();

			println!("[seed] project seeded");
			pid
		}
	};

	// -----------------------------------------------------------------------
	// Seed Property
	// -----------------------------------------------------------------------
	let prop_count =
		sqlx::query_scalar!("SELECT COUNT(*) FROM properties WHERE deleted_at IS NULL")
			.fetch_one(&pool)
			.await
			.unwrap()
			.unwrap_or(0);

	if prop_count == 0 {
		let prop_type_id =
			sqlx::query_scalar!("SELECT id FROM property_types WHERE slug = 'flat' LIMIT 1")
				.fetch_one(&pool)
				.await
				.expect("flat property type must exist");

		let prop_id = sqlx::query_scalar!(
			r#"
			INSERT INTO properties (
				project_id, property_type_id, unit_number, building,
				built_up_area, usable_area, description, slug
			)
			VALUES ($1, $2, '101', 'Tower A', 1250.0, 980.0, 'Spacious 2 BHK Apartment', 'unit-101-seed')
			RETURNING id
			"#,
			project_id,
			prop_type_id
		)
		.fetch_one(&pool)
		.await
		.expect("failed to seed property");

		sqlx::query!(
			r#"
			INSERT INTO residential_property_details (
				property_id, bedroom_count, bathroom_count, balcony_count, is_duplex, parking
			)
			VALUES ($1, 2.0, 2.0, 1, false, '1 Covered')
			"#,
			prop_id
		)
		.execute(&pool)
		.await
		.expect("failed to seed residential property details");

		sqlx::query!(
			r#"
			INSERT INTO property_listings (property_id, listing_type, status, price)
			VALUES ($1, 'sale', 'active', 7500000.0)
			"#,
			prop_id
		)
		.execute(&pool)
		.await
		.expect("failed to seed property listing");

		println!("[seed] property seeded");
	}
}

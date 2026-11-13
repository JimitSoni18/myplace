/// Seed binary: creates the initial super-admin account, default project types, amenities, and demo project.
///
/// Reads ADMIN_USERNAME and ADMIN_PASSWORD from the environment (.env is loaded
/// automatically). Safe to run multiple times.
use argon2::{
	Argon2,
	password_hash::{PasswordHasher as _, SaltString, rand_core::OsRng},
};
use sqlx::{PgPool, postgres::PgPoolOptions};

#[tokio::main]
async fn main() {
	// Load environment variables (.env.development, .env, or system env based on APP_ENV)
	myplace::config::load_dotenv();

	let admin_username =
		std::env::var("ADMIN_USERNAME").expect("ADMIN_USERNAME must be set for the seed binary");
	let admin_password =
		std::env::var("ADMIN_PASSWORD").expect("ADMIN_PASSWORD must be set for the seed binary");
	let raw_db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
	let database_url = myplace::config::normalize_db_url(&raw_db_url);

	if admin_username.is_empty() || admin_password.is_empty() {
		eprintln!("error: ADMIN_USERNAME and ADMIN_PASSWORD must not be empty");
		std::process::exit(1);
	}

	// Connect to Postgres
	let mut attempts = 0;
	let pool: PgPool = loop {
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
					eprintln!("[seed] failed to connect to database after 30 attempts: {e}");
					std::process::exit(1);
				}
				println!("[seed] waiting for database to be ready (attempt {attempts}/30)...");
				tokio::time::sleep(std::time::Duration::from_secs(1)).await;
			}
		}
	};

	// -----------------------------------------------------------------------
	// 1. Seed Super-Admin User
	// -----------------------------------------------------------------------
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
		let argon2 = Argon2::default();
		let salt = SaltString::generate(&mut OsRng);
		let password_hash = argon2
			.hash_password(admin_password.as_bytes(), &salt)
			.expect("failed to hash password")
			.to_string();

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
	// 2. Seed Amenities (No slug)
	// -----------------------------------------------------------------------
	let amenity_count = sqlx::query_scalar!("SELECT COUNT(*) FROM amenities")
		.fetch_one(&pool)
		.await
		.unwrap()
		.unwrap_or(0);

	if amenity_count == 0 {
		sqlx::query!(
			r#"
			INSERT INTO amenities (name, is_active) VALUES
			('Swimming Pool', true),
			('Gymnasium', true),
			('Clubhouse', true),
			('24/7 Security', true),
			('Children Play Area', true),
			('Jogging Track', true),
			('Landscaped Gardens', true),
			('Power Backup', true),
			('EV Charging Station', true)
			ON CONFLICT (name) DO NOTHING
			"#
		)
		.execute(&pool)
		.await
		.expect("failed to seed amenities");
		println!("[seed] amenities seeded");
	}

	// -----------------------------------------------------------------------
	// 3. Seed Project Owner (Builder / Developer)
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
					INSERT INTO project_owners (profile_id, name, slug, bio, email, phone, active)
					VALUES ($1, 'Prestige Estates', 'prestige-estates', 'Leading real estate developer with luxury residential developments.', 'contact@prestige.com', '9876543210', true)
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
	// 4. Seed Project with Embedded Location, RERA & Scale
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
			let residential_type_id = sqlx::query_scalar!(
				"SELECT id FROM project_types WHERE name = 'Residential' LIMIT 1"
			)
			.fetch_one(&pool)
			.await
			.expect("Residential project type must exist");

			let pid = sqlx::query_scalar!(
				r#"
				INSERT INTO projects (
					project_owner_id, project_type_id, name, slug, description,
					formatted_address, city, area, state_or_province, postal_code,
					latitude, longitude, rera_number, rera_authority,
					total_land_area, towers_count, floors_count, units_count
				)
				VALUES (
					$1, $2, 'Prestige Falcon City', 'prestige-falcon-city',
					'Prestige Falcon City is a state-of-the-art master planned community offering premium living spaces, lush landscaping, and lifestyle amenities.',
					'Kanakapura Road, Anjanadri Layout, Konanakunte, Bengaluru, Karnataka 560062',
					'Bengaluru', 'Kanakapura Road', 'Karnataka', '560062',
					12.8797000, 77.5528000, 'PRM/KA/RERA/1251/310/PR/170915/000256',
					'Karnataka Real Estate Regulatory Authority (K-RERA)',
					217800.0, 7, 31, 2520
				)
				RETURNING id
				"#,
				owner_id,
				residential_type_id
			)
			.fetch_one(&pool)
			.await
			.expect("failed to seed project");

			// Link amenities
			sqlx::query!(
				r#"
				INSERT INTO project_amenities (project_id, amenity_id)
				SELECT $1, id FROM amenities LIMIT 5
				"#,
				pid
			)
			.execute(&pool)
			.await
			.ok();

			// -------------------------------------------------------------------
			// 5. Seed Project Configurations (1:1 with floor plan / pricing)
			// -------------------------------------------------------------------
			sqlx::query!(
				r#"
				INSERT INTO project_configurations (
					project_id, name, bhk, carpet_area, built_up_area, super_built_up_area, price, price_on_request, sequence
				)
				VALUES
				($1, '2 BHK Executive', 2.0, 850.0, 1050.0, 1250.0, 8500000.0, false, 1),
				($1, '3 BHK Classic', 3.0, 1150.0, 1420.0, 1680.0, 12500000.0, false, 2),
				($1, '4 BHK Royal Sky Suite', 4.0, 1950.0, 2400.0, 2850.0, 22000000.0, false, 3)
				"#,
				pid
			)
			.execute(&pool)
			.await
			.expect("failed to seed project configurations");

			// -------------------------------------------------------------------
			// 6. Seed Project Specifications
			// -------------------------------------------------------------------
			sqlx::query!(
				r#"
				INSERT INTO project_specifications (project_id, category, items, sequence)
				VALUES
				($1, 'Structure', ARRAY['RCC framed earthquake-resistant structure with solid concrete block masonry.'], 1),
				($1, 'Flooring', ARRAY['Vitrified tiles in living, dining, and corridors', 'Laminated wooden flooring in master bedroom', 'Anti-skid ceramic tiles in balconies and utility areas'], 2),
				($1, 'Kitchen', ARRAY['Engineered granite counter platform with stainless steel sink', 'Provision for water purifier and exhaust fan', 'Glazed ceramic tiles up to 2 feet above platform'], 3),
				($1, 'Doors & Windows', ARRAY['Main door with teakwood frame and polished designer shutter', 'UPVC/Powder-coated aluminum sliding windows with mosquito mesh'], 4),
				($1, 'Electrical', ARRAY['Concealed copper wiring with modular switches (Schneider/Legrand)', 'TV and telephone points in living room and bedrooms', '100% DG power backup for common areas'], 5)
				"#,
				pid
			)
			.execute(&pool)
			.await
			.expect("failed to seed project specifications");

			// -------------------------------------------------------------------
			// 7. Seed Project Landmarks
			// -------------------------------------------------------------------
			sqlx::query!(
				r#"
				INSERT INTO project_landmarks (project_id, category, name, distance, duration, source, sequence)
				VALUES
				($1, 'Connectivity', 'Konanakunte Cross Metro Station', '300 meters', '3 mins walk', 'manual', 1),
				($1, 'Shopping', 'Forum South Bengaluru Mall', '500 meters', '5 mins walk', 'manual', 2),
				($1, 'Healthcare', 'Aster RV Hospital', '4.2 km', '10 mins drive', 'manual', 3),
				($1, 'Education', 'Delhi Public School South', '3.5 km', '8 mins drive', 'manual', 4),
				($1, 'Connectivity', 'NICE Ring Road Junction', '2.8 km', '6 mins drive', 'manual', 5)
				"#,
				pid
			)
			.execute(&pool)
			.await
			.expect("failed to seed project landmarks");

			println!("[seed] project, configurations, specifications, and landmarks seeded");
			pid
		}
	};

	let _ = project_id;
	println!("[seed] database initialization complete");
}

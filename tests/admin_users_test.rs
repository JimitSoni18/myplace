use myplace::crypto::password_hash;

#[test]
fn test_admin_users_password_hashing_and_verification() {
	let raw_password = "SecureAdminPassword123!";

	// Hash password with Argon2
	let hash_result = password_hash::hash(raw_password);
	assert!(hash_result.is_ok(), "Password hashing should succeed");

	let hash = hash_result.ok().expect("Password hashing failed");
	assert!(!hash.is_empty(), "Hash string should not be empty");
	assert!(
		hash.starts_with("$argon2"),
		"Hash should use Argon2 algorithm"
	);

	// Verify correct password succeeds
	let verify_correct = password_hash::verify(raw_password, &hash);
	assert!(
		verify_correct.is_ok(),
		"Correct password verification failed"
	);

	// Verify incorrect password fails
	let verify_wrong = password_hash::verify("WrongPassword123!", &hash);
	assert!(
		verify_wrong.is_err(),
		"Wrong password verification should fail"
	);
}

#[test]
fn test_admin_users_password_hashing_distinct_salts() {
	let raw_password = "SamePasswordForTwoAdmins!";

	// Two admins with identical passwords should have distinct hashes due to random salting
	let hash1 = password_hash::hash(raw_password)
		.ok()
		.expect("hash 1 failed");
	let hash2 = password_hash::hash(raw_password)
		.ok()
		.expect("hash 2 failed");

	assert_ne!(
		hash1, hash2,
		"Argon2 should generate different salt for each user"
	);
	assert!(password_hash::verify(raw_password, &hash1).is_ok());
	assert!(password_hash::verify(raw_password, &hash2).is_ok());
}

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

#[tokio::test]
async fn test_two_admins_can_be_logged_in_simultaneously() {
	use myplace::session_store::{AuthUser, SessionStore, SessionStoreTrait, UserRole};

	let store = SessionStore::default();

	// Admin 1 logs in
	let user1 = AuthUser::new(1, "admin1".to_string(), UserRole::Admin);
	let session1 = store.create_session(user1).await;

	// Admin 2 logs in
	let user2 = AuthUser::new(2, "admin2".to_string(), UserRole::Admin);
	let session2 = store.create_session(user2).await;

	// Both session IDs must be distinct
	assert_ne!(session1, session2, "Session IDs must be distinct");

	// Admin 1 must still be logged in!
	let retrieved1 = store.get_session(&session1).await;
	assert!(
		retrieved1.is_some(),
		"Admin 1 was unexpectedly logged out after Admin 2 logged in!"
	);
	assert_eq!(retrieved1.unwrap().username, "admin1");

	// Admin 2 must also be logged in!
	let retrieved2 = store.get_session(&session2).await;
	assert!(
		retrieved2.is_some(),
		"Admin 2 was not found in session store!"
	);
	assert_eq!(retrieved2.unwrap().username, "admin2");

	// Even with 10 admins logging in successively, none should be logged out
	let mut sessions = vec![(session1, "admin1"), (session2, "admin2")];
	for i in 3..=10 {
		let username = format!("admin{i}");
		let user = AuthUser::new(i, username.clone(), UserRole::Admin);
		let sid = store.create_session(user).await;
		sessions.push((sid, Box::leak(username.into_boxed_str())));
	}

	for (sid, uname) in &sessions {
		let sess = store.get_session(sid).await;
		assert!(
			sess.is_some(),
			"Admin session for {} was prematurely expired",
			uname
		);
		assert_eq!(sess.unwrap().username, *uname);
	}
}



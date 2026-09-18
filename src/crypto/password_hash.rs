use std::sync::LazyLock;

use argon2::{
	Argon2,
	password_hash::{
		PasswordHash, PasswordHasher as _, PasswordVerifier as _, SaltString, rand_core::OsRng,
	},
};

pub enum HashingError {
	InternalError,
}

pub enum VerificationError {
	InvalidHash,
	ValidationFailed,
}

static ARGON2_INST: LazyLock<Argon2> = LazyLock::new(Argon2::default);

pub fn hash(plain_text_password: &str) -> Result<String, HashingError> {
	match ARGON2_INST.hash_password(plain_text_password.as_bytes(), &SaltString::generate(OsRng)) {
		Ok(v) => Ok(v.to_string()),
		_ => Err(HashingError::InternalError),
	}
}

pub fn verify(plain_text_password: &str, hashed_password: &str) -> Result<(), VerificationError> {
	let password_hash =
		PasswordHash::new(hashed_password).map_err(|_| VerificationError::InvalidHash)?;
	ARGON2_INST
		.verify_password(plain_text_password.as_bytes(), &password_hash)
		.or(Err(VerificationError::ValidationFailed))
}

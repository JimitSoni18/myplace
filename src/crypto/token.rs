use std::sync::LazyLock;

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
	// user id
	sub: Uuid,
	// aud: // TODO: admin or project owner...?
	iat: OffsetDateTime,
	exp: OffsetDateTime,
}

const TOKEN_DURATION: time::Duration = time::Duration::seconds(60 * 60 * 24);

struct TokenConfig {
	encoding_key: EncodingKey,
	decoding_key: DecodingKey,
	header: Header,
	validation: Validation,
}

const ALG: Algorithm = Algorithm::EdDSA;

fn load_key_pem(env_var: &str, file_name: &str) -> Vec<u8> {
	if let Ok(val) = std::env::var(env_var) {
		let trimmed = val.trim();
		if !trimmed.is_empty() {
			return trimmed.replace("\\n", "\n").into_bytes();
		}
	}
	if let Ok(bytes) = std::fs::read(file_name) {
		return bytes;
	}
	panic!(
		"Error: Could not load Ed25519 key for JWT tokens. \
		 Expected environment variable `{}` or file `{}` to exist.",
		env_var, file_name
	);
}

static TOKEN_CONFIG: LazyLock<TokenConfig> = LazyLock::new(|| {
	let private_pem = load_key_pem("JWT_PRIVATE_KEY", "private.pem");
	let public_pem = load_key_pem("JWT_PUBLIC_KEY", "public.pem");

	TokenConfig {
		encoding_key: EncodingKey::from_ed_pem(&private_pem)
			.expect("Failed to parse Ed25519 private key from JWT_PRIVATE_KEY or private.pem"),
		decoding_key: DecodingKey::from_ed_pem(&public_pem)
			.expect("Failed to parse Ed25519 public key from JWT_PUBLIC_KEY or public.pem"),
		header: Header::new(ALG),
		validation: Validation::new(ALG),
	}
});

pub enum EncodingError {
	UnknownReason,
}

pub enum DecodingError {
	UnknownReason,
}

pub fn encode(sub: Uuid) -> Result<String, EncodingError> {
	let TokenConfig {
		encoding_key,
		header,
		..
	} = &*TOKEN_CONFIG;

	let iat = OffsetDateTime::now_utc();
	let exp = iat.saturating_add(TOKEN_DURATION);

	let claims = Claims { sub, iat, exp };

	jsonwebtoken::encode(header, &claims, encoding_key).map_err(|_| EncodingError::UnknownReason)
}

pub fn decode(token: &str) -> Result<TokenData<Claims>, DecodingError> {
	let TokenConfig {
		decoding_key,
		validation,
		..
	} = &*TOKEN_CONFIG;
	jsonwebtoken::decode::<Claims>(token, decoding_key, validation)
		.or(Err(DecodingError::UnknownReason))
}

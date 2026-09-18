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

static TOKEN_CONFIG: LazyLock<TokenConfig> = LazyLock::new(|| TokenConfig {
	encoding_key: EncodingKey::from_ed_pem(include_bytes!("../../private.pem")).unwrap(),
	decoding_key: DecodingKey::from_ed_pem(include_bytes!("../../public.pem")).unwrap(),
	header: Header::new(ALG),
	validation: Validation::new(ALG),
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

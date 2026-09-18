use base64::{Engine as _, engine::general_purpose::STANDARD};
use hmac::Mac as _;
use uuid::Uuid;

use crate::config::CONFIG;

type BlakeMac = hmac::SimpleHmac<blake3::Hasher>;

fn create_mac(id: &Uuid) -> hmac::SimpleHmac<blake3::Hasher> {
	let mut mac = BlakeMac::new_from_slice(CONFIG.cookie_signing_secret.as_bytes()).unwrap();
	mac.update(id.as_bytes());
	mac
}

pub fn get_signature(id: &Uuid) -> String {
	STANDARD.encode(create_mac(id).finalize().into_bytes())
}

pub fn verify_cookie(id: &Uuid, sign: &str) -> bool {
	let mac = create_mac(id);
	let decoded_sign = STANDARD.decode(sign).unwrap();
	mac.verify_slice(&decoded_sign).is_ok()
}

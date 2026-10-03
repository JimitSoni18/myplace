use std::path::Path;
use std::process::Command;

fn main() {
	println!("cargo:rerun-if-changed=private.pem");
	println!("cargo:rerun-if-changed=public.pem");

	if !Path::new("private.pem").exists() || !Path::new("public.pem").exists() {
		println!("cargo:warning=private.pem or public.pem not found; generating new Ed25519 key pair...");

		if !Path::new("private.pem").exists() {
			let status = Command::new("openssl")
				.args(["genpkey", "-algorithm", "ed25519", "-out", "private.pem"])
				.status();
			if let Err(e) = status {
				println!("cargo:warning=Failed to execute openssl: {e}");
			}
		}

		if !Path::new("public.pem").exists() && Path::new("private.pem").exists() {
			let status = Command::new("openssl")
				.args(["pkey", "-in", "private.pem", "-pubout", "-out", "public.pem"])
				.status();
			if let Err(e) = status {
				println!("cargo:warning=Failed to derive public key: {e}");
			}
		}
	}
}

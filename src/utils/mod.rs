pub mod form;
pub mod helpers;
pub mod markdown;

pub mod sql {
	pub fn escape(input: &str) -> String {
		input
			.replace('\\', "\\\\")
			.replace('%', "\\%")
			.replace('_', "\\_")
	}

	pub fn slugify(input: &str) -> String {
		let mut was_last_hyphen = true;
		let mut slug: String = input
			.chars()
			.filter_map(|mut c| {
				c = match c {
					'À'..='Å' | 'à'..='å' => 'a',
					'È'..='Ë' | 'è'..='ë' => 'e',
					'Ì'..='Ï' | 'ì'..='ï' => 'i',
					'Ò'..='Ö' | 'Ø' | 'ò'..='ö' | 'ø' => 'o',
					'Ù'..='Ü' | 'ù'..='ü' => 'u',
					'Ñ' | 'ñ' => 'n',
					'Ç' | 'ç' => 'c',
					'Ý' | 'ý' | 'ÿ' | 'Ÿ' => 'y',
					_ => c.to_ascii_lowercase(),
				};
				if c.is_ascii_alphanumeric() {
					was_last_hyphen = false;
					return Some(c);
				} else if !was_last_hyphen {
					was_last_hyphen = true;
					return Some('-');
				}
				None
			})
			.collect();
		if slug.ends_with('-') {
			slug.pop();
		}

		slug
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[test]
		fn test_slugify_ascii_mixed_case() {
			let fx_mixed_str = "Héllö, World! This is a Test... ";
			assert_eq!(slugify(fx_mixed_str), "hello-world-this-is-a-test");
		}
	}
}

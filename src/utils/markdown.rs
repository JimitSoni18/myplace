use pulldown_cmark::{Options, Parser, html};

/// Converts Markdown text to sanitized, safe HTML.
/// Supports tables, strikethrough, and task lists, sanitized with Ammonia to prevent XSS.
pub fn render_markdown(input: &str) -> String {
	let mut options = Options::empty();
	options.insert(Options::ENABLE_TABLES);
	options.insert(Options::ENABLE_STRIKETHROUGH);
	options.insert(Options::ENABLE_TASKLISTS);

	let parser = Parser::new_ext(input, options);
	let mut html_output = String::new();
	html::push_html(&mut html_output, parser);

	ammonia::clean(&html_output)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_renders_markdown_safely() {
		let input = "# Heading\n\n**Bold text** and [link](https://example.com)\n\n<script>alert('xss')</script>";
		let html = render_markdown(input);
		assert!(html.contains("<h1>Heading</h1>"));
		assert!(html.contains("<strong>Bold text</strong>"));
		assert!(html.contains("href=\"https://example.com\""));
		assert!(!html.contains("<script>"));
	}
}

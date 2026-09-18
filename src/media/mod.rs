use aws_sdk_s3::primitives::ByteStream;
use image::ImageReader;
use sqlx::PgPool;
use uuid::Uuid;
use webp::Encoder;

use crate::config::CONFIG;

#[derive(Debug)]
pub enum MediaError {
	ImageFormatNotSupported,
	ImageDecodeError(String),
	EncodingError(String),
	S3UploadError(String),
	DatabaseError(sqlx::Error),
	Internal(String),
}

impl std::fmt::Display for MediaError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::ImageFormatNotSupported => write!(f, "Image format not supported"),
			Self::ImageDecodeError(msg) => write!(f, "Image decode failed: {msg}"),
			Self::EncodingError(msg) => write!(f, "WebP encode failed: {msg}"),
			Self::S3UploadError(msg) => write!(f, "S3 upload failed: {msg}"),
			Self::DatabaseError(err) => write!(f, "Database error: {err}"),
			Self::Internal(msg) => write!(f, "Internal error: {msg}"),
		}
	}
}

impl std::error::Error for MediaError {}

pub struct UploadedImageResult {
	pub media_id: Uuid,
	pub original_key: String,
	pub thumbnail_key: String,
	pub original_url: String,
	pub thumbnail_url: String,
	pub width: i32,
	pub height: i32,
	pub file_size: i64,
}

/// Encodes raw image bytes into original WebP and thumbnail WebP,
/// uploads both to S3, and records the entry in the `media` table.
pub async fn process_and_upload_image(
	db: &PgPool,
	s3_client: &aws_sdk_s3::Client,
	bytes: Vec<u8>,
) -> Result<UploadedImageResult, MediaError> {
	let media_id = Uuid::now_v7();

	// Offload CPU-heavy image decode and WebP compression
	let (image_data, thumb_data, width, height) = tokio::task::spawn_blocking(move || {
		let reader = ImageReader::new(std::io::Cursor::new(bytes))
			.with_guessed_format()
			.map_err(|_| MediaError::ImageFormatNotSupported)?;

		let img = reader
			.decode()
			.map_err(|e| MediaError::ImageDecodeError(e.to_string()))?;

		let width = img.width() as i32;
		let height = img.height() as i32;

		let webp = Encoder::from_image(&img)
			.map_err(|e| MediaError::EncodingError(e.to_string()))?
			.encode(82.0);

		let thumb_img = img.thumbnail(400, 400);
		let thumb_webp = Encoder::from_image(&thumb_img)
			.map_err(|e| MediaError::EncodingError(e.to_string()))?
			.encode(80.0);

		Ok::<_, MediaError>((webp.to_vec(), thumb_webp.to_vec(), width, height))
	})
	.await
	.map_err(|e| MediaError::Internal(e.to_string()))??;

	let original_key = format!("media/{media_id}/original.webp");
	let thumbnail_key = format!("media/{media_id}/thumbnail.webp");
	let file_size = image_data.len() as i64;

	// Upload original
	s3_client
		.put_object()
		.bucket(&CONFIG.s3_bucket)
		.key(&original_key)
		.content_type("image/webp")
		.body(ByteStream::from(image_data))
		.send()
		.await
		.map_err(|e| MediaError::S3UploadError(e.to_string()))?;

	// Upload thumbnail
	s3_client
		.put_object()
		.bucket(&CONFIG.s3_bucket)
		.key(&thumbnail_key)
		.content_type("image/webp")
		.body(ByteStream::from(thumb_data))
		.send()
		.await
		.map_err(|e| MediaError::S3UploadError(e.to_string()))?;

	// Save to DB
	sqlx::query!(
		r#"
		INSERT INTO media (
			id, media_type, mime_type, s3_key, thumbnail_key,
			file_size, width, height
		)
		VALUES ($1, 'image', 'image/webp', $2, $3, $4, $5, $6)
		"#,
		media_id,
		original_key,
		thumbnail_key,
		file_size,
		width,
		height,
	)
	.execute(db)
	.await
	.map_err(MediaError::DatabaseError)?;

	let original_url = CONFIG.asset_url(&original_key);
	let thumbnail_url = CONFIG.asset_url(&thumbnail_key);

	tracing::info!(%media_id, width, height, "uploaded and recorded media image");

	Ok(UploadedImageResult {
		media_id,
		original_key,
		thumbnail_key,
		original_url,
		thumbnail_url,
		width,
		height,
		file_size,
	})
}

pub struct UploadedDocumentResult {
	pub media_id: Uuid,
	pub s3_key: String,
	pub url: String,
	pub file_size: i64,
}

/// Uploads a document (PDF, brochure, approval) to S3 and records it in `media`.
pub async fn upload_document(
	db: &PgPool,
	s3_client: &aws_sdk_s3::Client,
	filename: &str,
	mime_type: &str,
	bytes: Vec<u8>,
) -> Result<UploadedDocumentResult, MediaError> {
	let media_id = Uuid::now_v7();
	let s3_key = format!("documents/{media_id}/{filename}");
	let file_size = bytes.len() as i64;

	s3_client
		.put_object()
		.bucket(&CONFIG.s3_bucket)
		.key(&s3_key)
		.content_type(mime_type)
		.body(ByteStream::from(bytes))
		.send()
		.await
		.map_err(|e| MediaError::S3UploadError(e.to_string()))?;

	sqlx::query!(
		r#"
		INSERT INTO media (id, media_type, mime_type, s3_key, file_size)
		VALUES ($1, 'document', $2, $3, $4)
		"#,
		media_id,
		mime_type,
		s3_key,
		file_size,
	)
	.execute(db)
	.await
	.map_err(MediaError::DatabaseError)?;

	let url = CONFIG.asset_url(&s3_key);
	tracing::info!(%media_id, %filename, "uploaded document");

	Ok(UploadedDocumentResult {
		media_id,
		s3_key,
		url,
		file_size,
	})
}

/// Removes a media record and cleans up associated S3 objects.
pub async fn delete_media(
	db: &PgPool,
	s3_client: &aws_sdk_s3::Client,
	media_id: Uuid,
) -> Result<(), MediaError> {
	let row = sqlx::query!(
		"SELECT s3_key, thumbnail_key FROM media WHERE id = $1",
		media_id
	)
	.fetch_optional(db)
	.await
	.map_err(MediaError::DatabaseError)?;

	if let Some(r) = row {
		let _ = s3_client
			.delete_object()
			.bucket(&CONFIG.s3_bucket)
			.key(&r.s3_key)
			.send()
			.await;

		if let Some(thumb) = r.thumbnail_key {
			let _ = s3_client
				.delete_object()
				.bucket(&CONFIG.s3_bucket)
				.key(&thumb)
				.send()
				.await;
		}

		sqlx::query!("DELETE FROM media WHERE id = $1", media_id)
			.execute(db)
			.await
			.map_err(MediaError::DatabaseError)?;

		tracing::info!(%media_id, "deleted media and S3 objects");
	}

	Ok(())
}

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
	VideoProcessingError(String),
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
			Self::VideoProcessingError(msg) => write!(f, "Video processing failed: {msg}"),
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
	sqlx::query_file!(
		"queries/media/insert_image_media.sql",
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

#[derive(Debug, Clone)]
pub struct UploadedVideoResult {
	pub media_id: Uuid,
	pub s3_key: String,
	pub thumbnail_key: Option<String>,
	pub video_url: String,
	pub thumbnail_url: Option<String>,
	pub width: Option<i32>,
	pub height: Option<i32>,
	pub duration_secs: Option<i32>,
	pub file_size: i64,
}

#[derive(serde::Deserialize)]
struct ProbeOutput {
	streams: Option<Vec<ProbeStream>>,
	format: Option<ProbeFormat>,
}

#[derive(serde::Deserialize)]
struct ProbeStream {
	width: Option<i32>,
	height: Option<i32>,
}

#[derive(serde::Deserialize)]
struct ProbeFormat {
	duration: Option<String>,
}

struct TempDirGuard(std::path::PathBuf);

impl Drop for TempDirGuard {
	fn drop(&mut self) {
		let _ = std::fs::remove_dir_all(&self.0);
	}
}

/// Process video bytes using ffmpeg / ffprobe, extract dimensions and duration,
/// extract a poster thumbnail (WebP), upload both to S3, and record in `media`.
/// In case of failure, temporary files and any uploaded S3 objects are cleaned up.
pub async fn process_and_upload_video(
	db: &PgPool,
	s3_client: &aws_sdk_s3::Client,
	bytes: Vec<u8>,
	filename: &str,
	content_type: Option<&str>,
) -> Result<UploadedVideoResult, MediaError> {
	let media_id = Uuid::now_v7();
	let temp_dir = std::env::temp_dir().join(format!("myplace_vid_{media_id}"));
	tokio::fs::create_dir_all(&temp_dir)
		.await
		.map_err(|e| MediaError::Internal(e.to_string()))?;
	let _guard = TempDirGuard(temp_dir.clone());

	let safe_filename = if filename.is_empty() {
		"video.mp4"
	} else {
		filename
	};
	let in_path = temp_dir.join(safe_filename);
	tokio::fs::write(&in_path, &bytes)
		.await
		.map_err(|e| MediaError::Internal(e.to_string()))?;

	let in_path_str = in_path
		.to_str()
		.ok_or_else(|| MediaError::Internal("Invalid temp path".to_string()))?;

	// 1. Probe video metadata with ffprobe
	let mut width = None;
	let mut height = None;
	let mut duration_secs = None;
	let mut probe_succeeded = false;

	if let Ok(probe_out) = tokio::process::Command::new("ffprobe")
		.args([
			"-v",
			"error",
			"-show_entries",
			"format=duration:stream=width,height",
			"-of",
			"json",
			in_path_str,
		])
		.output()
		.await
	{
		if probe_out.status.success() {
			if let Ok(parsed) = serde_json::from_slice::<ProbeOutput>(&probe_out.stdout) {
				if let Some(streams) = parsed.streams {
					if let Some(first) = streams.first() {
						width = first.width;
						height = first.height;
						if width.is_some() || height.is_some() {
							probe_succeeded = true;
						}
					}
				}
				if let Some(fmt) = parsed.format {
					if let Some(dur_str) = fmt.duration {
						if let Ok(dur_f) = dur_str.parse::<f64>() {
							duration_secs = Some(dur_f.round() as i32);
							probe_succeeded = true;
						}
					}
				}
			}
		}
	}

	// 2. Extract poster thumbnail at 0.5s or fallback to 0.0s
	let thumb_path = temp_dir.join("thumbnail.webp");
	let mut thumb_bytes = match tokio::process::Command::new("ffmpeg")
		.args([
			"-ss",
			"00:00:00.500",
			"-i",
			in_path_str,
			"-vframes",
			"1",
			"-vf",
			"scale=400:-1",
			"-c:v",
			"libwebp",
			thumb_path.to_str().unwrap(),
			"-y",
		])
		.output()
		.await
	{
		Ok(out) if out.status.success() => tokio::fs::read(&thumb_path).await.ok(),
		_ => None,
	};

	if thumb_bytes.is_none() {
		thumb_bytes = match tokio::process::Command::new("ffmpeg")
			.args([
				"-ss",
				"00:00:00.000",
				"-i",
				in_path_str,
				"-vframes",
				"1",
				"-vf",
				"scale=400:-1",
				"-c:v",
				"libwebp",
				thumb_path.to_str().unwrap(),
				"-y",
			])
			.output()
			.await
		{
			Ok(out) if out.status.success() => tokio::fs::read(&thumb_path).await.ok(),
			_ => None,
		};
	}

	if !probe_succeeded && thumb_bytes.is_none() {
		return Err(MediaError::VideoProcessingError(
			"Invalid or unsupported video file".to_string(),
		));
	}

	// 3. Optimize container with +faststart for streaming (with timeout to avoid blocking)
	let opt_path = temp_dir.join("faststart.mp4");
	let final_video_bytes = match tokio::time::timeout(
		std::time::Duration::from_secs(15),
		tokio::process::Command::new("ffmpeg")
			.args([
				"-i",
				in_path_str,
				"-c:v",
				"copy",
				"-c:a",
				"copy",
				"-movflags",
				"+faststart",
				opt_path.to_str().unwrap(),
				"-y",
			])
			.output(),
	)
	.await
	{
		Ok(Ok(out)) if out.status.success() => tokio::fs::read(&opt_path)
			.await
			.unwrap_or_else(|_| bytes.clone()),
		_ => bytes,
	};

	let mime_type = content_type.unwrap_or("video/mp4");
	let video_key = format!("media/{media_id}/video.mp4");
	let thumbnail_key = thumb_bytes
		.as_ref()
		.map(|_| format!("media/{media_id}/thumbnail.webp"));
	let file_size = final_video_bytes.len() as i64;

	// 4. Upload video to S3
	let upload_video_res = s3_client
		.put_object()
		.bucket(&CONFIG.s3_bucket)
		.key(&video_key)
		.content_type(mime_type)
		.body(ByteStream::from(final_video_bytes))
		.send()
		.await;

	if let Err(e) = upload_video_res {
		return Err(MediaError::S3UploadError(e.to_string()));
	}

	// 5. Upload thumbnail to S3 if generated
	if let (Some(tb), Some(tk)) = (&thumb_bytes, &thumbnail_key) {
		let upload_thumb_res = s3_client
			.put_object()
			.bucket(&CONFIG.s3_bucket)
			.key(tk)
			.content_type("image/webp")
			.body(ByteStream::from(tb.clone()))
			.send()
			.await;

		if let Err(e) = upload_thumb_res {
			let _ = s3_client
				.delete_object()
				.bucket(&CONFIG.s3_bucket)
				.key(&video_key)
				.send()
				.await;
			return Err(MediaError::S3UploadError(e.to_string()));
		}
	}

	// 6. Record in DB
	let db_res = sqlx::query_file!(
		"queries/media/insert_video_media.sql",
		media_id,
		mime_type,
		video_key,
		thumbnail_key.as_deref(),
		file_size,
		width,
		height,
		duration_secs,
	)
	.execute(db)
	.await;

	if let Err(e) = db_res {
		// Clean up uploaded S3 objects on DB failure
		let _ = s3_client
			.delete_object()
			.bucket(&CONFIG.s3_bucket)
			.key(&video_key)
			.send()
			.await;
		if let Some(ref tk) = thumbnail_key {
			let _ = s3_client
				.delete_object()
				.bucket(&CONFIG.s3_bucket)
				.key(tk)
				.send()
				.await;
		}
		return Err(MediaError::DatabaseError(e));
	}

	let video_url = CONFIG.asset_url(&video_key);
	let thumbnail_url = thumbnail_key.as_ref().map(|k| CONFIG.asset_url(k));

	tracing::info!(%media_id, width, height, duration_secs, "uploaded and recorded media video");

	Ok(UploadedVideoResult {
		media_id,
		s3_key: video_key,
		thumbnail_key,
		video_url,
		thumbnail_url,
		width,
		height,
		duration_secs,
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

	sqlx::query_file!(
		"queries/media/insert_document_media.sql",
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
	let row = sqlx::query_file!("queries/media/get_media_keys.sql", media_id)
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

		sqlx::query_file!("queries/media/delete_media.sql", media_id)
			.execute(db)
			.await
			.map_err(MediaError::DatabaseError)?;

		tracing::info!(%media_id, "deleted media and S3 objects");
	}

	Ok(())
}

pub fn is_video_upload(
	field_name: &str,
	content_type: Option<&str>,
	filename: Option<&str>,
) -> bool {
	if field_name == "video" {
		return true;
	}
	if let Some(ct) = content_type {
		if ct.starts_with("video/") {
			return true;
		}
	}
	if let Some(name) = filename {
		let lower = name.to_lowercase();
		if lower.ends_with(".mp4")
			|| lower.ends_with(".mov")
			|| lower.ends_with(".webm")
			|| lower.ends_with(".mkv")
			|| lower.ends_with(".avi")
		{
			return true;
		}
	}
	false
}

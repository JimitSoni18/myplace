use aws_sdk_s3::{
	Client,
	types::{Delete, ObjectIdentifier},
};

pub async fn delete_all_with_prefix(
	client: &Client,
	bucket: &str,
	prefix: &str,
) -> Result<(), aws_sdk_s3::Error> {
	// 1. Prepare the paginator to scan objects matching the prefix
	let mut response_stream = client
		.list_objects_v2()
		.bucket(bucket)
		.prefix(prefix)
		.into_paginator()
		.send();

	// 2. Stream through pages of contents
	while let Some(page) = response_stream.next().await {
		let page = page?;

		if let Some(objects) = page.contents {
			// Build the block of objects to delete (Max 1,000 per S3 API spec)
			let mut object_ids = Vec::new();

			for object in objects {
				if let Some(key) = object.key {
					let obj_id = ObjectIdentifier::builder().key(key).build().unwrap();
					object_ids.push(obj_id);
				}
			}

			// 3. Perform batch deletion if files were found on this page
			if !object_ids.is_empty() {
				let delete_payload = Delete::builder()
					.set_objects(Some(object_ids))
					.build()
					.unwrap();

				client
					.delete_objects()
					.bucket(bucket)
					.delete(delete_payload)
					.send()
					.await?;
			}
		}
	}

	Ok(())
}

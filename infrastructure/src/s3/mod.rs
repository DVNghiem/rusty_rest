use std::env::set_var;

use super::config::Config;
use aws_config;
use aws_sdk_s3::{
    error::SdkError,
    operation::{
        complete_multipart_upload::{CompleteMultipartUploadError, CompleteMultipartUploadOutput}, create_bucket::{CreateBucketError, CreateBucketOutput}, get_object::{GetObjectError, GetObjectOutput}, put_object::{PutObjectError, PutObjectOutput}
    },
    primitives::ByteStream,
    types::{CompletedMultipartUpload, CompletedPart},
    Client,
};
use aws_smithy_types::byte_stream::Length;
use tokio::fs::File;

//In bytes, minimum chunk size of 5MB. Increase CHUNK_SIZE to send larger chunks.
const CHUNK_SIZE: u64 = 1024 * 1024 * 5;
const MAX_CHUNKS: u64 = 10000;

pub struct AmazonS3Client {
    s3_client: Client,
}

impl AmazonS3Client {
    pub async fn load_s3_client() -> Self {
        let config = Config::new();
        set_var("AWS_ACCESS_KEY_ID", config.aws_access_key_id);
        set_var("AWS_SECRET_ACCESS_KEY", config.aws_secret_access_key);
        set_var("AWS_REGION", config.aws_region);
        set_var("AWS_BUCKET", config.aws_bucket);

        let s3_config = aws_config::load_from_env().await;
        let s3_client = Client::new(&s3_config);
        Self { s3_client }
    }

    pub async fn create_bucket(
        &self,
        bucket_name: &str,
    ) -> Result<CreateBucketOutput, SdkError<CreateBucketError>> {
        let mut create_request = self.s3_client.create_bucket();
        create_request = create_request.bucket(bucket_name);
        create_request.send().await
    }

    pub async fn upload(
        &self,
        key: &str,
        bucket_name: &str,
        body: Vec<u8>,
    ) -> Result<PutObjectOutput, SdkError<PutObjectError>> {
        let mut upload_request = self.s3_client.put_object();
        let upload_data = ByteStream::from(body);
        upload_request = upload_request.bucket(bucket_name);
        upload_request = upload_request.key(key);
        upload_request = upload_request.body(upload_data);
        upload_request.send().await
    }

    pub async fn download(
        &self,
        key: &str,
        bucket_name: &str,
    ) -> Result<GetObjectOutput, SdkError<GetObjectError>> {
        let mut download_request = self.s3_client.get_object();
        download_request = download_request.bucket(bucket_name);
        download_request = download_request.key(key);
        let response = download_request.send().await;
        response
    }

    pub async fn upload_large_file(
        &self,
        key: &str,
        bucket_name: &str,
        file: File,
    ) -> Result<CompleteMultipartUploadOutput, SdkError<CompleteMultipartUploadError>> {
        let mut_upload_request = self
            .s3_client
            .create_multipart_upload()
            .bucket(bucket_name)
            .key(key)
            .send()
            .await
            .unwrap();
        let upload_id = mut_upload_request.upload_id().unwrap();
        let file_size = file.metadata().await.unwrap().len() as u64;
        let mut chunk_count = (file_size / CHUNK_SIZE) + 1;
        let mut size_of_last_chunk = file_size % CHUNK_SIZE;
        if size_of_last_chunk == 0 {
            size_of_last_chunk = CHUNK_SIZE;
            chunk_count -= 1;
        }

        if file_size == 0 {
            panic!("Bad file size.");
        }
        if chunk_count > MAX_CHUNKS {
            panic!("Too many chunks! Try increasing your chunk size.")
        }

        let mut upload_parts: Vec<CompletedPart> = Vec::new();

        for chunk_index in 0..chunk_count {
            let this_chunk = if chunk_count - 1 == chunk_index {
                size_of_last_chunk
            } else {
                CHUNK_SIZE
            };
            let stream = ByteStream::read_from()
                .file(file.try_clone().await.unwrap())
                .offset(chunk_index * CHUNK_SIZE)
                .length(Length::Exact(this_chunk))
                .build()
                .await
                .unwrap();
            //Chunk index needs to start at 0, but part numbers start at 1.
            let part_number = (chunk_index as i32) + 1;
            let upload_part_res = self
                .s3_client
                .upload_part()
                .key(&*key)
                .bucket(&*bucket_name)
                .upload_id(upload_id)
                .body(stream)
                .part_number(part_number)
                .send()
                .await
                .unwrap();
            upload_parts.push(
                CompletedPart::builder()
                    .e_tag(upload_part_res.e_tag.unwrap_or_default())
                    .part_number(part_number)
                    .build(),
            );
        }

        let completed_multipart_upload: CompletedMultipartUpload =
            CompletedMultipartUpload::builder()
                .set_parts(Some(upload_parts))
                .build();

        self
            .s3_client
            .complete_multipart_upload()
            .bucket(&*bucket_name)
            .key(&*key)
            .multipart_upload(completed_multipart_upload)
            .upload_id(upload_id)
            .send()
            .await

    }
}

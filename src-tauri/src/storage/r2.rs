use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{Client, Error as S3Error};
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use serde::{Deserialize, Serialize};
use std::io::Write;
use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum R2Error {
    #[error("AWS SDK error: {0}")]
    AwsError(#[from] S3Error),
    
    #[error("Failed to read file: {0}")]
    FileReadError(String),
    
    #[error("Failed to write file: {0}")]
    FileWriteError(String),
    
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Bucket does not exist: {0}")]
    BucketNotFound(String),
    
    #[error("Other error: {0}")]
    Other(String),
}

type R2Result<T> = std::result::Result<T, R2Error>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct R2Credentials {
    pub bucket_name: String,
    pub access_key: String,
    pub secret_key: String,
    pub endpoint: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct R2ConnectionResult {
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct R2UploadResult {
    pub success: bool,
    pub path: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct R2DownloadResult {
    pub success: bool,
    pub data: Option<Vec<u8>>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct R2DeleteResult {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct R2Client {
    client: Client,
    bucket_name: String,
}

impl R2Client {
    pub async fn test_connection(&self) -> R2ConnectionResult {
        match self.client.list_objects_v2().bucket(&self.bucket_name).send().await {
            Ok(_) => R2ConnectionResult {
                success: true,
                message: Some("Successfully connected to R2 bucket".to_string()),
            },
            Err(err) => R2ConnectionResult {
                success: false,
                message: Some(format!("Failed to connect to R2 bucket: {}", err)),
            },
        }
    }
}

#[tauri::command]
pub async fn initialize_r2_client(credentials: R2Credentials) -> Result<R2Client, String> {
    let creds = Credentials::new(
        &credentials.access_key,
        &credentials.secret_key,
        None,
        None,
        "R2Credentials",
    );

    let region_provider = RegionProviderChain::default_provider()
        .or_else(Region::new("auto"));

    let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(region_provider)
        .endpoint_url(&credentials.endpoint)
        .credentials_provider(creds)
        .load()
        .await;

    let s3_config = aws_sdk_s3::config::Builder::from(&config)
        .force_path_style(true)
        .build();

    let client = Client::from_conf(s3_config);

    Ok(R2Client {
        client,
        bucket_name: credentials.bucket_name,
    })
}

#[tauri::command]
pub async fn upload_file(
    r2_client: R2Client,
    file_name: String,
    data: Vec<u8>,
    content_type: String,
) -> R2UploadResult {
    let stream = ByteStream::from(data);

    match r2_client.client
        .put_object()
        .bucket(&r2_client.bucket_name)
        .key(&file_name)
        .body(stream)
        .content_type(content_type)
        .send()
        .await 
    {
        Ok(_) => R2UploadResult {
            success: true,
            path: Some(format!("{}/{}", r2_client.bucket_name, file_name)),
            error: None,
        },
        Err(e) => R2UploadResult {
            success: false,
            path: None,
            error: Some(format!("Failed to upload file: {}", e)),
        },
    }
}

#[tauri::command]
pub async fn download_file(
    r2_client: R2Client,
    file_name: String,
) -> R2DownloadResult {
    match r2_client.client
        .get_object()
        .bucket(&r2_client.bucket_name)
        .key(&file_name)
        .send()
        .await 
    {
        Ok(resp) => {
            match resp.body.collect().await {
                Ok(bytes) => R2DownloadResult {
                    success: true,
                    data: Some(bytes.to_vec()),
                    error: None,
                },
                Err(e) => R2DownloadResult {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to read file body: {}", e)),
                },
            }
        },
        Err(e) => R2DownloadResult {
            success: false,
            data: None,
            error: Some(format!("Failed to download file: {}", e)),
        },
    }
}

#[tauri::command]
pub async fn delete_file(
    r2_client: R2Client,
    file_name: String,
) -> R2DeleteResult {
    match r2_client.client
        .delete_object()
        .bucket(&r2_client.bucket_name)
        .key(&file_name)
        .send()
        .await 
    {
        Ok(_) => R2DeleteResult {
            success: true,
            error: None,
        },
        Err(e) => R2DeleteResult {
            success: false,
            error: Some(format!("Failed to delete file: {}", e)),
        },
    }
}

// Helper function to upload a file from a file path
#[tauri::command]
pub async fn upload_file_from_path(
    r2_client: R2Client,
    file_path: String,
    r2_path: String,
    content_type: String,
) -> R2UploadResult {
    match std::fs::read(&file_path) {
        Ok(data) => {
            upload_file(r2_client, r2_path, data, content_type).await
        },
        Err(e) => R2UploadResult {
            success: false,
            path: None,
            error: Some(format!("Failed to read file {}: {}", file_path, e)),
        },
    }
}

// Helper function to download a file to a file path
#[tauri::command]
pub async fn download_file_to_path(
    r2_client: R2Client,
    r2_path: String,
    file_path: String,
) -> R2DeleteResult {
    let download_result = download_file(r2_client, r2_path).await;
    
    if !download_result.success {
        return R2DeleteResult {
            success: false,
            error: download_result.error,
        };
    }
    
    let data = match download_result.data {
        Some(data) => data,
        None => {
            return R2DeleteResult {
                success: false,
                error: Some("Downloaded file data is empty".to_string()),
            }
        }
    };
    
    match std::fs::File::create(&file_path) {
        Ok(mut file) => {
            match file.write_all(&data) {
                Ok(_) => R2DeleteResult {
                    success: true,
                    error: None,
                },
                Err(e) => R2DeleteResult {
                    success: false,
                    error: Some(format!("Failed to write to file {}: {}", file_path, e)),
                },
            }
        },
        Err(e) => R2DeleteResult {
            success: false,
            error: Some(format!("Failed to create file {}: {}", file_path, e)),
        },
    }
} 
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{Client, types::{Delete, ObjectIdentifier}};
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use serde::{Deserialize, Serialize};
use std::io::Write;
use thiserror::Error;
use futures_util::StreamExt;

#[derive(Debug, Error)]
pub enum R2Error {
    #[error("AWS SDK error: {0}")]
    AwsError(String),
    
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
    pub account_id: String,
    pub bucket_name: String,
    pub access_key_id: String,
    pub secret_access_key: String,
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
    /// Creates a new R2Client wrapper.
    pub fn new(client: Client, bucket_name: String) -> Self {
        Self { client, bucket_name }
    }

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
    
    /// List all objects in the bucket
    pub async fn list_objects(&self) -> R2Result<Vec<String>> {
        let resp = self.client.list_objects_v2()
            .bucket(&self.bucket_name)
            .send()
            .await
            .map_err(|e| R2Error::AwsError(e.to_string()))?;
            
        let mut keys = Vec::new();
        
        if let Some(contents) = resp.contents {
            for object in contents {
                if let Some(key) = object.key {
                    keys.push(key);
                }
            }
        }
        
        Ok(keys)
    }
    
    /// Upload data to the bucket
    pub async fn upload_object(&self, key: &str, data: Vec<u8>, content_type: &str) -> R2Result<()> {
        let stream = ByteStream::from(data);
        
        self.client.put_object()
            .bucket(&self.bucket_name)
            .key(key)
            .body(stream)
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| R2Error::AwsError(e.to_string()))?;
            
        Ok(())
    }
    
    /// Download an object from the bucket
    pub async fn download_object(&self, key: &str) -> R2Result<Vec<u8>> {
        let resp = self.client.get_object()
            .bucket(&self.bucket_name)
            .key(key)
            .send()
            .await
            .map_err(|e| R2Error::AwsError(e.to_string()))?;
            
        let bytes = resp.body.collect().await
            .map_err(|e| R2Error::Other(format!("Failed to read object body: {}", e)))?;
            
        Ok(bytes.to_vec())
    }
    
    /// Delete an object from the bucket
    pub async fn delete_object(&self, key: &str) -> R2Result<()> {
        self.client.delete_object()
            .bucket(&self.bucket_name)
            .key(key)
            .send()
            .await
            .map_err(|e| R2Error::AwsError(e.to_string()))?;
            
        Ok(())
    }
    
    /// Delete multiple objects from the bucket
    pub async fn delete_objects(&self, keys: &[String]) -> R2Result<()> {
        if keys.is_empty() {
            return Ok(());
        }
        
        let objects: Vec<ObjectIdentifier> = keys.iter()
            .map(|key| {
                ObjectIdentifier::builder()
                    .key(key)
                    .build()
                    .map_err(|e| R2Error::AwsError(e.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;
            
        let delete = Delete::builder()
            .set_objects(Some(objects))
            .build()
            .map_err(|e| R2Error::AwsError(e.to_string()))?;
            
        self.client.delete_objects()
            .bucket(&self.bucket_name)
            .delete(delete)
            .send()
            .await
            .map_err(|e| R2Error::AwsError(e.to_string()))?;
            
        Ok(())
    }
    
    /// Check if an object exists
    pub async fn object_exists(&self, key: &str) -> R2Result<bool> {
        match self.client.head_object()
            .bucket(&self.bucket_name)
            .key(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(err) => {
                // Check if it's a "not found" error
                if err.to_string().contains("404") {
                    Ok(false)
                } else {
                    Err(R2Error::AwsError(err.to_string()))
                }
            }
        }
    }
}

#[tauri::command]
pub async fn initialize_r2_client(credentials: R2Credentials) -> Result<R2Client, String> {
    let creds = Credentials::new(
        &credentials.access_key_id,
        &credentials.secret_access_key,
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

/// Deletes multiple files from the R2 bucket based on their keys.
pub async fn delete_files(r2_client: &R2Client, file_keys: &[String]) -> Result<(), R2Error> {
    if file_keys.is_empty() {
        log::info!("No file keys provided for deletion.");
        return Ok(());
    }

    log::info!("Attempting to delete {} files from R2: {:?}", file_keys.len(), file_keys);

    // Convert file keys to ObjectIdentifiers
    let objects_to_delete: Vec<ObjectIdentifier> = file_keys.iter()
        .map(|key| ObjectIdentifier::builder().key(key).build())
        .collect::<Result<Vec<_>, _>>() // Collect into Result to handle potential build errors
        .map_err(|e| R2Error::Other(format!("Failed to build object identifiers: {}", e)))?;

    // Build the Delete request structure
    let delete_request = Delete::builder()
        .set_objects(Some(objects_to_delete))
        // .quiet(false) // Set to true if you don't need the list of deleted objects in the response
        .build()
        .map_err(|e| R2Error::Other(format!("Failed to build delete request: {}", e)))?;


    match r2_client.client
        .delete_objects()
        .bucket(&r2_client.bucket_name)
        .delete(delete_request)
        .send()
        .await
    {
        Ok(output) => {
            // Check if the Option<&[DeletedObject]> contains a non-empty slice
            if let Some(deleted_objects) = output.deleted { // Access the inner field directly if it's Option<Vec<T>> or handle Option<&[T]>
                 if !deleted_objects.is_empty() {
                    log::info!("Successfully deleted {} objects from R2.", deleted_objects.len());
                    // Optionally log the keys of deleted objects:
                    // for deleted in deleted_objects {
                    //     log::debug!("Deleted: {}", deleted.key().unwrap_or("Unknown key"));
                    // }
                 } else {
                     log::info!("DeleteObjects call successful, but the 'deleted' list was empty.");
                 }
            } else {
                 log::info!("DeleteObjects call successful, but no 'deleted' information returned.");
            }

            // Check if the Option<&[Error]> contains a non-empty slice
            if let Some(errors) = output.errors { // Access the inner field directly if it's Option<Vec<T>> or handle Option<&[T]>
                 if !errors.is_empty() {
                    log::error!("Errors occurred during R2 delete_objects operation:");
                    for error in errors {
                        log::error!("  Key: {}, Code: {}, Message: {}",
                            error.key().unwrap_or("Unknown key"),
                            error.code().unwrap_or("Unknown code"),
                            error.message().unwrap_or("No message"));
                    }
                    // Decide if partial failure should return an error
                    // For now, we log errors but return Ok if the call itself succeeded.
                    // return Err(R2Error::Other(format!("{} errors occurred during deletion.", errors.len())));
                 }
            }
            Ok(())
        },
        Err(e) => {
            log::error!("Failed to execute delete_objects request: {}", e);
            // Convert the SDK error into our custom R2Error::AwsError
            // The specific error type might be complex, using format! for simplicity here
            Err(R2Error::AwsError(e.to_string()))
        }
    }
}
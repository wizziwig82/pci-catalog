use mongodb::{Client, options::ClientOptions};
use tokio::sync::Mutex; // Add Mutex import
use aws_sdk_s3; // Add aws_sdk_s3 import

pub mod features;
pub mod core; // Declare core module here
pub use features::credentials::*;
pub use core::r2::R2Client; // Re-export R2Client for easier access

pub mod error; // Declare error module here
pub use error::CommandError; // Re-export CommandError for easier access

// Add re-exports for features module
pub mod feature_exports {
    pub use crate::error::CommandError;
}

pub mod mongo {
    use super::*;
    use crate::error::CommandError;
    
    pub async fn init(connection_string: &str) -> Result<Client, CommandError> {
        let client_options = ClientOptions::parse(connection_string).await
            .map_err(|e| CommandError::Database(format!("Failed to parse MongoDB connection string: {}", e)))?;
        
        let client = Client::with_options(client_options)
            .map_err(|e| CommandError::Database(format!("Failed to create MongoDB client: {}", e)))?;
        
        // Ping the server to see if we can connect
        client.database("admin").run_command(mongodb::bson::doc! {"ping": 1}, None).await
            .map_err(|e| CommandError::Database(format!("Failed to ping MongoDB server: {}", e)))?;
        
        Ok(client)
    }
}

pub mod r2 {
    use crate::error::CommandError;
    
    // Error module moved to top level
    pub async fn init(client: aws_sdk_s3::Client) -> Result<aws_sdk_s3::Client, CommandError> {
        // We're just passing through the client since it's already initialized
        // with the proper credentials
        Ok(client)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// --- State Structs Moved from main.rs ---

/// MongoDB client state
pub struct MongoState {
    // Make fields public if they need to be accessed directly from outside the lib crate,
    // otherwise keep them private and provide methods. Keeping private for now.
    pub client: Mutex<Option<mongodb::Client>>, // Make field public
}

/// R2 client state
pub struct R2State {
    pub client: Mutex<Option<aws_sdk_s3::Client>>, // Make field public
    pub bucket_name: Mutex<Option<String>>, // Make field public
}

// Re-export CredentialsError for easier access from main.rs
pub use features::credentials::CredentialsError;

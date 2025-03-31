use mongodb::{Client, options::ClientOptions};
use anyhow::Result;

pub mod mongo {
    use super::*;
    pub async fn init(connection_string: &str) -> Result<Client> {
        let client_options = ClientOptions::parse(connection_string).await?;
        let client = Client::with_options(client_options)?;
        
        // Ping the server to see if we can connect
        client.database("admin").run_command(mongodb::bson::doc! {"ping": 1}, None).await?;
        
        Ok(client)
    }
}

pub mod r2 {
    use anyhow::Result;
    
    pub async fn init(client: aws_sdk_s3::Client) -> Result<aws_sdk_s3::Client> {
        // We're just passing through the client since it's already initialized
        // with the proper credentials
        Ok(client)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

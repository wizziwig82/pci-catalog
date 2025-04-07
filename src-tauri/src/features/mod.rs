// src-tauri/src/features/mod.rs
pub mod catalog;
pub mod upload;
pub mod credentials;

// Import the CommandError type directly from the crate root
use crate::core::r2; // This is just to demonstrate that `crate` refers to app_lib
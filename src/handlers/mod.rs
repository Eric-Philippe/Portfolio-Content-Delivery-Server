//! Handlers module
//! 
//! This module contains all HTTP request handlers organized by functionality:
//! - `dev_projects` - Development project management endpoints
//! - `albums` - Photo album management endpoints  
//! - `files` - File upload and management endpoints

pub mod dev_projects;
pub mod albums;
pub mod files;

// Re-export all handler functions for easy access
pub use dev_projects::*;
pub use albums::*;
pub use files::*;

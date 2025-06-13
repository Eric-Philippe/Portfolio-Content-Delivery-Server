//! File Management Handlers
//! 
//! This module contains HTTP handlers for file operations including uploads and deletions.
//! It handles multipart file uploads, generates thumbnails for images, and manages folder operations.

use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::Json,
};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{error, info};
use utoipa;
use uuid::Uuid;

use crate::AppState;

/// Upload files to an album
///
/// Upload one or more files to a specific album. Files are automatically organized by album slug.
/// Thumbnails are generated for image files.
/// 
/// **Authentication Required**: This endpoint requires a valid API key in the `X-API-Key` header.
/// 
/// Required form fields:
/// - `slug`: Album identifier (string)
/// - `file`: File to upload (binary, can be multiple files)
/// 
/// Required headers:
/// - `X-API-Key`: Valid API key for authentication
#[utoipa::path(
    post,
    path = "/upload",
    request_body(
        content = UploadFormData,
        content_type = "multipart/form-data"
    ),
    responses(
        (status = 200, description = "Files uploaded successfully", body = UploadResponse),
        (status = 400, description = "Bad request - no files uploaded or missing slug"),
        (status = 401, description = "Unauthorized - invalid or missing API key"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("api_key" = [])
    ),
    tag = "File Management"
)]
pub async fn upload_file(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut slug: Option<String> = None;
    let mut file_data: Vec<(String, Vec<u8>)> = Vec::new();

    // First pass: collect all fields
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("Failed to read multipart field: {}", e);
        StatusCode::BAD_REQUEST
    })? {
        let name = field.name().unwrap_or("");

        if name == "slug" {
            let data = field.bytes().await.map_err(|e| {
                error!("Failed to read slug data: {}", e);
                StatusCode::BAD_REQUEST
            })?;
            slug = Some(String::from_utf8(data.to_vec()).map_err(|e| {
                error!("Invalid UTF-8 in slug: {}", e);
                StatusCode::BAD_REQUEST
            })?);
            info!("Received slug: {:?}", slug);
        } else if name == "file" {
            let filename = field.file_name().unwrap_or("unknown").to_string();
            let data = field.bytes().await.map_err(|e| {
                error!("Failed to read file data: {}", e);
                StatusCode::BAD_REQUEST
            })?;
            info!("Received file: {}", filename);
            file_data.push((filename, data.to_vec()));
        }
    }

    // Validate we have both slug and files
    let slug_val = slug.ok_or_else(|| {
        error!("No slug provided");
        StatusCode::BAD_REQUEST
    })?;

    if file_data.is_empty() {
        error!("No files provided");
        return Err(StatusCode::BAD_REQUEST);
    }

    // Process uploaded files
    let mut uploaded_files = Vec::new();
    
    // Create slug directory
    let slug_dir = state.upload_dir.join(&slug_val);
    fs::create_dir_all(&slug_dir).await.map_err(|e| {
        error!("Failed to create directory {}: {}", slug_dir.display(), e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    for (filename, data) in file_data {
        // Generate unique filename
        let ext = std::path::Path::new(&filename)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        
        let file_stem = std::path::Path::new(&filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
            
        let unique_filename = format!("{}_{}.{}", 
            file_stem,
            Uuid::new_v4().to_string()[..8].to_string(),
            ext
        );

        let file_path = slug_dir.join(&unique_filename);
        
        // Write file
        let mut file = fs::File::create(&file_path).await.map_err(|e| {
            error!("Failed to create file {}: {}", file_path.display(), e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        
        file.write_all(&data).await.map_err(|e| {
            error!("Failed to write file {}: {}", file_path.display(), e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // Generate thumbnail if it's an image
        if is_image(&filename) {
            generate_thumbnail(&file_path, &data).await;
        }

        let file_url = format!("/files/{}/{}", slug_val, unique_filename);
        uploaded_files.push(serde_json::json!({
            "filename": unique_filename,
            "url": file_url,
            "path": file_path.to_string_lossy()
        }));

        info!("Uploaded file: {} to {}", filename, file_path.display());
    }

    Ok(Json(serde_json::json!({
        "message": "Files uploaded successfully",
        "files": uploaded_files
    })))
}

/// Delete a complete folder and all its contents
///
/// Deletes a folder (typically an album folder) and all files within it.
/// This operation is irreversible and will permanently remove all files in the specified folder.
/// 
/// **Authentication Required**: This endpoint requires a valid API key in the `X-API-Key` header.
/// 
/// Required headers:
/// - `X-API-Key`: Valid API key for authentication
#[utoipa::path(
    delete,
    path = "/folder/{slug}",
    responses(
        (status = 200, description = "Folder deleted successfully", body = DeleteResponse),
        (status = 404, description = "Folder not found"),
        (status = 401, description = "Unauthorized - invalid or missing API key"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("slug" = String, Path, description = "Folder name/slug to delete")
    ),
    security(
        ("api_key" = [])
    ),
    tag = "File Management"
)]
pub async fn delete_folder(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let folder_path = state.upload_dir.join(&slug);
    
    // Check if folder exists
    if !folder_path.exists() {
        error!("Folder not found: {}", folder_path.display());
        return Err(StatusCode::NOT_FOUND);
    }
    
    // Check if it's actually a directory
    if !folder_path.is_dir() {
        error!("Path is not a directory: {}", folder_path.display());
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Remove the directory and all its contents
    match fs::remove_dir_all(&folder_path).await {
        Ok(_) => {
            info!("Successfully deleted folder: {}", folder_path.display());
            Ok(Json(serde_json::json!({
                "message": "Folder deleted successfully",
                "folder": slug
            })))
        }
        Err(e) => {
            error!("Failed to delete folder {}: {}", folder_path.display(), e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Check if a file is an image based on its extension
fn is_image(filename: &str) -> bool {
    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp")
}

/// Generate a thumbnail for an image file
async fn generate_thumbnail(file_path: &std::path::Path, data: &[u8]) {
    if let Ok(img) = image::load_from_memory(data) {
        let thumbnail = img.thumbnail(300, 300);
        
        let thumb_path = file_path.with_extension(
            format!("thumb.{}", 
                file_path.extension().unwrap_or_default().to_str().unwrap_or("jpg")
            )
        );
        
        if let Err(e) = thumbnail.save(&thumb_path) {
            error!("Failed to save thumbnail: {}", e);
        } else {
            info!("Generated thumbnail: {}", thumb_path.display());
        }
    }
}

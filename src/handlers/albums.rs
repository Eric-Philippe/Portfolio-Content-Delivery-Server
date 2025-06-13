//! Photo Albums Handlers
//! 
//! This module contains HTTP handlers for managing photo albums in the portfolio.
//! It provides endpoints for listing albums and retrieving album details with content.

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

use crate::{database, models::*, AppState};

/// Get all photo albums
///
/// Returns a list of all photo albums in the portfolio
#[utoipa::path(
    get,
    path = "/albums",
    responses(
        (status = 200, description = "List of photo albums", body = [AlbumMetadata]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Photo Albums"
)]
pub async fn get_albums(
    State(state): State<AppState>,
) -> Result<Json<Vec<AlbumMetadata>>, StatusCode> {
    match database::get_all_albums(&state.db).await {
        Ok(albums) => Ok(Json(albums)),
        Err(e) => {
            error!("Failed to fetch albums: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get a specific photo album with its content
///
/// Returns detailed information about a photo album including all its images
#[utoipa::path(
    get,
    path = "/albums/{slug}",
    responses(
        (status = 200, description = "Album with content", body = AlbumWithContent),
        (status = 404, description = "Album not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("slug" = String, Path, description = "Album slug identifier")
    ),
    tag = "Photo Albums"
)]
pub async fn get_album(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<AlbumWithContent>, StatusCode> {
    match database::get_album_with_content(&state.db, &slug).await {
        Ok(Some(album)) => Ok(Json(album)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to fetch album: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create a new album
///
/// Create a new photo album in the portfolio
/// 
/// **Authentication Required**: This endpoint requires a valid API key in the `X-API-Key` header.
#[utoipa::path(
    post,
    path = "/albums",
    request_body = CreateAlbumRequest,
    responses(
        (status = 201, description = "Album created successfully", body = AlbumOperationResponse),
        (status = 400, description = "Invalid request data"),
        (status = 409, description = "Album with this slug already exists"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("api_key" = [])
    ),
    tag = "Photo Albums"
)]
pub async fn create_album(
    State(state): State<AppState>,
    Json(request): Json<CreateAlbumRequest>,
) -> Result<Json<AlbumOperationResponse>, StatusCode> {
    // Check if album with this slug already exists
    match database::album_exists(&state.db, &request.slug).await {
        Ok(true) => {
            return Err(StatusCode::CONFLICT);
        }
        Ok(false) => {} // OK, album doesn't exist
        Err(e) => {
            error!("Failed to check existing album: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // Convert request to AlbumMetadata
    let album = AlbumMetadata {
        slug: request.slug.clone(),
        title: request.title,
        description: request.description,
        short_title: request.short_title,
        date: request.date,
        camera: request.camera,
        lens: request.lens,
        phone: request.phone,
        preview_img_one_url: request.preview_img_one_url,
        feature: request.feature,
        category: request.category,
    };

    // Create album directory
    let album_dir = state.upload_dir.join(&request.slug);
    if let Err(e) = fs::create_dir_all(&album_dir).await {
        error!("Failed to create album directory {}: {}", album_dir.display(), e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    match database::create_album(&state.db, &album).await {
        Ok(_) => {
            info!("Created album: {}", request.slug);
            Ok(Json(AlbumOperationResponse {
                message: "Album created successfully".to_string(),
                slug: request.slug,
            }))
        }
        Err(e) => {
            error!("Failed to create album: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create a new album with files
///
/// Create a new photo album and upload files to it in one operation
/// 
/// **Authentication Required**: This endpoint requires a valid API key in the `X-API-Key` header.
/// 
/// Required form fields:
/// - `album_data`: Album metadata as JSON string
/// - `files`: Files to upload (binary, can be multiple files)
#[utoipa::path(
    post,
    path = "/albums/with-files",
    request_body(
        content = CreateAlbumWithFilesFormData,
        content_type = "multipart/form-data"
    ),
    responses(
        (status = 201, description = "Album created with files successfully", body = AddPhotosResponse),
        (status = 400, description = "Bad request - invalid data or missing fields"),
        (status = 409, description = "Album with this slug already exists"),
        (status = 401, description = "Unauthorized - invalid or missing API key"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("api_key" = [])
    ),
    tag = "Photo Albums"
)]
pub async fn create_album_with_files(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<AddPhotosResponse>, StatusCode> {
    let mut album_data: Option<String> = None;
    let mut file_data: Vec<(String, Vec<u8>)> = Vec::new();

    // Collect all fields
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("Failed to read multipart field: {}", e);
        StatusCode::BAD_REQUEST
    })? {
        let name = field.name().unwrap_or("");

        if name == "album_data" {
            let data = field.bytes().await.map_err(|e| {
                error!("Failed to read album data: {}", e);
                StatusCode::BAD_REQUEST
            })?;
            album_data = Some(String::from_utf8(data.to_vec()).map_err(|e| {
                error!("Invalid UTF-8 in album data: {}", e);
                StatusCode::BAD_REQUEST
            })?);
        } else if name == "files" {
            let filename = field.file_name().unwrap_or("unknown").to_string();
            let data = field.bytes().await.map_err(|e| {
                error!("Failed to read file data: {}", e);
                StatusCode::BAD_REQUEST
            })?;
            file_data.push((filename, data.to_vec()));
        }
    }

    // Parse album data
    let album_json = album_data.ok_or_else(|| {
        error!("No album data provided");
        StatusCode::BAD_REQUEST
    })?;

    let album_request: CreateAlbumRequest = serde_json::from_str(&album_json).map_err(|e| {
        error!("Failed to parse album data: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    // Check if album exists
    match database::album_exists(&state.db, &album_request.slug).await {
        Ok(true) => return Err(StatusCode::CONFLICT),
        Ok(false) => {},
        Err(e) => {
            error!("Failed to check existing album: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // Create album
    let album = AlbumMetadata {
        slug: album_request.slug.clone(),
        title: album_request.title,
        description: album_request.description,
        short_title: album_request.short_title,
        date: album_request.date,
        camera: album_request.camera,
        lens: album_request.lens,
        phone: album_request.phone,
        preview_img_one_url: album_request.preview_img_one_url,
        feature: album_request.feature,
        category: album_request.category,
    };

    // Create album directory
    let album_dir = state.upload_dir.join(&album_request.slug);
    fs::create_dir_all(&album_dir).await.map_err(|e| {
        error!("Failed to create album directory {}: {}", album_dir.display(), e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Create album in database
    if let Err(e) = database::create_album(&state.db, &album).await {
        error!("Failed to create album: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Process uploaded files
    let mut added_photos = Vec::new();
    
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

        let file_path = album_dir.join(&unique_filename);
        
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

        let img_url = format!("/files/{}/{}", album_request.slug, unique_filename);
        let img_path = format!("uploads/{}/{}", album_request.slug, unique_filename);

        // Add to album content
        let content = AlbumContent {
            slug: album_request.slug.clone(),
            img_url: img_url.clone(),
            caption: format!("Photo from {}", filename),
            img_path: img_path.clone(),
        };

        if let Err(e) = database::add_album_content(&state.db, &content).await {
            error!("Failed to add album content: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }

        added_photos.push(content);
        info!("Added photo: {} to album {}", unique_filename, album_request.slug);
    }

    Ok(Json(AddPhotosResponse {
        message: "Album created with files successfully".to_string(),
        album_slug: album_request.slug,
        added_photos,
    }))
}

/// Update an existing album
///
/// Update an existing photo album. Only provided fields will be updated.
/// 
/// **Authentication Required**: This endpoint requires a valid API key in the `X-API-Key` header.
#[utoipa::path(
    put,
    path = "/albums/{slug}",
    request_body = UpdateAlbumRequest,
    responses(
        (status = 200, description = "Album updated successfully", body = AlbumOperationResponse),
        (status = 400, description = "Invalid request data"),
        (status = 404, description = "Album not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("slug" = String, Path, description = "Album slug identifier")
    ),
    security(
        ("api_key" = [])
    ),
    tag = "Photo Albums"
)]
pub async fn update_album(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(request): Json<UpdateAlbumRequest>,
) -> Result<Json<AlbumOperationResponse>, StatusCode> {
    // Get existing album
    let mut existing_album = match database::get_album_with_content(&state.db, &slug).await {
        Ok(Some(album)) => album.metadata,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to fetch existing album: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Update only provided fields
    if let Some(title) = request.title {
        existing_album.title = title;
    }
    if let Some(description) = request.description {
        existing_album.description = description;
    }
    if let Some(short_title) = request.short_title {
        existing_album.short_title = short_title;
    }
    if let Some(date) = request.date {
        existing_album.date = date;
    }
    if let Some(camera) = request.camera {
        existing_album.camera = Some(camera);
    }
    if let Some(lens) = request.lens {
        existing_album.lens = Some(lens);
    }
    if let Some(phone) = request.phone {
        existing_album.phone = Some(phone);
    }
    if let Some(preview_img_one_url) = request.preview_img_one_url {
        existing_album.preview_img_one_url = preview_img_one_url;
    }
    if let Some(feature) = request.feature {
        existing_album.feature = feature;
    }
    if let Some(category) = request.category {
        existing_album.category = category;
    }

    match database::update_album(&state.db, &slug, &existing_album).await {
        Ok(true) => Ok(Json(AlbumOperationResponse {
            message: "Album updated successfully".to_string(),
            slug,
        })),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to update album: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete an album
///
/// Delete an existing photo album and all its content from the database.
/// Note: This only removes database entries, not the actual files from the server.
/// 
/// **Authentication Required**: This endpoint requires a valid API key in the `X-API-Key` header.
#[utoipa::path(
    delete,
    path = "/albums/{slug}",
    responses(
        (status = 200, description = "Album deleted successfully", body = AlbumOperationResponse),
        (status = 404, description = "Album not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("slug" = String, Path, description = "Album slug identifier")
    ),
    security(
        ("api_key" = [])
    ),
    tag = "Photo Albums"
)]
pub async fn delete_album(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<AlbumOperationResponse>, StatusCode> {
    match database::delete_album(&state.db, &slug).await {
        Ok(true) => {
            info!("Deleted album: {}", slug);
            Ok(Json(AlbumOperationResponse {
                message: "Album deleted successfully".to_string(),
                slug,
            }))
        }
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to delete album: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Add photos to an existing album
///
/// Upload and add new photos to an existing album
/// 
/// **Authentication Required**: This endpoint requires a valid API key in the `X-API-Key` header.
/// 
/// Required form fields:
/// - `caption`: Optional caption for the photos
/// - `files`: Files to upload (binary, can be multiple files)
#[utoipa::path(
    put,
    path = "/albums/{slug}/photos",
    request_body(
        content = AddPhotosToAlbumFormData,
        content_type = "multipart/form-data"
    ),
    responses(
        (status = 200, description = "Photos added successfully", body = AddPhotosResponse),
        (status = 400, description = "Bad request - no files uploaded"),
        (status = 404, description = "Album not found"),
        (status = 401, description = "Unauthorized - invalid or missing API key"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("slug" = String, Path, description = "Album slug identifier")
    ),
    security(
        ("api_key" = [])
    ),
    tag = "Photo Albums"
)]
pub async fn add_photos_to_album(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<AddPhotosResponse>, StatusCode> {
    // Check if album exists
    if !database::album_exists(&state.db, &slug).await.map_err(|e| {
        error!("Failed to check album existence: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })? {
        return Err(StatusCode::NOT_FOUND);
    }

    let mut caption: Option<String> = None;
    let mut file_data: Vec<(String, Vec<u8>)> = Vec::new();

    // Collect all fields
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("Failed to read multipart field: {}", e);
        StatusCode::BAD_REQUEST
    })? {
        let name = field.name().unwrap_or("");

        if name == "caption" {
            let data = field.bytes().await.map_err(|e| {
                error!("Failed to read caption data: {}", e);
                StatusCode::BAD_REQUEST
            })?;
            caption = Some(String::from_utf8(data.to_vec()).map_err(|e| {
                error!("Invalid UTF-8 in caption: {}", e);
                StatusCode::BAD_REQUEST
            })?);
        } else if name == "files" {
            let filename = field.file_name().unwrap_or("unknown").to_string();
            let data = field.bytes().await.map_err(|e| {
                error!("Failed to read file data: {}", e);
                StatusCode::BAD_REQUEST
            })?;
            file_data.push((filename, data.to_vec()));
        }
    }

    if file_data.is_empty() {
        error!("No files provided");
        return Err(StatusCode::BAD_REQUEST);
    }

    // Get album directory
    let album_dir = state.upload_dir.join(&slug);
    fs::create_dir_all(&album_dir).await.map_err(|e| {
        error!("Failed to create album directory {}: {}", album_dir.display(), e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut added_photos = Vec::new();
    let default_caption = caption.unwrap_or_else(|| "Photo".to_string());

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

        let file_path = album_dir.join(&unique_filename);
        
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

        let img_url = format!("/files/{}/{}", slug, unique_filename);
        let img_path = format!("uploads/{}/{}", slug, unique_filename);

        // Add to album content
        let content = AlbumContent {
            slug: slug.clone(),
            img_url: img_url.clone(),
            caption: default_caption.clone(),
            img_path: img_path.clone(),
        };

        if let Err(e) = database::add_album_content(&state.db, &content).await {
            error!("Failed to add album content: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }

        added_photos.push(content);
        info!("Added photo: {} to album {}", unique_filename, slug);
    }

    Ok(Json(AddPhotosResponse {
        message: "Photos added successfully".to_string(),
        album_slug: slug,
        added_photos,
    }))
}

/// Remove a photo from an album
///
/// Remove a specific photo from an album. Only removes the database entry, not the actual file.
/// 
/// **Authentication Required**: This endpoint requires a valid API key in the `X-API-Key` header.
#[utoipa::path(
    delete,
    path = "/albums/{slug}/photos",
    request_body = RemovePhotoRequest,
    responses(
        (status = 200, description = "Photo removed successfully", body = AlbumOperationResponse),
        (status = 404, description = "Album or photo not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("slug" = String, Path, description = "Album slug identifier")
    ),
    security(
        ("api_key" = [])
    ),
    tag = "Photo Albums"
)]
pub async fn remove_photo_from_album(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(request): Json<RemovePhotoRequest>,
) -> Result<Json<AlbumOperationResponse>, StatusCode> {
    match database::remove_album_content(&state.db, &slug, &request.img_url).await {
        Ok(true) => {
            info!("Removed photo: {} from album {}", request.img_url, slug);
            Ok(Json(AlbumOperationResponse {
                message: "Photo removed successfully".to_string(),
                slug,
            }))
        }
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to remove photo from album: {}", e);
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

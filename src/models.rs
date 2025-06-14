use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "slug": "portfolio-server",
    "en_title": "Portfolio Server",
    "en_short_description": "A lightweight Rust server for portfolio content",
    "fr_title": "Serveur Portfolio",
    "fr_short_description": "Un serveur Rust léger pour le contenu de portfolio",
    "techs": "Rust,Axum,PostgreSQL",
    "link": "https://github.com/username/portfolio-server",
    "date": "2025-06-13",
    "tags": "web,backend,api",
    "priority": 1
}))]
pub struct Dev_Project_Metadata {
    pub slug: String,
    pub en_title: String,
    pub en_short_description: String,
    pub fr_title: String,
    pub fr_short_description: String,
    pub techs: String,
    pub link: String,
    pub date: String,
    pub tags: String,
    pub priority: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "slug": "urban-exploration",
    "title": "Urban Exploration 2025",
    "description": "Exploring the city through photography",
    "short_title": "Urban 2025",
    "date": "2025-06-01",
    "camera": "Canon EOS R5",
    "lens": "RF 24-70mm f/2.8L",
    "phone": null,
    "preview_img_one_url": "/files/urban-exploration/preview1.jpg",
    "featured": true,
    "category": "Street"
}))]
pub struct Album_Metadata {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub short_title: String,
    pub date: String,
    pub camera: Option<String>,
    pub lens: Option<String>,
    pub phone: Option<String>,
    pub preview_img_one_url: String,
    pub featured: bool,
    pub category: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "slug": "urban-exploration",
    "img_url": "/files/urban-exploration/street1.jpg",
    "caption": "Street art in downtown",
}))]
pub struct Album_Content {
    pub slug: String,
    pub img_url: String,
    pub caption: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AlbumWithContent {
    #[serde(flatten)]
    #[schema(inline)]
    pub metadata: Album_Metadata,
    pub content: Vec<Album_Content>,
}

/// Form data for file upload
/// 
/// This represents the multipart/form-data structure for uploading files.
/// In Swagger UI, you'll see:
/// - A text input for 'slug' field
/// - A file picker for 'file' field
#[derive(ToSchema)]
pub struct UploadFormData {
    /// Album slug identifier where the file will be uploaded
    /// 
    /// This should match an existing album slug in your system.
    /// Examples: "nature-walks", "urban-exploration", "portraits"
    #[schema(example = "nature-walks")]
    pub slug: String,
    
    /// File to upload
    /// 
    /// Select one or more files using the file picker.
    /// Supported formats: images (jpg, png, gif, webp), videos, documents
    #[schema(format = "binary")]
    pub file: Vec<u8>,
}

#[derive(ToSchema, Serialize, Deserialize)]
#[schema(example = json!({
    "message": "Files uploaded successfully",
    "files": [
        {
            "filename": "photo_a1b2c3d4.jpg",
            "url": "/files/nature-walks/photo_a1b2c3d4.jpg",
            "path": "/home/user/uploads/nature-walks/photo_a1b2c3d4.jpg"
        }
    ]
}))]
pub struct UploadResponse {
    /// Success message
    pub message: String,
    
    /// List of uploaded files with their URLs and paths
    pub files: Vec<UploadedFileInfo>,
}

#[derive(ToSchema, Serialize, Deserialize)]
#[schema(example = json!({
    "filename": "photo_a1b2c3d4.jpg",
    "url": "/files/nature-walks/photo_a1b2c3d4.jpg",
    "path": "/home/user/uploads/nature-walks/photo_a1b2c3d4.jpg"
}))]
pub struct UploadedFileInfo {
    /// Generated filename with unique identifier
    pub filename: String,
    
    /// Public URL to access the uploaded file
    pub url: String,
    
    /// Full path to the uploaded file on the server
    pub path: String,
}

#[derive(ToSchema, Serialize, Deserialize)]
#[schema(example = json!({
    "message": "Folder deleted successfully",
    "folder": "urban-exploration"
}))]
pub struct DeleteResponse {
    /// Success message
    pub message: String,
    
    /// Name of the deleted folder
    pub folder: String,
}

/// Input data for creating a new development project
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "slug": "new-project",
    "en_title": "New Project",
    "en_short_description": "A new amazing project",
    "fr_title": "Nouveau Projet",
    "fr_short_description": "Un nouveau projet formidable",
    "techs": "Rust,JavaScript,Python",
    "link": "https://github.com/username/new-project",
    "date": "2025-06-13",
    "tags": "web,api,tools",
    "priority": 1
}))]
pub struct CreateDevProjectRequest {
    pub slug: String,
    pub en_title: String,
    pub en_short_description: String,
    pub fr_title: String,
    pub fr_short_description: String,
    pub techs: String,
    pub link: String,
    pub date: String,
    pub tags: String,
    pub priority: Option<i32>,
}

/// Input data for updating a development project
/// All fields are optional - only provided fields will be updated
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "en_title": "Updated Project Title",
    "en_short_description": "Updated project description",
    "techs": "Rust,TypeScript,React",
    "priority": 2
}))]
pub struct UpdateDevProjectRequest {
    pub en_title: Option<String>,
    pub en_short_description: Option<String>,
    pub fr_title: Option<String>,
    pub fr_short_description: Option<String>,
    pub techs: Option<String>,
    pub link: Option<String>,
    pub date: Option<String>,
    pub tags: Option<String>,
    pub priority: Option<i32>,
}

/// Response for project creation/update operations
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "message": "Project created successfully",
    "slug": "new-project"
}))]
pub struct ProjectOperationResponse {
    pub message: String,
    pub slug: String,
}

/// Input data for creating a new album
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "slug": "paris-2025",
    "title": "Paris Street Photography 2025",
    "description": "A collection of street photography from my trip to Paris in 2025",
    "short_title": "Paris 2025",
    "date": "2025-06-13",
    "camera": "Canon EOS R5",
    "lens": "RF 24-70mm f/2.8L",
    "phone": null,
    "preview_img_one_url": "/files/paris-2025/preview.jpg",
    "featured": true,
    "category": "Street"
}))]
pub struct CreateAlbumRequest {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub short_title: String,
    pub date: String,
    pub camera: Option<String>,
    pub lens: Option<String>,
    pub phone: Option<String>,
    pub preview_img_one_url: String,
    pub featured: bool,
    pub category: String,
}

/// Input data for updating an album
/// All fields are optional - only provided fields will be updated
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "title": "Updated Album Title",
    "description": "Updated album description",
    "featured": false
}))]
pub struct UpdateAlbumRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub short_title: Option<String>,
    pub date: Option<String>,
    pub camera: Option<String>,
    pub lens: Option<String>,
    pub phone: Option<String>,
    pub preview_img_one_url: Option<String>,
    pub featured: Option<bool>,
    pub category: Option<String>,
}

/// Response for album creation/update/delete operations
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "message": "Album created successfully",
    "slug": "paris-2025"
}))]
pub struct AlbumOperationResponse {
    pub message: String,
    pub slug: String,
}

/// Form data for album creation with file upload
/// 
/// This represents the multipart/form-data structure for creating an album with files.
#[derive(ToSchema)]
pub struct CreateAlbumWithFilesFormData {
    /// Album metadata as JSON string
    /// 
    /// This should contain all the album information as a JSON object
    #[schema(example = r#"{"slug":"paris-2025","title":"Paris Street Photography 2025","description":"A collection of street photography","short_title":"Paris 2025","date":"2025-06-13","camera":"Canon EOS R5","lens":"RF 24-70mm f/2.8L","phone":null,"preview_img_one_url":"/files/paris-2025/preview.jpg","featured":true,"category":"Street"}"#)]
    pub album_data: String,
    
    /// Files to upload with the album
    /// 
    /// Select one or more files using the file picker.
    /// These will be added as content to the album.
    #[schema(format = "binary")]
    pub files: Vec<u8>,
}

/// Form data for adding photos to an existing album
#[derive(ToSchema)]
pub struct AddPhotosToAlbumFormData {
    /// Caption for the photos (optional)
    /// 
    /// A general caption that will be applied to all uploaded photos.
    /// If not provided, default captions will be generated.
    #[schema(example = "Beautiful sunset view")]
    pub caption: Option<String>,
    
    /// Files to upload to the album
    /// 
    /// Select one or more files using the file picker.
    #[schema(format = "binary")]
    pub files: Vec<u8>,
}

/// Response for adding photos to an album
#[derive(ToSchema, Serialize, Deserialize)]
#[schema(example = json!({
    "message": "Photos added successfully",
    "album_slug": "paris-2025",
    "added_photos": [
        {
            "img_url": "/files/paris-2025/photo_a1b2c3d4.jpg",
            "caption": "Beautiful sunset view"
        }
    ]
}))]
pub struct AddPhotosResponse {
    /// Success message
    pub message: String,
    
    /// Slug of the album photos were added to
    pub album_slug: String,
    
    /// List of photos that were added
    pub added_photos: Vec<Album_Content>,
}

/// Request to remove a photo from an album
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "img_url": "/files/paris-2025/photo_a1b2c3d4.jpg"
}))]
pub struct RemovePhotoRequest {
    /// URL of the image to remove from the album
    pub img_url: String,
}

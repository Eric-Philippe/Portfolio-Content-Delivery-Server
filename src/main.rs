use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::path::PathBuf;
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod models;
mod handlers;
mod middleware;
pub mod database;

use handlers::*;
use models::*;
use database::init_database;
use sqlx::sqlite::SqlitePool;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::dev_projects::get_dev_projects,
        handlers::dev_projects::get_dev_project,
        handlers::dev_projects::create_dev_project,
        handlers::dev_projects::update_dev_project,
        handlers::dev_projects::delete_dev_project,
        handlers::albums::get_albums,
        handlers::albums::get_album,
        handlers::albums::create_album,
        handlers::albums::create_album_with_files,
        handlers::albums::update_album,
        handlers::albums::delete_album,
        handlers::albums::add_photos_to_album,
        handlers::albums::remove_photo_from_album,
        handlers::files::upload_file,
        handlers::files::delete_folder,
    ),
    components(
        schemas(DevProjectMetadata, CreateDevProjectRequest, UpdateDevProjectRequest, ProjectOperationResponse, AlbumMetadata, AlbumContent, AlbumWithContent, CreateAlbumRequest, UpdateAlbumRequest, AlbumOperationResponse, CreateAlbumWithFilesFormData, AddPhotosToAlbumFormData, AddPhotosResponse, RemovePhotoRequest, UploadFormData, UploadResponse, UploadedFileInfo, DeleteResponse)
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Development Projects", description = "Portfolio development projects management"),
        (name = "Photo Albums", description = "Photo albums and gallery management"),
        (name = "File Management", description = "File upload and management")
    ),
    info(
        title = "Portfolio API",
        description = "API for managing portfolio content including development projects and photo albums",
        version = "0.1.0",
        contact(
            name = "Portfolio API Support",
            email = "support@portfolio.com"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://127.0.0.1:3000", description = "Local development server"),
        (url = "/", description = "Production server")
    )
)]
struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "api_key",
            utoipa::openapi::security::SecurityScheme::ApiKey(
                utoipa::openapi::security::ApiKey::Header(
                    utoipa::openapi::security::ApiKeyValue::new("X-API-Key")
                )
            )
        );
    }
}

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub upload_dir: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Get configuration from environment or use defaults
    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "3000".to_string());
    let upload_dir = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "uploads".to_string());
    
    // Create upload directory
    let upload_dir = PathBuf::from(upload_dir);
    tokio::fs::create_dir_all(&upload_dir).await?;

    // Initialize database
    let db = init_database().await?;

    let state = AppState { db, upload_dir };

    // Build our application with routes
    let protected_routes = Router::new()
        .route("/upload", post(upload_file))
        .route("/folder/:slug", delete(delete_folder))
        .route("/dev-projects", post(handlers::dev_projects::create_dev_project))
        .route("/dev-projects/:slug", put(handlers::dev_projects::update_dev_project))
        .route("/dev-projects/:slug", delete(handlers::dev_projects::delete_dev_project))
        .route("/albums", post(handlers::albums::create_album))
        .route("/albums/with-files", post(handlers::albums::create_album_with_files))
        .route("/albums/:slug", put(handlers::albums::update_album))
        .route("/albums/:slug", delete(handlers::albums::delete_album))
        .route("/albums/:slug/photos", put(handlers::albums::add_photos_to_album))
        .route("/albums/:slug/photos", delete(handlers::albums::remove_photo_from_album))
        .route_layer(axum::middleware::from_fn(middleware::api_key_auth));

    let app = Router::new()
        .route("/dev-projects", get(get_dev_projects))
        .route("/dev-projects/:slug", get(get_dev_project))
        .route("/albums", get(get_albums))
        .route("/albums/:slug", get(get_album))
        .merge(protected_routes)
        .merge(SwaggerUi::new("/swagger-ui")
            .url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest_service("/files", ServeDir::new("uploads"))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let bind_address = format!("{}:{}", host, port);
    info!("Server starting on http://{}", bind_address);
    info!("Swagger UI available at http://{}/swagger-ui", bind_address);
    info!("OpenAPI JSON available at http://{}/api-docs/openapi.json", bind_address);

    // Run the server
    let listener = tokio::net::TcpListener::bind(&bind_address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

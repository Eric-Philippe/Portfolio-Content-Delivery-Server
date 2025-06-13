//! Development Projects Handlers
//! 
//! This module contains HTTP handlers for managing development projects in the portfolio.
//! It provides endpoints for listing all projects and retrieving individual project details.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use tracing::error;
use utoipa;

use crate::{database, models::*, AppState};

/// Get all development projects
///
/// Returns a list of all development projects in the portfolio
#[utoipa::path(
    get,
    path = "/dev-projects",
    responses(
        (status = 200, description = "List of development projects", body = [Dev_Project_Metadata]),
        (status = 500, description = "Internal server error")
    ),
    tag = "Development Projects"
)]
pub async fn get_dev_projects(
    State(state): State<AppState>,
) -> Result<Json<Vec<Dev_Project_Metadata>>, StatusCode> {
    match database::get_all_dev_projects(&state.db).await {
        Ok(projects) => Ok(Json(projects)),
        Err(e) => {
            error!("Failed to fetch dev projects: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get a specific development project by slug
///
/// Returns detailed information about a development project
#[utoipa::path(
    get,
    path = "/dev-projects/{slug}",
    responses(
        (status = 200, description = "Development project details", body = Dev_Project_Metadata),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("slug" = String, Path, description = "Project slug identifier")
    ),
    tag = "Development Projects"
)]
pub async fn get_dev_project(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Dev_Project_Metadata>, StatusCode> {
    match database::get_dev_project_by_slug(&state.db, &slug).await {
        Ok(Some(project)) => Ok(Json(project)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to fetch dev project: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create a new development project
///
/// Create a new development project in the portfolio
/// 
/// **Authentication Required**: This endpoint requires a valid API key in the `X-API-Key` header.
#[utoipa::path(
    post,
    path = "/dev-projects",
    request_body = CreateDevProjectRequest,
    responses(
        (status = 201, description = "Project created successfully", body = ProjectOperationResponse),
        (status = 400, description = "Invalid request data"),
        (status = 409, description = "Project with this slug already exists"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("api_key" = [])
    ),
    tag = "Development Projects"
)]
pub async fn create_dev_project(
    State(state): State<AppState>,
    Json(request): Json<CreateDevProjectRequest>,
) -> Result<Json<ProjectOperationResponse>, StatusCode> {
    // Check if project with this slug already exists
    match database::get_dev_project_by_slug(&state.db, &request.slug).await {
        Ok(Some(_)) => {
            return Err(StatusCode::CONFLICT);
        }
        Ok(None) => {} // OK, project doesn't exist
        Err(e) => {
            error!("Failed to check existing project: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // Convert request to Dev_Project_Metadata
    let project = Dev_Project_Metadata {
        slug: request.slug.clone(),
        en_title: request.en_title,
        en_short_description: request.en_short_description,
        fr_title: request.fr_title,
        fr_short_description: request.fr_short_description,
        techs: request.techs,
        link: request.link,
        date: request.date,
        tags: request.tags,
    };

    match database::create_dev_project(&state.db, &project).await {
        Ok(_) => Ok(Json(ProjectOperationResponse {
            message: "Project created successfully".to_string(),
            slug: request.slug,
        })),
        Err(e) => {
            error!("Failed to create dev project: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update an existing development project
///
/// Update an existing development project. Only provided fields will be updated.
/// 
/// **Authentication Required**: This endpoint requires a valid API key in the `X-API-Key` header.
#[utoipa::path(
    put,
    path = "/dev-projects/{slug}",
    request_body = UpdateDevProjectRequest,
    responses(
        (status = 200, description = "Project updated successfully", body = ProjectOperationResponse),
        (status = 400, description = "Invalid request data"),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("slug" = String, Path, description = "Project slug identifier")
    ),
    security(
        ("api_key" = [])
    ),
    tag = "Development Projects"
)]
pub async fn update_dev_project(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(request): Json<UpdateDevProjectRequest>,
) -> Result<Json<ProjectOperationResponse>, StatusCode> {
    // Get existing project
    let mut existing_project = match database::get_dev_project_by_slug(&state.db, &slug).await {
        Ok(Some(project)) => project,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to fetch existing project: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Update only provided fields
    if let Some(en_title) = request.en_title {
        existing_project.en_title = en_title;
    }
    if let Some(en_short_description) = request.en_short_description {
        existing_project.en_short_description = en_short_description;
    }
    if let Some(fr_title) = request.fr_title {
        existing_project.fr_title = fr_title;
    }
    if let Some(fr_short_description) = request.fr_short_description {
        existing_project.fr_short_description = fr_short_description;
    }
    if let Some(techs) = request.techs {
        existing_project.techs = techs;
    }
    if let Some(link) = request.link {
        existing_project.link = link;
    }
    if let Some(date) = request.date {
        existing_project.date = date;
    }
    if let Some(tags) = request.tags {
        existing_project.tags = tags;
    }

    match database::update_dev_project(&state.db, &slug, &existing_project).await {
        Ok(true) => Ok(Json(ProjectOperationResponse {
            message: "Project updated successfully".to_string(),
            slug,
        })),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to update dev project: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Delete a development project
///
/// Delete an existing development project
/// 
/// **Authentication Required**: This endpoint requires a valid API key in the `X-API-Key` header.
#[utoipa::path(
    delete,
    path = "/dev-projects/{slug}",
    responses(
        (status = 200, description = "Project deleted successfully", body = ProjectOperationResponse),
        (status = 404, description = "Project not found"),
        (status = 500, description = "Internal server error")
    ),
    params(
        ("slug" = String, Path, description = "Project slug identifier")
    ),
    security(
        ("api_key" = [])
    ),
    tag = "Development Projects"
)]
pub async fn delete_dev_project(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<ProjectOperationResponse>, StatusCode> {
    match database::delete_dev_project(&state.db, &slug).await {
        Ok(true) => Ok(Json(ProjectOperationResponse {
            message: "Project deleted successfully".to_string(),
            slug,
        })),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to delete dev project: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

use sqlx::{postgres::PgPool, Row};
use tracing::info;

use crate::models::*;

pub async fn init_database() -> Result<PgPool, sqlx::Error> {
    // Get database URL from environment or use default
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://portfolio_user:portfolio_password@localhost:5432/portfolio".to_string());
      let pool = PgPool::connect(&database_url).await?;    // Create tables
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS Dev_Project_Metadata (
            slug VARCHAR(255) PRIMARY KEY,
            en_title VARCHAR(500) NOT NULL,
            en_short_description TEXT NOT NULL,
            fr_title VARCHAR(500) NOT NULL,
            fr_short_description TEXT NOT NULL,
            techs TEXT NOT NULL,
            link VARCHAR(1000) NOT NULL,
            date VARCHAR(50) NOT NULL,
            tags TEXT NOT NULL,            priority INT DEFAULT 0
        )
        "#,
    )
    .execute(&pool)
    .await?;

    // Add priority column if it doesn't exist (for existing databases)
    sqlx::query(
        "ALTER TABLE Dev_Project_Metadata ADD COLUMN IF NOT EXISTS priority INT DEFAULT 0"
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS Album_Metadata (
            slug VARCHAR(255) PRIMARY KEY,
            title VARCHAR(500) NOT NULL,
            description TEXT NOT NULL,
            short_title VARCHAR(200) NOT NULL,
            date VARCHAR(50) NOT NULL,
            camera VARCHAR(200),
            lens VARCHAR(200),
            phone VARCHAR(200),
            preview_img_one_url VARCHAR(1000) NOT NULL,
            featured BOOLEAN NOT NULL DEFAULT FALSE,
            category VARCHAR(100) NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS Album_Content (
            slug VARCHAR(255) NOT NULL,
            img_url VARCHAR(1000) NOT NULL,
            caption TEXT NOT NULL,
            PRIMARY KEY (slug, img_url),
            FOREIGN KEY (slug) REFERENCES Album_Metadata(slug) ON DELETE CASCADE
        )
        "#,
    )
    .execute(&pool)
    .await?;

    // Insert sample data if tables are empty
    let dev_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM Dev_Project_Metadata")
        .fetch_one(&pool)
        .await?;

    if dev_count == 0 {        info!("Inserting sample dev projects...");
        sqlx::query(
            "INSERT INTO Dev_Project_Metadata 
            (slug, en_title, en_short_description, fr_title, fr_short_description, techs, link, date, tags, priority) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
        )
        .bind("portfolio-server")
        .bind("Portfolio Server")
        .bind("A lightweight Rust server for portfolio content")
        .bind("Serveur Portfolio")
        .bind("Un serveur Rust léger pour le contenu de portfolio")
        .bind("Rust,Axum,PostgreSQL")
        .bind("https://github.com/username/portfolio-server")
        .bind("2025-06-13")
        .bind("web,backend,api")
        .bind(1)
        .execute(&pool)
        .await?;

        sqlx::query(
            "INSERT INTO Dev_Project_Metadata 
            (slug, en_title, en_short_description, fr_title, fr_short_description, techs, link, date, tags, priority) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
        )
        .bind("photo-gallery")
        .bind("Photo Gallery App")
        .bind("Modern photo gallery with responsive design")
        .bind("Application Galerie Photo")
        .bind("Galerie photo moderne avec design responsive")
        .bind("React,TypeScript,Tailwind")
        .bind("https://github.com/username/photo-gallery")
        .bind("2025-05-20")
        .bind("frontend,react,photography")
        .bind(2)
        .execute(&pool)
        .await?;

        sqlx::query(
            "INSERT INTO Album_Metadata 
            (slug, title, description, short_title, date, camera, lens, phone, preview_img_one_url, featured, category) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
        )
        .bind("urban-exploration")
        .bind("Urban Exploration 2025")
        .bind("Exploring the city through photography")
        .bind("Urban 2025")
        .bind("2025-06-01")
        .bind("Canon EOS R5")
        .bind("RF 24-70mm f/2.8L")
        .bind(None::<String>)
        .bind("/files/urban-exploration/preview1.jpg")
        .bind(true)
        .bind("Street")
        .execute(&pool)
        .await?;

        sqlx::query(
            "INSERT INTO Album_Content (slug, img_url, caption) VALUES ($1, $2, $3)"
        )
        .bind("urban-exploration")
        .bind("/files/urban-exploration/street1.jpg")
        .bind("Street art in downtown")
        .execute(&pool)
        .await?;

        info!("Sample data inserted successfully");
    }

    info!("Database initialized successfully");
    Ok(pool)
}

pub async fn get_all_dev_projects(pool: &PgPool) -> Result<Vec<Dev_Project_Metadata>, sqlx::Error> {
    let rows = sqlx::query("SELECT * FROM Dev_Project_Metadata ORDER BY priority ASC, date DESC")
        .fetch_all(pool)
        .await?;

    let projects = rows
        .into_iter()
        .map(|row| Dev_Project_Metadata {
            slug: row.get("slug"),
            en_title: row.get("en_title"),
            en_short_description: row.get("en_short_description"),
            fr_title: row.get("fr_title"),
            fr_short_description: row.get("fr_short_description"),
            techs: row.get("techs"),
            link: row.get("link"),
            date: row.get("date"),
            tags: row.get("tags"),
            priority: row.get("priority"),
        })
        .collect();

    Ok(projects)
}

pub async fn get_dev_project_by_slug(
    pool: &PgPool,
    slug: &str,
) -> Result<Option<Dev_Project_Metadata>, sqlx::Error> {
    let row = sqlx::query("SELECT * FROM Dev_Project_Metadata WHERE slug = $1")
        .bind(slug)
        .fetch_optional(pool)
        .await?;    if let Some(row) = row {
        Ok(Some(Dev_Project_Metadata {
            slug: row.get("slug"),
            en_title: row.get("en_title"),
            en_short_description: row.get("en_short_description"),
            fr_title: row.get("fr_title"),
            fr_short_description: row.get("fr_short_description"),
            techs: row.get("techs"),
            link: row.get("link"),
            date: row.get("date"),
            tags: row.get("tags"),
            priority: row.get("priority"),
        }))
    } else {
        Ok(None)
    }
}

pub async fn get_all_albums(pool: &PgPool) -> Result<Vec<Album_Metadata>, sqlx::Error> {
    let rows = sqlx::query("SELECT * FROM Album_Metadata ORDER BY date DESC")
        .fetch_all(pool)
        .await?;

    let albums = rows
        .into_iter()
        .map(|row| Album_Metadata {
            slug: row.get("slug"),
            title: row.get("title"),
            description: row.get("description"),
            short_title: row.get("short_title"),
            date: row.get("date"),
            camera: row.get("camera"),
            lens: row.get("lens"),
            phone: row.get("phone"),
            preview_img_one_url: row.get("preview_img_one_url"),
            featured: row.get("featured"),
            category: row.get("category"),
        })
        .collect();

    Ok(albums)
}

pub async fn get_album_with_content(
    pool: &PgPool,
    slug: &str,
) -> Result<Option<AlbumWithContent>, sqlx::Error> {
    // Get album metadata
    let album_row = sqlx::query("SELECT * FROM Album_Metadata WHERE slug = $1")
        .bind(slug)
        .fetch_optional(pool)
        .await?;

    if let Some(album_row) = album_row {
        let metadata = Album_Metadata {
            slug: album_row.get("slug"),
            title: album_row.get("title"),
            description: album_row.get("description"),
            short_title: album_row.get("short_title"),
            date: album_row.get("date"),
            camera: album_row.get("camera"),
            lens: album_row.get("lens"),
            phone: album_row.get("phone"),
            preview_img_one_url: album_row.get("preview_img_one_url"),
            featured: album_row.get("featured"),
            category: album_row.get("category"),
        };        // Get album content
        let content_rows = sqlx::query("SELECT * FROM Album_Content WHERE slug = $1")
            .bind(slug)
            .fetch_all(pool)
            .await?;

        let content = content_rows
            .into_iter()
            .map(|row| Album_Content {
                slug: row.get("slug"),
                img_url: row.get("img_url"),
                caption: row.get("caption"),
            })
            .collect();

        Ok(Some(AlbumWithContent { metadata, content }))
    } else {
        Ok(None)
    }
}

/// Create a new development project
pub async fn create_dev_project(
    pool: &PgPool,
    project: &Dev_Project_Metadata,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO Dev_Project_Metadata 
        (slug, en_title, en_short_description, fr_title, fr_short_description, techs, link, date, tags, priority) 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
    )
    .bind(&project.slug)
    .bind(&project.en_title)
    .bind(&project.en_short_description)
    .bind(&project.fr_title)
    .bind(&project.fr_short_description)
    .bind(&project.techs)
    .bind(&project.link)
    .bind(&project.date)
    .bind(&project.tags)
    .bind(project.priority)
    .execute(pool)
    .await?;

    Ok(())
}

/// Update an existing development project
pub async fn update_dev_project(
    pool: &PgPool,
    slug: &str,
    project: &Dev_Project_Metadata,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE Dev_Project_Metadata 
        SET en_title = $1, en_short_description = $2, fr_title = $3, fr_short_description = $4, 
            techs = $5, link = $6, date = $7, tags = $8, priority = $9 
        WHERE slug = $10"
    )
    .bind(&project.en_title)
    .bind(&project.en_short_description)
    .bind(&project.fr_title)
    .bind(&project.fr_short_description)
    .bind(&project.techs)
    .bind(&project.link)
    .bind(&project.date)
    .bind(&project.tags)
    .bind(project.priority)
    .bind(slug)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Delete a development project
pub async fn delete_dev_project(
    pool: &PgPool,
    slug: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM Dev_Project_Metadata WHERE slug = $1")
        .bind(slug)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Create a new album
pub async fn create_album(
    pool: &PgPool,
    album: &Album_Metadata,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO Album_Metadata 
        (slug, title, description, short_title, date, camera, lens, phone, preview_img_one_url, featured, category) 
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
    )
    .bind(&album.slug)
    .bind(&album.title)
    .bind(&album.description)
    .bind(&album.short_title)
    .bind(&album.date)
    .bind(&album.camera)
    .bind(&album.lens)
    .bind(&album.phone)
    .bind(&album.preview_img_one_url)
    .bind(album.featured)
    .bind(&album.category)
    .execute(pool)
    .await?;

    Ok(())
}

/// Update an existing album
pub async fn update_album(
    pool: &PgPool,
    slug: &str,
    album: &Album_Metadata,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE Album_Metadata 
        SET title = $1, description = $2, short_title = $3, date = $4, camera = $5, lens = $6, 
            phone = $7, preview_img_one_url = $8, featured = $9, category = $10 
        WHERE slug = $11"
    )
    .bind(&album.title)
    .bind(&album.description)
    .bind(&album.short_title)
    .bind(&album.date)
    .bind(&album.camera)
    .bind(&album.lens)
    .bind(&album.phone)
    .bind(&album.preview_img_one_url)
    .bind(album.featured)
    .bind(&album.category)
    .bind(slug)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Delete an album and all its content
pub async fn delete_album(
    pool: &PgPool,
    slug: &str,
) -> Result<bool, sqlx::Error> {
    // Start a transaction to ensure both operations succeed or fail together
    let mut tx = pool.begin().await?;

    // Delete album content first (due to foreign key constraint)
    sqlx::query("DELETE FROM Album_Content WHERE slug = $1")
        .bind(slug)
        .execute(&mut *tx)
        .await?;

    // Delete album metadata
    let result = sqlx::query("DELETE FROM Album_Metadata WHERE slug = $1")
        .bind(slug)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(result.rows_affected() > 0)
}

/// Add content to an album
pub async fn add_album_content(
    pool: &PgPool,
    content: &Album_Content,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO Album_Content (slug, img_url, caption) VALUES ($1, $2, $3)"
    )
    .bind(&content.slug)
    .bind(&content.img_url)
    .bind(&content.caption)
    .execute(pool)
    .await?;

    Ok(())
}

/// Remove specific content from an album
pub async fn remove_album_content(
    pool: &PgPool,
    slug: &str,
    img_url: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM Album_Content WHERE slug = $1 AND img_url = $2")
        .bind(slug)
        .bind(img_url)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Check if an album exists
pub async fn album_exists(
    pool: &PgPool,
    slug: &str,
) -> Result<bool, sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM Album_Metadata WHERE slug = $1")
        .bind(slug)
        .fetch_one(pool)
        .await?;

    Ok(count > 0)
}

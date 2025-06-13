use sqlx::{sqlite::SqlitePool, Row};
use tracing::info;

use crate::models::*;

pub async fn init_database() -> Result<SqlitePool, sqlx::Error> {
    // Get database URL from environment or use default with file-based database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./data/portfolio.db".to_string());
    
    // Create directories if using file-based database
    if database_url.starts_with("sqlite:") && !database_url.contains(":memory:") {
        // Handle both sqlite:path and sqlite://path formats
        let db_path = if let Some(path) = database_url.strip_prefix("sqlite://") {
            path
        } else if let Some(path) = database_url.strip_prefix("sqlite:") {
            path
        } else {
            return Err(sqlx::Error::Configuration("Invalid DATABASE_URL format".into()));
        };
        
        if let Some(parent) = std::path::Path::new(db_path).parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                sqlx::Error::Configuration(format!("Failed to create database directory: {}", e).into())
            })?;
        }
    }
    
    let pool = SqlitePool::connect(&database_url).await?;

    // Create tables
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS DevProjectMetadata (
            slug TEXT PRIMARY KEY,
            en_title TEXT NOT NULL,
            en_short_description TEXT NOT NULL,
            fr_title TEXT NOT NULL,
            fr_short_description TEXT NOT NULL,
            techs TEXT NOT NULL,
            link TEXT NOT NULL,
            date TEXT NOT NULL,
            tags TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS AlbumMetadata (
            slug TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            short_title TEXT NOT NULL,
            date TEXT NOT NULL,
            camera TEXT,
            lens TEXT,
            phone TEXT,
            preview_img_one_url TEXT NOT NULL,
            feature BOOLEAN NOT NULL DEFAULT FALSE,
            category TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS AlbumContent (
            slug TEXT NOT NULL,
            img_url TEXT NOT NULL,
            caption TEXT NOT NULL,
            img_path TEXT NOT NULL,
            PRIMARY KEY (slug, img_url),
            FOREIGN KEY (slug) REFERENCES AlbumMetadata(slug) ON DELETE CASCADE
        )
        "#,
    )
    .execute(&pool)
    .await?;

    // Insert sample data if tables are empty
    let dev_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM DevProjectMetadata")
        .fetch_one(&pool)
        .await?;

    if dev_count == 0 {
        info!("Inserting sample dev projects...");
        sqlx::query(
            "INSERT INTO DevProjectMetadata 
            (slug, en_title, en_short_description, fr_title, fr_short_description, techs, link, date, tags) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind("portfolio-server")
        .bind("Portfolio Server")
        .bind("A lightweight Rust server for portfolio content")
        .bind("Serveur Portfolio")
        .bind("Un serveur Rust léger pour le contenu de portfolio")
        .bind("Rust,Axum,SQLite")
        .bind("https://github.com/username/portfolio-server")
        .bind("2025-06-13")
        .bind("web,backend,api")
        .execute(&pool)
        .await?;

        sqlx::query(
            "INSERT INTO DevProjectMetadata 
            (slug, en_title, en_short_description, fr_title, fr_short_description, techs, link, date, tags) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
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
        .execute(&pool)
        .await?;

        sqlx::query(
            "INSERT INTO AlbumMetadata 
            (slug, title, description, short_title, date, camera, lens, phone, preview_img_one_url, feature, category) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
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
            "INSERT INTO AlbumContent (slug, img_url, caption, img_path) VALUES (?, ?, ?, ?)"
        )
        .bind("urban-exploration")
        .bind("/files/urban-exploration/street1.jpg")
        .bind("Street art in downtown")
        .bind("uploads/urban-exploration/street1.jpg")
        .execute(&pool)
        .await?;

        info!("Sample data inserted successfully");
    }

    info!("Database initialized successfully");
    Ok(pool)
}

pub async fn get_all_dev_projects(pool: &SqlitePool) -> Result<Vec<DevProjectMetadata>, sqlx::Error> {
    let rows = sqlx::query("SELECT * FROM DevProjectMetadata ORDER BY date DESC")
        .fetch_all(pool)
        .await?;

    let projects = rows
        .into_iter()
        .map(|row| DevProjectMetadata {
            slug: row.get("slug"),
            en_title: row.get("en_title"),
            en_short_description: row.get("en_short_description"),
            fr_title: row.get("fr_title"),
            fr_short_description: row.get("fr_short_description"),
            techs: row.get("techs"),
            link: row.get("link"),
            date: row.get("date"),
            tags: row.get("tags"),
        })
        .collect();

    Ok(projects)
}

pub async fn get_dev_project_by_slug(
    pool: &SqlitePool,
    slug: &str,
) -> Result<Option<DevProjectMetadata>, sqlx::Error> {
    let row = sqlx::query("SELECT * FROM DevProjectMetadata WHERE slug = ?")
        .bind(slug)
        .fetch_optional(pool)
        .await?;

    if let Some(row) = row {
        Ok(Some(DevProjectMetadata {
            slug: row.get("slug"),
            en_title: row.get("en_title"),
            en_short_description: row.get("en_short_description"),
            fr_title: row.get("fr_title"),
            fr_short_description: row.get("fr_short_description"),
            techs: row.get("techs"),
            link: row.get("link"),
            date: row.get("date"),
            tags: row.get("tags"),
        }))
    } else {
        Ok(None)
    }
}

pub async fn get_all_albums(pool: &SqlitePool) -> Result<Vec<AlbumMetadata>, sqlx::Error> {
    let rows = sqlx::query("SELECT * FROM AlbumMetadata ORDER BY date DESC")
        .fetch_all(pool)
        .await?;

    let albums = rows
        .into_iter()
        .map(|row| AlbumMetadata {
            slug: row.get("slug"),
            title: row.get("title"),
            description: row.get("description"),
            short_title: row.get("short_title"),
            date: row.get("date"),
            camera: row.get("camera"),
            lens: row.get("lens"),
            phone: row.get("phone"),
            preview_img_one_url: row.get("preview_img_one_url"),
            feature: row.get("feature"),
            category: row.get("category"),
        })
        .collect();

    Ok(albums)
}

pub async fn get_album_with_content(
    pool: &SqlitePool,
    slug: &str,
) -> Result<Option<AlbumWithContent>, sqlx::Error> {
    // Get album metadata
    let album_row = sqlx::query("SELECT * FROM AlbumMetadata WHERE slug = ?")
        .bind(slug)
        .fetch_optional(pool)
        .await?;

    if let Some(album_row) = album_row {
        let metadata = AlbumMetadata {
            slug: album_row.get("slug"),
            title: album_row.get("title"),
            description: album_row.get("description"),
            short_title: album_row.get("short_title"),
            date: album_row.get("date"),
            camera: album_row.get("camera"),
            lens: album_row.get("lens"),
            phone: album_row.get("phone"),
            preview_img_one_url: album_row.get("preview_img_one_url"),
            feature: album_row.get("feature"),
            category: album_row.get("category"),
        };

        // Get album content
        let content_rows = sqlx::query("SELECT * FROM AlbumContent WHERE slug = ?")
            .bind(slug)
            .fetch_all(pool)
            .await?;

        let content = content_rows
            .into_iter()
            .map(|row| AlbumContent {
                slug: row.get("slug"),
                img_url: row.get("img_url"),
                caption: row.get("caption"),
                img_path: row.get("img_path"),
            })
            .collect();

        Ok(Some(AlbumWithContent { metadata, content }))
    } else {
        Ok(None)
    }
}

/// Create a new development project
pub async fn create_dev_project(
    pool: &SqlitePool,
    project: &DevProjectMetadata,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO DevProjectMetadata 
        (slug, en_title, en_short_description, fr_title, fr_short_description, techs, link, date, tags) 
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
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
    .execute(pool)
    .await?;

    Ok(())
}

/// Update an existing development project
pub async fn update_dev_project(
    pool: &SqlitePool,
    slug: &str,
    project: &DevProjectMetadata,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE DevProjectMetadata 
        SET en_title = ?, en_short_description = ?, fr_title = ?, fr_short_description = ?, 
            techs = ?, link = ?, date = ?, tags = ? 
        WHERE slug = ?"
    )
    .bind(&project.en_title)
    .bind(&project.en_short_description)
    .bind(&project.fr_title)
    .bind(&project.fr_short_description)
    .bind(&project.techs)
    .bind(&project.link)
    .bind(&project.date)
    .bind(&project.tags)
    .bind(slug)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Delete a development project
pub async fn delete_dev_project(
    pool: &SqlitePool,
    slug: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM DevProjectMetadata WHERE slug = ?")
        .bind(slug)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Create a new album
pub async fn create_album(
    pool: &SqlitePool,
    album: &AlbumMetadata,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO AlbumMetadata 
        (slug, title, description, short_title, date, camera, lens, phone, preview_img_one_url, feature, category) 
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
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
    .bind(album.feature)
    .bind(&album.category)
    .execute(pool)
    .await?;

    Ok(())
}

/// Update an existing album
pub async fn update_album(
    pool: &SqlitePool,
    slug: &str,
    album: &AlbumMetadata,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE AlbumMetadata 
        SET title = ?, description = ?, short_title = ?, date = ?, camera = ?, lens = ?, 
            phone = ?, preview_img_one_url = ?, feature = ?, category = ? 
        WHERE slug = ?"
    )
    .bind(&album.title)
    .bind(&album.description)
    .bind(&album.short_title)
    .bind(&album.date)
    .bind(&album.camera)
    .bind(&album.lens)
    .bind(&album.phone)
    .bind(&album.preview_img_one_url)
    .bind(album.feature)
    .bind(&album.category)
    .bind(slug)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Delete an album and all its content
pub async fn delete_album(
    pool: &SqlitePool,
    slug: &str,
) -> Result<bool, sqlx::Error> {
    // Start a transaction to ensure both operations succeed or fail together
    let mut tx = pool.begin().await?;

    // Delete album content first (due to foreign key constraint)
    sqlx::query("DELETE FROM AlbumContent WHERE slug = ?")
        .bind(slug)
        .execute(&mut *tx)
        .await?;

    // Delete album metadata
    let result = sqlx::query("DELETE FROM AlbumMetadata WHERE slug = ?")
        .bind(slug)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(result.rows_affected() > 0)
}

/// Add content to an album
pub async fn add_album_content(
    pool: &SqlitePool,
    content: &AlbumContent,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO AlbumContent (slug, img_url, caption, img_path) VALUES (?, ?, ?, ?)"
    )
    .bind(&content.slug)
    .bind(&content.img_url)
    .bind(&content.caption)
    .bind(&content.img_path)
    .execute(pool)
    .await?;

    Ok(())
}

/// Remove specific content from an album
pub async fn remove_album_content(
    pool: &SqlitePool,
    slug: &str,
    img_url: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM AlbumContent WHERE slug = ? AND img_url = ?")
        .bind(slug)
        .bind(img_url)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Check if an album exists
pub async fn album_exists(
    pool: &SqlitePool,
    slug: &str,
) -> Result<bool, sqlx::Error> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM AlbumMetadata WHERE slug = ?")
        .bind(slug)
        .fetch_one(pool)
        .await?;

    Ok(count > 0)
}

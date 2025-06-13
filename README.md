# Portfolio - Content Delivery Server

A lightweight Rust server to manage and serve your portfolio content (development projects and photo albums).

> This super lightweight server is designed to serve as a content provider for my portfolio website. Like this I can easily manage my photo hosting, the albums and development projects without needing a full CMS or complex backend. It provides a simple REST API to retrieve projects and albums, supports file uploads with automatic thumbnail generation, and uses SQLite for data storage everything under secure and efficient conditions.

## Features

- **Simple REST API** to retrieve projects and albums
- **File upload** with automatic thumbnail generation for images
- **SQLite database** easily accessible with DataGrip or any SQL client
- **Integrated static file server** with thumbnail support
- **CORS enabled** for frontend integration
- **Environment variable configuration**

## Installation and Launch

1. **Install Rust** (if not already done):

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **Clone and configure**:

   ```bash
   git clone <your-repo>
   cd PortfolioContent
   cp .env.example .env
   # Edit .env according to your needs
   ```

3. **Launch the server**:
   ```bash
   cargo run
   ```

The server starts by default on `http://127.0.0.1:3000`

## Configuration

Copy `.env.example` to `.env` and modify according to your needs:

```bash
# Server configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=3000

# Database (uses SQLite in memory by default)
DATABASE_URL=sqlite:./data/portfolio.db  # Persistent file
# DATABASE_URL=sqlite::memory:           # In-memory database (for tests)

# Upload directory
UPLOAD_DIR=./uploads

# API Key to protect uploads (change this value in production)
API_KEY=your-secret-api-key-change-in-production

# Log level
RUST_LOG=info
```

## Database

### Connection with DataGrip/DBeaver

1. Open your SQL client
2. New data source → SQLite
3. Database file: `data/portfolio.db` (according to your config)

### Database Schema

```sql
-- Development projects
DevProjectMetadata (
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

-- Photo albums
AlbumMetadata (
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

-- Album content
AlbumContent (
    slug TEXT NOT NULL,
    img_url TEXT NOT NULL,
    caption TEXT NOT NULL,
    img_path TEXT NOT NULL,
    PRIMARY KEY (slug, img_url),
    FOREIGN KEY (slug) REFERENCES AlbumMetadata(slug) ON DELETE CASCADE
)
```

## API Endpoints

### Development Projects

- `GET /dev-projects` - List all projects
- `GET /dev-projects/{slug}` - Project details

### Photo Albums

- `GET /albums` - List all albums
- `GET /albums/{slug}` - Album with its content

### File Upload

- `POST /upload` - Upload a file (**Authentication required**)
  - Required headers: `X-API-Key: your-api-key`
  - Form data: `slug` (string) + `file` (file)
  - Returns the file access URL
  - Automatically generates thumbnails for images

### Static Files

- `GET /files/{slug}/{filename}` - Original file
- `GET /files/{slug}/{filename}/thumb` - Thumbnail (for images)

## Usage Examples

### Get all projects

```bash
curl http://127.0.0.1:3000/dev-projects
```

### Upload an image

```bash
curl -X POST \
  -H "X-API-Key: your-secret-api-key-change-in-production" \
  -F "slug=my-album" \
  -F "file=@photo.jpg" \
  http://127.0.0.1:3000/upload
```

### Access a file

```
http://127.0.0.1:3000/files/my-album/photo.jpg
http://127.0.0.1:3000/files/my-album/photo.thumb.jpg
```

## Frontend Integration

The server is designed to be used with any frontend. Use the API endpoints to retrieve JSON data.

Example with fetch:

```javascript
// Get projects
const projects = await fetch("http://127.0.0.1:3000/dev-projects").then((r) =>
  r.json()
);

// Get an album
const album = await fetch(
  "http://127.0.0.1:3000/albums/urban-exploration"
).then((r) => r.json());

// Upload a file
const formData = new FormData();
formData.append("slug", "my-album");
formData.append("file", fileInput.files[0]);

const response = await fetch("http://127.0.0.1:3000/upload", {
  method: "POST",
  headers: {
    "X-API-Key": "your-secret-api-key-change-in-production",
  },
  body: formData,
});
```

## Deployment

### Development

```bash
cargo run
```

### Production (Native)

```bash
cargo build --release
RUST_LOG=warn ./target/release/portfolio-server
```

### Docker Deployment

#### Quick Start with Docker Compose

```bash
# Clone and configure
git clone <your-repo>
cd PortfolioContent

# Configure environment (edit as needed)
cp .env.docker .env

# Build and run with Docker Compose
docker-compose up -d

# View logs
docker-compose logs -f portfolio-server
```

#### Manual Docker Build

```bash
# Build the image
docker build -t portfolio-server .

# Run the container
docker run -d \
  --name portfolio-server \
  -p 3000:3000 \
  -e SERVER_HOST=0.0.0.0 \
  -e API_KEY=your-production-api-key \
  -v portfolio_data:/app/data \
  -v portfolio_uploads:/app/uploads \
  portfolio-server
```

#### Docker Features

- **Multi-stage build** for optimized image size
- **Non-root user** for security
- **Health checks** for container monitoring
- **Persistent volumes** for data and uploads
- **Alpine Linux** base for minimal footprint

## Advanced Features

- **Automatic thumbnails**: Uploaded images automatically generate 300x300px thumbnails
- **Unique UUIDs**: Each uploaded file receives a unique identifier to avoid conflicts
- **MIME validation**: Automatic file type detection
- **Configured CORS**: Ready for integration with web frontends
- **Structured logs**: Uses `tracing` for professional logging

## Recommended Workflow

1. **Development**: Use in-memory database (`sqlite::memory:`)
2. **Testing**: Local file database (`sqlite:./test.db`)
3. **Production**: Persistent file database with backups

The server is optimized to be **ultra-lightweight** and **simple to deploy**. Perfect for a personal portfolio with low traffic.

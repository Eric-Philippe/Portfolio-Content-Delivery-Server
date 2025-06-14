# Portfolio - Content Delivery Server

A lightweight Rust server to manage and serve your portfolio content (development projects and photo albums).

> This super lightweight server is designed to serve as a content provider for my portfolio website. Like this I can easily manage my photo hosting, the albums and development projects without needing a full CMS or complex backend. It provides a simple REST API to retrieve projects and albums, supports file uploads with automatic thumbnail generation, and uses PostgreSQL for data storage everything under secure and efficient conditions.

> The file upload feature is not meant to stay, it was more done as a proof of concept. I'd like to implement my own solution for easy/free photo hosting with a proper UI in the future and all features you can expect from a a file "sharing" service. With the wish to make it self-hostable and open source, like so everyone will be able to use it without having also the "My portfolio" specific features.

## Features

- **Simple REST API** to retrieve projects and albums
- **File upload** with automatic thumbnail generation for images
- **PostgreSQL database** with robust relational features
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

# Database
DATABASE_URL=postgresql://portfolio_user:portfolio_password@localhost:5432/portfolio

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
2. New data source → PostgreSQL
3. Host: `localhost`, Port: `5432`
4. Database: `portfolio`
5. User: `portfolio_user`, Password: `portfolio_password`

### Setting up PostgreSQL

Before running the server, make sure PostgreSQL is installed and running:

```bash
# On Ubuntu/Debian
sudo apt update
sudo apt install postgresql postgresql-contrib

# On macOS with Homebrew
brew install postgresql
brew services start postgresql

# Create database and user
sudo -u postgres psql
CREATE DATABASE portfolio;
CREATE USER portfolio_user WITH PASSWORD 'portfolio_password';
GRANT ALL PRIVILEGES ON DATABASE portfolio TO portfolio_user;
\q
```

### Database Schema

```sql
-- Development projects
Dev_Project_Metadata (
    slug VARCHAR(255) PRIMARY KEY,
    en_title VARCHAR(500) NOT NULL,
    en_short_description TEXT NOT NULL,
    fr_title VARCHAR(500) NOT NULL,
    fr_short_description TEXT NOT NULL,
    techs TEXT NOT NULL,
    link VARCHAR(1000) NOT NULL,
    date VARCHAR(50) NOT NULL,
    tags TEXT NOT NULL
)

-- Photo albums
Album_Metadata (
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

-- Album content
Album_Content (
    slug VARCHAR(255) NOT NULL,
    img_url VARCHAR(1000) NOT NULL,
    caption TEXT NOT NULL,
    img_path VARCHAR(1000) NOT NULL,
    PRIMARY KEY (slug, img_url),
    FOREIGN KEY (slug) REFERENCES Album_Metadata(slug) ON DELETE CASCADE
)
```

## API Endpoints

### Development Projects

- `GET /dev-projects` - List all projects
- `GET /dev-projects/{slug}` - Project details

### Photo Albums

- `GET /albums` - List all albums (with their content)
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

1. **Development**: Use local PostgreSQL instance with test database
2. **Testing**: Local PostgreSQL database (`postgresql://localhost:5432/test_db`)
3. **Production**: Remote PostgreSQL instance with proper backup strategy

The server is optimized to be **ultra-lightweight** and **simple to deploy**. Perfect for a personal portfolio with low traffic while benefiting from PostgreSQL's robust features and performance.

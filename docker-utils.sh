#!/bin/bash

# Portfolio Server Docker Management Script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_blue() {
    echo -e "${BLUE}[DOCKER]${NC} $1"
}

# Check if Docker is installed
check_docker() {
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed or not in PATH"
        exit 1
    fi
    
    if ! command -v docker-compose &> /dev/null; then
        print_error "Docker Compose is not installed or not in PATH"
        exit 1
    fi
}

# Build the Docker image
build() {
    print_status "Building Portfolio Server Docker image..."
    docker build -t portfolio-server .
    print_status "Build completed successfully!"
}

# Start the application with Docker Compose
start() {
    print_status "Starting Portfolio Server with Docker Compose..."
    
    if [ ! -f .env ]; then
        print_warning ".env file not found, copying from .env.docker"
        cp .env.docker .env
        print_warning "Please edit .env file with your configuration"
    fi
    
    docker-compose up -d
    print_status "Portfolio Server started successfully!"
    print_blue "Server available at: http://localhost:3000"
    print_blue "Swagger UI available at: http://localhost:3000/swagger-ui"
}

# Stop the application
stop() {
    print_status "Stopping Portfolio Server..."
    docker-compose down
    print_status "Portfolio Server stopped!"
}

# Restart the application
restart() {
    print_status "Restarting Portfolio Server..."
    docker-compose down
    docker-compose up -d
    print_status "Portfolio Server restarted!"
}

# View logs
logs() {
    print_status "Showing Portfolio Server logs (Ctrl+C to exit)..."
    docker-compose logs -f portfolio-server
}

# Clean up (remove containers, images, volumes)
clean() {
    print_warning "This will remove all containers, images, and volumes. Are you sure? (y/N)"
    read -r response
    if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
        print_status "Cleaning up Docker resources..."
        docker-compose down -v --rmi all
        docker system prune -f
        print_status "Cleanup completed!"
    else
        print_status "Cleanup cancelled."
    fi
}

# Check status
status() {
    print_status "Portfolio Server Status:"
    docker-compose ps
    echo ""
    print_status "Docker Images:"
    docker images | grep -E "(portfolio|REPOSITORY)"
    echo ""
    print_status "Volumes:"
    docker volume ls | grep -E "(portfolio|DRIVER)"
}

# Run database initialization
init_db() {
    print_status "Initializing database with sample data..."
    docker-compose exec postgres psql -U portfolio_user -d portfolio_db < ./sample_data.sql
    print_status "Database initialized with sample data!"
}

# Backup data
backup() {
    BACKUP_DIR="./backups/$(date +%Y%m%d_%H%M%S)"
    mkdir -p "$BACKUP_DIR"
    
    print_status "Creating backup in $BACKUP_DIR..."
    
    # Backup database
    docker-compose exec postgres pg_dump -U portfolio_user portfolio_db > "$BACKUP_DIR/portfolio_db_backup.sql"
    
    # Backup uploads
    docker cp "$(docker-compose ps -q portfolio-server):/uploads" "$BACKUP_DIR/"
    
    print_status "Backup completed: $BACKUP_DIR"
}

# Show usage
usage() {
    echo "Portfolio Server Docker Management"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  build     Build the Docker image"
    echo "  start     Start the application with Docker Compose"
    echo "  stop      Stop the application"
    echo "  restart   Restart the application"
    echo "  logs      View application logs"
    echo "  status    Show status of containers and resources"
    echo "  init-db   Initialize database with sample data"
    echo "  backup    Create backup of data and uploads"
    echo "  clean     Remove all Docker resources (WARNING: destructive)"
    echo "  help      Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 start                 # Start the application"
    echo "  $0 logs                  # View logs"
    echo "  $0 backup                # Create backup"
}

# Main script logic
main() {
    check_docker
    
    case "${1:-help}" in
        build)
            build
            ;;
        start)
            start
            ;;
        stop)
            stop
            ;;
        restart)
            restart
            ;;
        logs)
            logs
            ;;
        status)
            status
            ;;
        init-db)
            init_db
            ;;
        backup)
            backup
            ;;
        clean)
            clean
            ;;
        help|--help|-h)
            usage
            ;;
        *)
            print_error "Unknown command: $1"
            echo ""
            usage
            exit 1
            ;;
    esac
}

main "$@"

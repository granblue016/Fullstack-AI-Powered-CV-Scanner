# Docker Deployment Guide for AI Resume Scanner

## Quick Start

### Prerequisites
- Docker Desktop installed ([Download](https://www.docker.com/products/docker-desktop))
- GEMINI_API_KEY environment variable set

### Build and Run with Docker Compose

#### 1. **Create .env file in root directory**
```bash
# Create backend/.env with your Gemini API key
GEMINI_API_KEY=your-api-key-here
PORT=3001
```

#### 2. **Start all services**
```bash
docker-compose up
```

This command will:
- Build the backend Docker image
- Build the frontend Docker image
- Start both containers
- Set up networking between them
- Expose:
  - Frontend: http://localhost:3000
  - Backend API: http://localhost:3001

#### 3. **Access the application**
- Open your browser to `http://localhost:3000`

### Useful Docker Commands

#### Build only (without running)
```bash
docker-compose build
```

#### Run in background (detached mode)
```bash
docker-compose up -d
```

#### Stop all services
```bash
docker-compose down
```

#### Stop and remove all data
```bash
docker-compose down -v
```

#### View logs
```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f backend
docker-compose logs -f frontend
```

#### Rebuild after code changes
```bash
docker-compose up --build
```

#### Run specific service only
```bash
docker-compose up backend
docker-compose up frontend
```

### Development with Docker

#### Hot reload for backend
The backend container supports volume mounting of source code:
```bash
docker-compose up
# Edit src files, they will be automatically compiled
```

#### Access container shell
```bash
# Backend
docker-compose exec backend /bin/sh

# Frontend
docker-compose exec frontend sh
```

### Production Deployment

#### Build images with tags
```bash
docker build -t cv-scanner-backend:v1.0 ./backend
docker build -t cv-scanner-frontend:v1.0 ./frontend
```

#### Push to registry
```bash
docker tag cv-scanner-backend:v1.0 your-registry/cv-scanner-backend:v1.0
docker push your-registry/cv-scanner-backend:v1.0

docker tag cv-scanner-frontend:v1.0 your-registry/cv-scanner-frontend:v1.0
docker push your-registry/cv-scanner-frontend:v1.0
```

#### Deploy to cloud
- Update image names in docker-compose.yml
- Deploy using: `docker-compose up -d`

### Troubleshooting

#### Port already in use
```bash
# Change ports in docker-compose.yml
# Or kill existing process:
lsof -i :3000  # Find process on port 3000
kill -9 <PID>
```

#### Build failures
```bash
# Clean build
docker-compose down -v
docker-compose build --no-cache
docker-compose up
```

#### Network issues
```bash
# Inspect network
docker network ls
docker network inspect cv-scanner-ai-network

# Rebuild network
docker-compose down
docker-compose up
```

### Environment Variables

Backend requires:
- `GEMINI_API_KEY` - Your Google Gemini API key (Required)
- `PORT` - Server port (default: 3001)
- `RUST_LOG` - Log level (default: info)

Frontend uses:
- `PUBLIC_API_URL` - Backend API URL (must be http://backend:3001 inside Docker)

### Health Checks

Both services include health checks:
```bash
# Check service health
docker-compose ps

# Output will show STATUS: "healthy" or "unhealthy"
```

### Performance Optimization

#### Build stage optimization
- Multi-stage builds reduce final image size
- Only production dependencies included
- Unused build tools removed from runtime images

#### Layer caching
- Docker cache is used efficiently
- Rebuild only changed layers
- Use `.dockerignore` to exclude unnecessary files

### Monitoring

```bash
# View resource usage
docker stats

# View detailed logs with timestamps
docker-compose logs --timestamps -f
```

## Support

For issues or questions, refer to the main README.md or open an issue on GitHub.

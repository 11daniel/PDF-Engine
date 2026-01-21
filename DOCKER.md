# Running pdfsnap-server with Docker

## Prerequisites

- Docker and Docker Compose installed
- S3 credentials and configuration

## Quick Start

1. **Create a `.env` file** in the project root with the following variables:

```env
# Required S3 Configuration
S3_ACCESS_KEY=your_access_key
S3_SECRET_KEY=your_secret_key
S3_ENDPOINT=https://your-s3-endpoint.com
S3_BUCKET=your-bucket-name
S3_PUBLIC_URL_FORMAT=https://your-bucket-url.com/{key}

# Optional Configuration
HOST=0.0.0.0
PORT=6970
RUST_LOG=info
```

2. **Build and run with Docker Compose:**

```bash
docker-compose up --build
```

Or to run in detached mode:

```bash
docker-compose up -d --build
```

3. **The server will be available at:** `http://localhost:6970`

## Using Docker directly (without docker-compose)

1. **Build the image:**

```bash
docker build -t pdfsnap-server .
```

2. **Run the container:**

```bash
docker run -d \
  --name pdfsnap-server \
  -p 6970:6970 \
  -e S3_ACCESS_KEY=your_access_key \
  -e S3_SECRET_KEY=your_secret_key \
  -e S3_ENDPOINT=https://your-s3-endpoint.com \
  -e S3_BUCKET=your-bucket-name \
  -e S3_PUBLIC_URL_FORMAT=https://your-bucket-url.com/{key} \
  -e RUST_LOG=info \
  pdfsnap-server
```

Or use a `.env` file:

```bash
docker run -d \
  --name pdfsnap-server \
  -p 6970:6970 \
  --env-file .env \
  pdfsnap-server
```

## Useful Commands

- **View logs:**
  ```bash
  docker-compose logs -f
  ```

- **Stop the container:**
  ```bash
  docker-compose down
  ```

- **Rebuild after code changes:**
  ```bash
  docker-compose up --build
  ```

- **Check health:**
  ```bash
  curl http://localhost:6970/healthcheck
  ```

## Troubleshooting

### I/O Errors During Build

If you encounter I/O errors like "Input/output error (os error 5)" during the Docker build:

1. **Clean Docker build cache:**
   ```bash
   docker builder prune -a
   ```

2. **Check disk space:**
   ```bash
   df -h
   ```
   Ensure you have at least 10GB free space.

3. **Restart Docker Desktop:**
   - Quit Docker Desktop completely
   - Restart it
   - Wait for it to fully start

4. **Try building again:**
   ```bash
   docker-compose build --no-cache
   ```

5. **If issues persist, try building directly:**
   ```bash
   docker build --no-cache -t pdfsnap-server .
   ```

### Build Takes Too Long

The first build will take 5-10 minutes as it compiles all Rust dependencies. Subsequent builds will be faster due to Docker layer caching.

## Environment Variables

### Required:
- `S3_ACCESS_KEY` - S3 access key
- `S3_SECRET_KEY` - S3 secret key
- `S3_ENDPOINT` - S3 endpoint URL
- `S3_BUCKET` - S3 bucket name
- `S3_PUBLIC_URL_FORMAT` - Format string for public URLs (use `{key}` placeholder)

### Optional:
- `HOST` - Server host (default: `0.0.0.0`)
- `PORT` - Server port (default: `6970`)
- `RUST_LOG` - Log level (default: `info`)


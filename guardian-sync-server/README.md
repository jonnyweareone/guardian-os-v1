# Guardian Sync Server

Self-hosted synchronization server for Guardian OS.

## Features

- **Self-hosted Authentication**: Email/password registration with device tokens
- **Settings Sync**: Synchronize desktop settings across devices
- **File Sync**: Upload/download wallpapers, themes, configs via S3
- **Family Management**: Parental controls and family groups

## Requirements

- Rust 1.75+
- MySQL/MariaDB 8.0+
- Redis 7.0+
- S3-compatible storage (Peasoup, MinIO, AWS)

## Configuration

Copy `.env.example` to `.env` and configure:

```env
# Server
GRPC_PORT=50051
HTTP_PORT=8080

# Database
DB_HOST=127.0.0.1
DB_PORT=3306
DB_NAME=guardian_sync
DB_USER=guardian
DB_PASS=your_password

# Redis
REDIS_HOST=127.0.0.1
REDIS_PORT=6379
REDIS_PASSWORD=your_redis_password

# S3 Storage
S3_ENDPOINT=https://s3.eu-west-1.peasoup.cloud
S3_REGION=eu-west-1
S3_BUCKET=guardian-sync-files
S3_ACCESS_KEY=your_access_key
S3_SECRET_KEY=your_secret_key

# JWT
JWT_SECRET=your_64_char_secret
JWT_ACCESS_EXPIRY_HOURS=24
JWT_REFRESH_EXPIRY_DAYS=30

# Domain
DOMAIN=sync.gameguardian.ai
```

## Building

```bash
# Development
cargo build

# Release
cargo build --release
```

## Running

```bash
# Development
cargo run

# Production
./target/release/guardian-sync-server
```

## Docker

```bash
# Build image
docker build -t guardian-sync-server .

# Run
docker run -d \
  --name guardian-sync \
  -p 50051:50051 \
  -p 8080:8080 \
  --env-file .env \
  guardian-sync-server
```

## API

gRPC services on port 50051:
- `AuthService` - Authentication and device management
- `SyncService` - Settings synchronization
- `FileService` - File upload/download
- `FamilyService` - Family and parental controls

## Proto Files

See `proto/` directory for service definitions:
- `auth.proto` - Authentication
- `sync.proto` - Settings sync
- `file.proto` - File management
- `family.proto` - Family features

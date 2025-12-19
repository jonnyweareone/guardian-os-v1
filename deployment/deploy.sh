#!/bin/bash
# Guardian Sync Server - Vultr Deployment Script
# Run on fresh Ubuntu 24.04 VPS

set -e

echo "========================================"
echo "Guardian Sync Server - Vultr Deployment"
echo "========================================"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check if running as root
if [ "$EUID" -ne 0 ]; then
  echo -e "${RED}Please run as root${NC}"
  exit 1
fi

# Variables
DEPLOY_DIR="/opt/guardian-sync"
DOMAIN=${1:-"guardian-os.com"}

echo -e "${YELLOW}Domain: $DOMAIN${NC}"

# ==========================================
# 1. System Updates
# ==========================================
echo -e "${GREEN}[1/6] Updating system...${NC}"
apt-get update
apt-get upgrade -y

# ==========================================
# 2. Install Docker
# ==========================================
echo -e "${GREEN}[2/6] Installing Docker...${NC}"
if ! command -v docker &> /dev/null; then
  curl -fsSL https://get.docker.com | sh
  systemctl enable docker
  systemctl start docker
fi

# Install Docker Compose plugin
apt-get install -y docker-compose-plugin

# ==========================================
# 3. Configure Firewall
# ==========================================
echo -e "${GREEN}[3/6] Configuring firewall...${NC}"
apt-get install -y ufw
ufw default deny incoming
ufw default allow outgoing
ufw allow ssh
ufw allow 80/tcp
ufw allow 443/tcp
ufw allow 50051/tcp
ufw --force enable

# ==========================================
# 4. Create deployment directory
# ==========================================
echo -e "${GREEN}[4/6] Setting up deployment...${NC}"
mkdir -p $DEPLOY_DIR
cd $DEPLOY_DIR

# Copy files (assuming they're in current directory or passed via scp)
if [ -f "./docker-compose.yml" ]; then
  cp ./docker-compose.yml $DEPLOY_DIR/
  cp ./.env $DEPLOY_DIR/ 2>/dev/null || true
  cp -r ./init-db $DEPLOY_DIR/ 2>/dev/null || true
fi

# ==========================================
# 5. Generate secrets if .env doesn't exist
# ==========================================
echo -e "${GREEN}[5/6] Configuring environment...${NC}"
if [ ! -f "$DEPLOY_DIR/.env" ]; then
  echo -e "${YELLOW}Creating .env file with generated secrets...${NC}"
  
  DB_ROOT_PASS=$(openssl rand -base64 32 | tr -dc 'a-zA-Z0-9' | head -c 32)
  DB_PASS=$(openssl rand -base64 32 | tr -dc 'a-zA-Z0-9' | head -c 32)
  REDIS_PASS=$(openssl rand -base64 32 | tr -dc 'a-zA-Z0-9' | head -c 32)
  
  cat > $DEPLOY_DIR/.env <<EOF
# Guardian Sync Server - Production Environment
# Generated on $(date)

DOMAIN=$DOMAIN
ACME_EMAIL=admin@$DOMAIN

# Database
DB_ROOT_PASSWORD=$DB_ROOT_PASS
DB_NAME=guardian_sync
DB_USER=guardian
DB_PASS=$DB_PASS

# Redis
REDIS_PASSWORD=$REDIS_PASS

# S3 - CONFIGURE THESE
S3_REGION=ewr1
S3_BUCKET=guardian-sync-files
S3_ENDPOINT=https://ewr1.vultrobjects.com
S3_ACCESS_KEY=YOUR_VULTR_S3_ACCESS_KEY
S3_SECRET_KEY=YOUR_VULTR_S3_SECRET_KEY

# OAuth - CONFIGURE THESE
OAUTH_CLIENT_ID=your_oauth_client_id
OAUTH_CLIENT_SECRET=your_oauth_client_secret
OAUTH_AUTH_URL=https://accounts.google.com/o/oauth2/v2/auth
OAUTH_TOKEN_URL=https://oauth2.googleapis.com/token
OAUTH_USER_INFO_URL=https://openidconnect.googleapis.com/v1/userinfo
EOF

  echo -e "${YELLOW}⚠️  IMPORTANT: Edit $DEPLOY_DIR/.env and configure S3 and OAuth settings${NC}"
fi

# ==========================================
# 6. Start services
# ==========================================
echo -e "${GREEN}[6/6] Starting services...${NC}"
cd $DEPLOY_DIR

# Pull and start
docker compose pull
docker compose up -d

# Wait for services
echo "Waiting for services to start..."
sleep 10

# Check status
docker compose ps

echo ""
echo -e "${GREEN}========================================"
echo "Deployment complete!"
echo "========================================"
echo ""
echo "Services:"
echo "  - Traefik:  https://$DOMAIN"
echo "  - gRPC:     sync.$DOMAIN:50051"
echo ""
echo "Next steps:"
echo "  1. Point DNS: sync.$DOMAIN -> $(curl -s ifconfig.me)"
echo "  2. Edit .env: nano $DEPLOY_DIR/.env"
echo "  3. Configure S3 and OAuth settings"
echo "  4. Restart: cd $DEPLOY_DIR && docker compose restart"
echo ""
echo "Logs: docker compose logs -f"
echo -e "${NC}"

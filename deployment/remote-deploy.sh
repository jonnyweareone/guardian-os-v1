#!/bin/bash
# Run on the Vultr VPS

set -e

echo "========================================="
echo "Guardian Sync Server - Vultr Deployment"
echo "1GB RAM / 1 vCPU - Optimized Config"
echo "========================================="

# Update system
apt-get update
apt-get install -y curl git

# Install Docker
curl -fsSL https://get.docker.com | sh
systemctl enable docker
systemctl start docker

# Install Docker Compose
apt-get install -y docker-compose-plugin

# Configure firewall
apt-get install -y ufw
ufw default deny incoming
ufw default allow outgoing
ufw allow ssh
ufw allow 80/tcp
ufw allow 443/tcp
ufw allow 50051/tcp
ufw --force enable

# Create directory
mkdir -p /opt/guardian-sync/init-db
cd /opt/guardian-sync

# Generate passwords
DB_ROOT_PASS=$(openssl rand -base64 24 | tr -dc 'a-zA-Z0-9' | head -c 24)
DB_PASS=$(openssl rand -base64 24 | tr -dc 'a-zA-Z0-9' | head -c 24)
REDIS_PASS=$(openssl rand -base64 24 | tr -dc 'a-zA-Z0-9' | head -c 24)

echo "DB_ROOT_PASS: $DB_ROOT_PASS"
echo "DB_PASS: $DB_PASS"
echo "REDIS_PASS: $REDIS_PASS"

# Create .env
cat > /opt/guardian-sync/.env <<EOF
DOMAIN=guardian-os.net
ACME_EMAIL=admin@guardian-os.net

DB_ROOT_PASSWORD=$DB_ROOT_PASS
DB_NAME=guardian_sync
DB_USER=guardian
DB_PASS=$DB_PASS

REDIS_PASSWORD=$REDIS_PASS

S3_REGION=ewr1
S3_BUCKET=guardian-sync-files
S3_ENDPOINT=https://ewr1.vultrobjects.com
S3_ACCESS_KEY=CONFIGURE_LATER
S3_SECRET_KEY=CONFIGURE_LATER

OAUTH_CLIENT_ID=CONFIGURE_LATER
OAUTH_CLIENT_SECRET=CONFIGURE_LATER
OAUTH_AUTH_URL=https://accounts.google.com/o/oauth2/v2/auth
OAUTH_TOKEN_URL=https://oauth2.googleapis.com/token
OAUTH_USER_INFO_URL=https://openidconnect.googleapis.com/v1/userinfo
EOF

echo "✅ .env created"
echo "========================================="
echo "Now upload docker-compose.yml and init-db/"
echo "Then run: cd /opt/guardian-sync && docker compose up -d"
echo "========================================="

#!/bin/bash
# Guardian OS Endpoint Test Script
# Tests the Supabase endpoints used by the installer

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# API Base URL
API_BASE="https://xzxjwuzwltoapifcyzww.supabase.co/functions/v1"

echo "Guardian OS Endpoint Test"
echo "========================="
echo ""
echo "API Base: $API_BASE"
echo ""

# Test connectivity
echo -n "Testing connectivity to Supabase... "
if curl -s -I "$API_BASE/../" > /dev/null 2>&1; then
    echo -e "${GREEN}OK${NC}"
else
    echo -e "${RED}FAILED${NC}"
    echo "Cannot connect to Supabase"
    exit 1
fi

# Test auth-login endpoint
echo ""
echo "Testing auth-login endpoint:"
echo "POST $API_BASE/auth-login"
RESPONSE=$(curl -sS -X POST \
    "$API_BASE/auth-login" \
    -H 'Content-Type: application/json' \
    -d '{"email":"test@example.com","password":"WrongPassword123!"}' \
    2>&1) || true

if echo "$RESPONSE" | grep -q "error\|Error"; then
    echo -e "${GREEN}Endpoint responding (auth failed as expected)${NC}"
else
    echo -e "${YELLOW}Response: ${RESPONSE:0:100}...${NC}"
fi

# Test auth-register endpoint
echo ""
echo "Testing auth-register endpoint:"
echo "POST $API_BASE/auth-register"
RESPONSE=$(curl -sS -X POST \
    "$API_BASE/auth-register" \
    -H 'Content-Type: application/json' \
    -d '{"email":"test'$(date +%s)'@example.com","password":"TestPass123!"}' \
    2>&1) || true

if echo "$RESPONSE" | grep -q "parent_access_token\|error"; then
    echo -e "${GREEN}Endpoint responding${NC}"
else
    echo -e "${YELLOW}Response: ${RESPONSE:0:100}...${NC}"
fi

# Test bind-device endpoint (should require auth)
echo ""
echo "Testing bind-device endpoint (without auth):"
echo "POST $API_BASE/bind-device"
RESPONSE=$(curl -sS -X POST \
    "$API_BASE/bind-device" \
    -H 'Content-Type: application/json' \
    -d '{"device_fingerprint":"sha256:test","parent_email":"test@example.com"}' \
    2>&1) || true

if echo "$RESPONSE" | grep -q "unauthorized\|Unauthorized\|error"; then
    echo -e "${GREEN}Endpoint requires auth (correct)${NC}"
else
    echo -e "${YELLOW}Response: ${RESPONSE:0:100}...${NC}"
fi

# Test device-heartbeat endpoint
echo ""
echo "Testing device-heartbeat endpoint (without JWT):"
echo "POST $API_BASE/device-heartbeat"
RESPONSE=$(curl -sS -X POST \
    "$API_BASE/device-heartbeat" \
    -H 'Content-Type: application/json' \
    -d '{"status":"online","versions":{"os":"test"}}' \
    2>&1) || true

if echo "$RESPONSE" | grep -q "error\|unauthorized"; then
    echo -e "${GREEN}Endpoint responding (auth check working)${NC}"
else
    echo -e "${YELLOW}Response: ${RESPONSE:0:100}...${NC}"
fi

echo ""
echo "================================"
echo -e "${GREEN}Endpoint connectivity test complete${NC}"
echo ""
echo "Next steps:"
echo "1. Deploy Edge Functions if not already done:"
echo "   supabase functions deploy auth-login"
echo "   supabase functions deploy auth-register"
echo ""
echo "2. Test with real credentials:"
echo "   EMAIL='your-test@example.com'"
echo "   PASSWORD='YourPassword123!'"
echo "   curl -X POST '$API_BASE/auth-login' \\"
echo "     -H 'Content-Type: application/json' \\"
echo "     -d '{\"email\":\"'\$EMAIL'\",\"password\":\"'\$PASSWORD'\"}'"

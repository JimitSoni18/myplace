#!/bin/sh
set -e

# =============================================================================
# Garage S3 Initialization & Provisioning Script
# =============================================================================
# This script idempotently initializes a Garage instance:
# 1. Configures cluster layout (if single-node dev)
# 2. Creates the S3 access key (if not already existing)
# 3. Creates the S3 bucket (if not already existing)
# 4. Grants read/write permissions on the bucket to the key
# 5. Enables public web access on the bucket for browser-accessible assets
#
# Usage:
#   # Run locally targeting Docker/Podman container:
#   ./scripts/setup-garage.sh
#
#   # Or run directly inside the container / on host:
#   GARAGE_CMD="garage" ./scripts/setup-garage.sh
# =============================================================================

BUCKET_NAME="${S3_BUCKET:-myplace}"
KEY_NAME="${KEY_NAME:-myplace-app-key}"
CONTAINER_NAME="${GARAGE_CONTAINER:-myplace_s3_1}"

# Determine command prefix
if [ -n "$GARAGE_CMD" ]; then
    CMD="$GARAGE_CMD"
elif command -v podman >/dev/null 2>&1 && podman ps --format "{{.Names}}" | grep -q "^${CONTAINER_NAME}$"; then
    CMD="podman exec $CONTAINER_NAME garage"
elif command -v docker >/dev/null 2>&1 && docker ps --format "{{.Names}}" | grep -q "^${CONTAINER_NAME}$"; then
    CMD="docker exec $CONTAINER_NAME garage"
elif command -v garage >/dev/null 2>&1; then
    CMD="garage"
else
    echo "Error: Neither podman/docker container '${CONTAINER_NAME}' nor local 'garage' CLI found."
    echo "Start your containers with 'docker compose up -d' first, or set GARAGE_CMD."
    exit 1
fi

echo "==> Using Garage CLI: $CMD"

# 1. Ensure node layout is initialized (required for fresh single-node Garage instances)
echo "==> Checking cluster layout..."
NODE_STATUS=$($CMD status 2>&1 || true)
if echo "$NODE_STATUS" | grep -q "NO ROLE ASSIGNED"; then
    NODE_ID=$($CMD node id -q 2>/dev/null || echo "$NODE_STATUS" | awk '/NO ROLE ASSIGNED/{print $1}' | head -n1)
    if [ -n "$NODE_ID" ]; then
        echo "==> Assigning role to node $NODE_ID..."
        $CMD layout assign -z dc1 -c 1G "$NODE_ID"
        $CMD layout apply --version 1
    fi
fi

# 2. Check / create S3 Access Key
echo "==> Ensuring access key '$KEY_NAME' exists..."
KEY_INFO=$($CMD key info "$KEY_NAME" 2>&1 || true)
if echo "$KEY_INFO" | grep -qi "Key ID:"; then
    echo "==> Key '$KEY_NAME' already exists."
    KEY_ID=$(echo "$KEY_INFO" | awk -F: '/Key ID:/{print $2}' | tr -d ' ')
else
    echo "==> Creating access key '$KEY_NAME'..."
    CREATE_OUT=$($CMD key create "$KEY_NAME")
    echo "$CREATE_OUT"
    KEY_ID=$(echo "$CREATE_OUT" | awk -F: '/Key ID:/{print $2}' | tr -d ' ')
fi

# 3. Check / create Bucket
echo "==> Ensuring bucket '$BUCKET_NAME' exists..."
BUCKET_INFO=$($CMD bucket info "$BUCKET_NAME" 2>&1 || true)
if echo "$BUCKET_INFO" | grep -qi "Bucket:"; then
    echo "==> Bucket '$BUCKET_NAME' already exists."
else
    echo "==> Creating bucket '$BUCKET_NAME'..."
    $CMD bucket create "$BUCKET_NAME" || true
fi

# 4. Grant read & write permissions on bucket to the key
echo "==> Authorizing key '$KEY_NAME' for bucket '$BUCKET_NAME'..."
$CMD bucket allow --read --write "$BUCKET_NAME" --key "$KEY_NAME"

# 5. Enable public website access for browser asset delivery
echo "==> Enabling public web access on bucket '$BUCKET_NAME'..."
$CMD bucket website --allow "$BUCKET_NAME"

echo ""
echo "============================================================================="
echo "Garage setup completed successfully!"
echo "Bucket Name:  $BUCKET_NAME"
echo "Key Name:     $KEY_NAME"
if [ -n "$KEY_ID" ]; then
    echo "Key ID:       $KEY_ID"
fi
echo "============================================================================="

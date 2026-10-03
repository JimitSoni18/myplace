#!/bin/sh
set -e

# =============================================================================
# Ed25519 Key Generation Script for MyPlace
# =============================================================================
# Generates PKCS#8 Ed25519 private and public keys in PEM format if not present.
# =============================================================================

if [ ! -f private.pem ]; then
    echo "==> Generating Ed25519 private key (private.pem)..."
    openssl genpkey -algorithm ed25519 -out private.pem
    chmod 600 private.pem
fi

if [ ! -f public.pem ]; then
    echo "==> Deriving Ed25519 public key (public.pem)..."
    openssl pkey -in private.pem -pubout -out public.pem
fi

echo "==> Keys ready:"
ls -l private.pem public.pem

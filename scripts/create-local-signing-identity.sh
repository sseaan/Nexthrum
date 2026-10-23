#!/usr/bin/env bash
# Create a local self-signed code-signing identity so macOS Keychain grants
# persist across rebuilds.
#
# Why: with ad-hoc signing (`CODE_SIGN_IDENTITY=-`) every build gets a new
# signature, so macOS treats each build as a different app and re-prompts for
# Keychain access. A stable signing identity fixes that — no Apple Developer
# account required, but the app is still not notarized.
#
# Usage:
#   scripts/create-local-signing-identity.sh            # create identity if missing
#   SIGN_IDENTITY="Nexthrum Local Signing" scripts/package-macos-dmg.sh
set -euo pipefail

IDENTITY_NAME="${1:-Nexthrum Local Signing}"
KEYCHAIN="${HOME}/Library/Keychains/login.keychain-db"

if security find-identity -v -p codesigning "$KEYCHAIN" 2>/dev/null | grep -Fq "$IDENTITY_NAME"; then
  echo "Signing identity already present: $IDENTITY_NAME"
  exit 0
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

cat > "$WORK/cert.cnf" <<'CONF'
[ req ]
distinguished_name = dn
x509_extensions = ext
prompt = no
[ dn ]
CN = Nexthrum Local Signing
[ ext ]
basicConstraints = critical,CA:false
keyUsage = critical,digitalSignature
extendedKeyUsage = critical,codeSigning
CONF

echo "==> Generating self-signed code-signing certificate"
openssl req -x509 -newkey rsa:2048 -nodes \
  -keyout "$WORK/key.pem" -out "$WORK/cert.pem" \
  -days 3650 -config "$WORK/cert.cnf" >/dev/null 2>&1

openssl pkcs12 -export \
  -inkey "$WORK/key.pem" -in "$WORK/cert.pem" \
  -name "$IDENTITY_NAME" -out "$WORK/identity.p12" -passout pass: >/dev/null 2>&1

echo "==> Importing into the login keychain (you may be asked for your password)"
security import "$WORK/identity.p12" -k "$KEYCHAIN" -P "" -T /usr/bin/codesign >/dev/null
security set-key-partition-list -S apple-tool:,apple: -s -k "" "$KEYCHAIN" >/dev/null 2>&1 || true

echo "==> Trusting the certificate for code signing (confirm the prompt if shown)"
security add-trusted-cert -d -r trustRoot -p codeSign -k "$KEYCHAIN" "$WORK/cert.pem" >/dev/null 2>&1 || \
  echo "warning: could not set trust automatically; approve it in Keychain Access → Certificates"

echo
echo "Created identity: $IDENTITY_NAME"
echo "Now package with it so the signature (and Keychain grants) stay stable:"
echo "  SIGN_IDENTITY=\"$IDENTITY_NAME\" scripts/package-macos-dmg.sh"

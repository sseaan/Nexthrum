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

# macOS cannot read modern PKCS#12 (AES/PBKDF2) with an empty passphrase, so use
# legacy encoding plus a throwaway passphrase.
P12_PASSWORD="$(openssl rand -hex 16)"
openssl pkcs12 -export -legacy \
  -inkey "$WORK/key.pem" -in "$WORK/cert.pem" \
  -name "$IDENTITY_NAME" -out "$WORK/identity.p12" \
  -passout "pass:$P12_PASSWORD" >/dev/null 2>&1

echo "==> Importing into the login keychain"
security import "$WORK/identity.p12" -k "$KEYCHAIN" -P "$P12_PASSWORD" -T /usr/bin/codesign >/dev/null

echo "==> Trusting the certificate for code signing"
# User-domain trust avoids needing sudo; codesign only lists trusted identities.
security add-trusted-cert -r trustRoot -p codeSign -k "$KEYCHAIN" "$WORK/cert.pem" \
  || echo "warning: could not set trust automatically; approve it in Keychain Access → Certificates"

BACKUP_DIR="${HOME}/.tauri/nexthrum-signing"
mkdir -p "$BACKUP_DIR"
chmod 700 "$BACKUP_DIR"
openssl pkcs12 -export -legacy \
  -inkey "$WORK/key.pem" -in "$WORK/cert.pem" \
  -name "$IDENTITY_NAME" -out "$BACKUP_DIR/${IDENTITY_NAME// /-}.p12" \
  -passout "pass:$P12_PASSWORD" >/dev/null 2>&1
printf '%s' "$P12_PASSWORD" > "$BACKUP_DIR/${IDENTITY_NAME// /-}.password"
chmod 600 "$BACKUP_DIR"/*

echo
echo "Created identity: $IDENTITY_NAME"
echo "Encrypted backup: $BACKUP_DIR (password in the .password file)"
echo "Now package with it so the signature (and Keychain grants) stay stable:"
echo "  SIGN_IDENTITY=\"$IDENTITY_NAME\" scripts/package-macos-dmg.sh"

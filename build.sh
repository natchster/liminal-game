#!/bin/bash
# Build the native macOS (Apple Silicon) Backrooms: a release binary and a
# signed, double-clickable Backrooms.app in native/dist/.
#
#   ./build.sh            release build + app bundle
#   ./build.sh --run      ... and launch it
#   ./build.sh --debug    debug build (faster to compile, slower to run)
set -euo pipefail
cd "$(dirname "$0")"
PROFILE=release
RUN=0
for a in "$@"; do
  case "$a" in
    --debug) PROFILE=debug ;;
    --run) RUN=1 ;;
  esac
done
if [ "$(uname -m)" != "arm64" ]; then echo "note: building on $(uname -m); the target is aarch64-apple-darwin" >&2; fi
command -v cargo >/dev/null || { echo "error: Rust (cargo) is required — https://rustup.rs" >&2; exit 1; }
if [ "$PROFILE" = release ]; then cargo build --release -p backrooms --target aarch64-apple-darwin; else cargo build -p backrooms --target aarch64-apple-darwin; fi
BIN="target/aarch64-apple-darwin/$PROFILE/backrooms"
APP="dist/Backrooms.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BIN" "$APP/Contents/MacOS/backrooms"
cp packaging/Info.plist "$APP/Contents/Info.plist"
if [ -f packaging/AppIcon.icns ]; then cp packaging/AppIcon.icns "$APP/Contents/Resources/AppIcon.icns"; fi
# ad-hoc signature (required to run on Apple Silicon)
codesign --force --deep --sign - "$APP" >/dev/null 2>&1 || echo "warning: codesign failed; the app may need 'xattr -cr' before it runs" >&2
cp "$BIN" dist/backrooms
echo "built $APP ($(du -sh "$APP" | cut -f1)) and dist/backrooms"
if [ "$RUN" = 1 ]; then open "$APP"; fi

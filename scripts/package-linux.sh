#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"
APPDIR="${DIST_DIR}/nexus-desktop.AppDir"
BIN_PATH="${ROOT_DIR}/target/release/desktop"

mkdir -p "${DIST_DIR}"

echo "Building desktop binary..."
cargo build --release --bin desktop

rm -rf "${APPDIR}"
mkdir -p "${APPDIR}/usr/bin"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/scalable/apps"

cp "${BIN_PATH}" "${APPDIR}/usr/bin/nexus-desktop"
cp "${ROOT_DIR}/assets/nexus-icon.svg" "${APPDIR}/usr/share/icons/hicolor/scalable/apps/nexus.svg"
cp "${ROOT_DIR}/assets/nexus-icon.svg" "${APPDIR}/nexus.svg"

cat > "${APPDIR}/AppRun" <<'EOF'
#!/usr/bin/env bash
HERE="$(cd "$(dirname "$0")" && pwd)"
exec "${HERE}/usr/bin/nexus-desktop" "$@"
EOF
chmod +x "${APPDIR}/AppRun"

cat > "${APPDIR}/nexus.desktop" <<'EOF'
[Desktop Entry]
Type=Application
Name=Nexus Codex
Comment=Local AI orchestration dashboard
Exec=nexus-desktop
Icon=nexus
Categories=Utility;Development;
Terminal=false
EOF

if command -v appimagetool >/dev/null 2>&1; then
  echo "Packaging AppImage..."
  appimagetool "${APPDIR}" "${DIST_DIR}/NexusCodex.AppImage"
  echo "Created ${DIST_DIR}/NexusCodex.AppImage"
else
  echo "appimagetool not found; creating tarball instead."
  tar -czf "${DIST_DIR}/nexus-desktop.AppDir.tar.gz" -C "${DIST_DIR}" "$(basename "${APPDIR}")"
  echo "Created ${DIST_DIR}/nexus-desktop.AppDir.tar.gz"
fi

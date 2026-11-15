#!/bin/sh
# Wraps a built binary in "Lab Cloud.app" and a drag-to-Applications LabCloud-macos.dmg.
# Usage: scripts/package-macos.sh <binary> <version>
set -eu
bin=$1
version=$2
out=$(mktemp -d)
app="$out/Lab Cloud.app"

mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$bin" "$app/Contents/MacOS/labcloud"

# assets/icon.png (1024 px) -> AppIcon.icns
icons="$out/AppIcon.iconset"
mkdir "$icons"
for s in 16 32 128 256 512; do
  sips -z $s $s assets/icon.png --out "$icons/icon_${s}x${s}.png" >/dev/null
  sips -z $((s * 2)) $((s * 2)) assets/icon.png --out "$icons/icon_${s}x${s}@2x.png" >/dev/null
done
iconutil -c icns "$icons" -o "$app/Contents/Resources/AppIcon.icns"
rm -rf "$icons"
cat > "$app/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleName</key><string>Lab Cloud</string>
  <key>CFBundleDisplayName</key><string>Lab Cloud</string>
  <key>CFBundleIdentifier</key><string>io.github.leoglez10.labcloud</string>
  <key>CFBundleExecutable</key><string>labcloud</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleIconFile</key><string>AppIcon</string>
  <key>CFBundleShortVersionString</key><string>$version</string>
  <key>CFBundleVersion</key><string>$version</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict></plist>
EOF
# ponytail: ad-hoc signature only; Developer ID signing and notarization need Apple certificates (PLAN Phase 8).
codesign --force --deep --sign - "$app"

ln -s /Applications "$out/Applications"
hdiutil create -volname "Lab Cloud" -srcfolder "$out" -ov -format UDZO LabCloud-macos.dmg
rm -rf "$out"

#!/usr/bin/env bash
# PC 묶음 만들기(0240, 0276 S4·E4) — build-desktop.yml이 부른다. 빌드는 끝나 있어야 한다.
#   tools/pc/package.sh <windows|macos|linux> <flavor> [store]
#   flavor: dev | release | steam | epic (묶음 이름에 들어간다)
#   store:  ''(직접 받는 판) | steam(Steam 재배포 파일을 실행 파일 옆에) | epic(EOS SDK DLL을 실행 파일 옆에, Windows)
# 묶음 안: 실행 파일 + assets(웹 전용 mp3·vendor의 .git·비공개 SDK 폴더 뺌) + 제3자 고지.
# 게임은 실행 파일 옆(또는 .app/Contents/Resources)의 assets를 찾는다(src/main.rs `assets_dir`).
# 끝나면 묶음 파일 이름을 $GITHUB_ENV의 PKG에 적는다(없으면 표준 출력).
set -euo pipefail
NAME="$1"; FLAVOR="$2"; STORE="${3:-}"
VERSION="${VERSION:-$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)}"
out() { if [ -n "${GITHUB_ENV:-}" ]; then echo "PKG=$1" >> "$GITHUB_ENV"; fi; echo "PKG=$1"; }

# 상점 재배포 파일: steamworks-sys가 빌드 때 OUT_DIR에 복사한 것(레포에 넣지 않는다, Valve SDK 계약).
steam_lib() {
  local f="$1"
  find target -path '*steamworks-sys-*/out/*' -name "$f" -print -quit
}

rm -rf stage
mkdir -p stage
cp -R assets stage/assets
rm -f stage/assets/music/*.mp3
rm -rf stage/assets/vendor/.git stage/assets/vendor/eos
cp assets/THIRD_PARTY_LICENSES.txt stage/
du -sh stage/assets

case "$NAME" in
windows)
  # 실행 파일 아이콘(pc/nail.ico, tools/icons/make_icons.py)을 rcedit로 넣는다(빌드 의존성 없이).
  curl -sSL -o rcedit.exe https://github.com/electron/rcedit/releases/download/v2.0.0/rcedit-x64.exe
  ./rcedit.exe target/release/nail.exe --set-icon pc/nail.ico \
    --set-version-string ProductName "Nail: Under the Roots" \
    --set-version-string FileDescription "Nail: Under the Roots" \
    --set-file-version "$VERSION" --set-product-version "$VERSION"
  D="Nail-Under-the-Roots"
  rm -rf "$D"; mkdir -p "$D"
  cp target/release/nail.exe "$D/Nail.exe"
  cp -R stage/assets stage/THIRD_PARTY_LICENSES.txt "$D/"
  if [ "$STORE" = steam ]; then
    lib=$(steam_lib steam_api64.dll); [ -n "$lib" ] || { echo "steam_api64.dll을 못 찾았다"; exit 1; }
    cp "$lib" "$D/"
  fi
  if [ "$STORE" = epic ]; then
    cp "$EOS_SDK_DIR/Bin/EOSSDK-Win64-Shipping.dll" "$D/"
  fi
  7z a -tzip "nail-windows-$FLAVOR.zip" "$D" > /dev/null
  ls -la "nail-windows-$FLAVOR.zip"
  out "nail-windows-$FLAVOR.zip"
  ;;
macos)
  APP="Nail.app"
  rm -rf "$APP"; mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
  cp target/release-nail-universal "$APP/Contents/MacOS/nail"
  if [ "$STORE" = steam ]; then
    # 겸용(x86_64 + arm64) dylib, 설치 이름 @loader_path/libsteam_api.dylib — 실행 파일 옆에 둔다.
    lib=$(steam_lib libsteam_api.dylib); [ -n "$lib" ] || { echo "libsteam_api.dylib을 못 찾았다"; exit 1; }
    cp "$lib" "$APP/Contents/MacOS/"
  fi
  cp -R stage/assets "$APP/Contents/Resources/assets"
  cp stage/THIRD_PARTY_LICENSES.txt "$APP/Contents/Resources/"
  cp pc/nail.icns "$APP/Contents/Resources/nail.icns"
  cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Naïl</string>
  <key>CFBundleDisplayName</key><string>Naïl: Under the Roots</string>
  <key>CFBundleIdentifier</key><string>io.github.aimenjoe.nail</string>
  <key>CFBundleExecutable</key><string>nail</string>
  <key>CFBundleIconFile</key><string>nail.icns</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>${VERSION}</string>
  <key>CFBundleVersion</key><string>${VERSION}</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>LSApplicationCategoryType</key><string>public.app-category.role-playing-games</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST
  plutil -lint "$APP/Contents/Info.plist"
  # 임시(ad-hoc) 서명: lipo로 합치면 arm64의 링커 서명이 깨져 애플 실리콘에서 켜지지 않는다.
  # 개발자 ID 서명·공증은 Apple Developer 가입 뒤(0240). 그 전에는 처음 열 때 "개인정보 보호 및 보안 → 그래도 열기".
  codesign --force --deep -s - "$APP"
  codesign --verify --verbose "$APP"
  ditto -c -k --sequesterRsrc --keepParent "$APP" "nail-macos-$FLAVOR.zip"
  ls -la "nail-macos-$FLAVOR.zip"
  out "nail-macos-$FLAVOR.zip"
  ;;
linux)
  D="Nail-Under-the-Roots"
  rm -rf "$D"; mkdir -p "$D"
  cp target/release/nail "$D/nail"
  if [ "$STORE" = steam ]; then
    # 실행 파일은 rpath $ORIGIN으로 빌드한다(build-desktop.yml) — 옆의 libsteam_api.so를 찾는다.
    lib=$(steam_lib libsteam_api.so); [ -n "$lib" ] || { echo "libsteam_api.so를 못 찾았다"; exit 1; }
    cp "$lib" "$D/"
  fi
  cp -R stage/assets stage/THIRD_PARTY_LICENSES.txt "$D/"
  tar czf "nail-linux-$FLAVOR.tar.gz" "$D"
  ls -la "nail-linux-$FLAVOR.tar.gz"
  ldd "$D/nail" | head -20
  out "nail-linux-$FLAVOR.tar.gz"
  ;;
*) echo "모르는 대상: $NAME"; exit 1 ;;
esac

#!/usr/bin/env bash
# APK → AAB(Android App Bundle). Google Play는 새 앱을 AAB로만 받는다(0295).
# cargo-quad-apk는 APK만 만든다 — 그 APK를 aapt2로 proto 형식(번들이 쓰는 꼴)으로 바꾸고,
# 번들 모듈 꼴(manifest/·dex/·lib/·assets/·res/·resources.pb)로 옮겨 담아 bundletool로 묶는다.
# 서명은 하지 않는다(올리기 키로 jarsigner — build-android.yml).
#
#   tools/android/make_aab.sh <in.apk> <out.aab> <bundletool.jar> <aapt2>
set -euo pipefail
APK=$(realpath "$1"); OUT=$(realpath -m "$2"); BUNDLETOOL=$(realpath "$3"); AAPT2=$4

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

# 1) 바이너리 XML·resources.arsc → proto(AndroidManifest.xml·resources.pb). 나머지(dex·lib·assets·res 파일)는 그대로 실린다.
"$AAPT2" convert --output-format proto -o "$WORK/proto.apk" "$APK"

# 2) 모듈 꼴로 옮겨 담기
mkdir -p "$WORK/apk" "$WORK/base/manifest" "$WORK/base/dex"
unzip -q "$WORK/proto.apk" -d "$WORK/apk"
mv "$WORK/apk/AndroidManifest.xml" "$WORK/base/manifest/"
mv "$WORK/apk/resources.pb" "$WORK/base/"
for dex in "$WORK"/apk/classes*.dex; do mv "$dex" "$WORK/base/dex/"; done
for d in lib assets res; do [ -d "$WORK/apk/$d" ] && mv "$WORK/apk/$d" "$WORK/base/"; done
# APK 서명 흔적(META-INF)은 번들에 넣지 않는다 — 번들 서명은 따로 한다.
rm -rf "$WORK/apk/META-INF"
left=$(cd "$WORK/apk" && find . -type f | head -5)
if [ -n "$left" ]; then echo "make_aab: 옮기지 못한 파일이 있다(모듈 꼴에 자리가 없음):"; echo "$left"; exit 1; fi

# 3) 모듈 zip → 번들. 소리(ogg)·그림(png)은 이미 눌려 있어 다시 누르지 않는다(설치 크기만 는다).
(cd "$WORK/base" && zip -q -r -D "$WORK/base.zip" .)
cat > "$WORK/BundleConfig.json" <<'JSON'
{ "compression": { "uncompressedGlob": ["assets/**/*.ogg", "assets/**/*.png"] } }
JSON
rm -f "$OUT"
java -jar "$BUNDLETOOL" build-bundle --modules="$WORK/base.zip" --config="$WORK/BundleConfig.json" --output="$OUT"
java -jar "$BUNDLETOOL" validate --bundle="$OUT" >/dev/null
echo "make_aab: $OUT ($(du -h "$OUT" | cut -f1))"

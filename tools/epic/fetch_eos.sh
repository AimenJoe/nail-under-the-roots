#!/usr/bin/env bash
# Epic 포털에서 받는 파일(EOS C SDK·BuildPatchTool)을 비공개 에셋 레포의 Release `eos`에서 받아 푼다(0276 E4).
# 파일이 커서(웹 업로드 25MB·git 100MB 한도) 레포 대신 Release 첨부에 둔다 — 첨부는 하나에 2GB까지, zip 이름은 아무거나.
# 쓰기: tools/epic/fetch_eos.sh sdk|bpt <작업 폴더(상대 경로)>   → 마지막 줄에 찾은 경로(작업 폴더 기준 상대 경로)
#   sdk: Include/eos_sdk.h·Lib/EOSSDK-Win64-Shipping.lib·Bin/EOSSDK-Win64-Shipping.dll이 있는 SDK 뿌리
#   bpt: 경로에 Linux가 들어간 BuildPatchTool 실행 파일
# 환경: ASSETS_REPO(owner/name), GH_TOKEN(ASSETS_TOKEN — 비공개 레포 읽기). Windows 러너(Git Bash)에서도 돈다.
set -euo pipefail
what=${1:?sdk|bpt}; out=${2:?작업 폴더}
mkdir -p "$out/zip" "$out/x"
gh release download eos -R "$ASSETS_REPO" -p '*.zip' -D "$out/zip" --clobber >&2
for z in "$out"/zip/*.zip; do
  d="$out/x/$(basename "$z" .zip)"
  # 풀다 몇 파일이 실패해도(Windows에서 macOS 심볼릭 링크 등) 계속 — 필요한 파일이 있는지는 아래에서 본다.
  if command -v unzip >/dev/null; then unzip -q -o "$z" -d "$d" || echo "::warning::$(basename "$z") 일부를 못 풀었다" >&2
  else 7z x -y -bd -o"$d" "$z" >/dev/null || echo "::warning::$(basename "$z") 일부를 못 풀었다" >&2; fi
done
case $what in
  sdk)
    h=$(find "$out/x" -path '*/Include/eos_sdk.h' | head -1)
    root=${h%/Include/eos_sdk.h}
    if [ -z "$h" ] || [ ! -f "$root/Lib/EOSSDK-Win64-Shipping.lib" ] || [ ! -f "$root/Bin/EOSSDK-Win64-Shipping.dll" ]; then
      echo "::error::Release eos에 EOS C SDK(Include·Lib·Bin의 Win64 파일)가 없다 — 포털의 C SDK zip을 붙인다" >&2; exit 1
    fi
    echo "$root" ;;
  bpt)
    b=$(find "$out/x" -path '*Linux*' -name BuildPatchTool -type f | head -1)
    [ -n "$b" ] || { echo "::error::Release eos에 BuildPatchTool(Linux)이 없다 — 포털의 BuildPatchTool zip을 붙인다" >&2; exit 1; }
    chmod +x "$b"; echo "$b" ;;
  *) echo "sdk|bpt" >&2; exit 2 ;;
esac

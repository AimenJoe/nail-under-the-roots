#!/usr/bin/env bash
# 앱인토스 묶음(.ait) 만들기(0300). 운영 웹 빌드 폴더를 받아 토스용으로 손보고 `ait build`로 묶는다.
#   TOSS_APP_NAME=<콘솔의 appName> bash tools/toss/build.sh <운영 웹 빌드 폴더(assemble.py release)>
# 나오는 것: toss/<appName>.ait. 올리기는 store-deploy `toss`가 `ait deploy --api-key`로.
# 손보는 것: 서비스 워커·로딩용 미리 받기 목록은 뺀다(토스가 묶음째 준다), 토스 다리(nail-toss.js)를 nail-host.js 앞에 끼운다.
set -euo pipefail
WEB=$(realpath "$1")
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
T="$ROOT/toss"
[ -n "${TOSS_APP_NAME:-}" ] || { echo "TOSS_APP_NAME이 없다(콘솔에 등록한 appName)"; exit 1; }
cd "$T"
[ -d node_modules ] || npm ci --ignore-scripts --no-audit --no-fund
rm -rf dist && cp -r "$WEB" dist
rm -f dist/sw.js dist/_headers
npm run --silent bridge
python3 - dist/index.html <<'PY'
import sys
p = sys.argv[1]
s = open(p, encoding="utf-8").read()
tag = '<script src="nail-host.js"></script>'
assert tag in s, "index.html에 nail-host.js 자리가 없다"
s = s.replace(tag, '<script src="nail-toss.js"></script>\n  ' + tag, 1)
open(p, "w", encoding="utf-8").write(s)
PY
# 압축 풀어 100MB 이하(미니앱 등록하기 문서).
size=$(du -sb dist | cut -f1)
echo "dist: $((size / 1024 / 1024))MB"
[ "$size" -le $((100 * 1000 * 1000)) ] || { echo "토스 묶음 한도(압축 풀어 100MB)를 넘는다: $size"; exit 1; }
npx ait build
ls -la "$T"/*.ait

#!/usr/bin/env bash
# Android 에뮬레이터에서 APK를 켜 보고 로그를 남긴다(2026-11-07, Knellroot 045ff55·bab436d를 옮김 — nail 0286).
#   bash tools/test/android-smoke.sh <apk> <나갈 폴더> [패키지]
# 패키지를 주지 않으면 APK에서 읽는다(aapt2 — 개발판은 io.github.aimenjoe.dev.nail, 0286). 하는 일: 깔기 → 켜기 → 20초 뒤 화면·살아 있나
# → Enter 둘(타이틀 → 시작) → 15초 뒤 화면·살아 있나 → logcat. 켜고 20초 뒤 죽어 있으면 실패(종료 코드 1).
# Rust 패닉은 logcat의 RustStdoutStderr·"panicked", 라이브러리를 못 찾으면 "UnsatisfiedLinkError",
# 네이티브 크래시는 "Fatal signal"·"backtrace"로 남는다.
set -u
APK=$1
OUT=$2
mkdir -p "$OUT"
if [ -n "${3:-}" ]; then
  PKG=$3
else
  BT=$(ls -d "$ANDROID_HOME"/build-tools/* 2>/dev/null | sort -V | tail -1)
  PKG=$("$BT/aapt2" dump badging "$APK" | sed -n "s/^package: name='\([^']*\)'.*/\1/p")
fi
echo "package: $PKG" | tee "$OUT/package.txt"
unzip -l "$APK" | grep "lib/arm64-v8a/" | tee -a "$OUT/package.txt"
adb install -r -g "$APK" > "$OUT/install.txt" 2>&1
cat "$OUT/install.txt"
adb logcat -c
adb shell monkey -p "$PKG" -c android.intent.category.LAUNCHER 1 > /dev/null 2>&1
alive() { if adb shell pidof "$PKG" > /dev/null 2>&1; then echo alive; else echo DEAD; fi; }
sleep 20
adb exec-out screencap -p > "$OUT/1-title.png"
first=$(alive)
echo "after launch: $first" | tee "$OUT/alive.txt"
for _ in 1 2; do adb shell input keyevent 66; sleep 2; done
sleep 15
adb exec-out screencap -p > "$OUT/2-start.png"
echo "after start: $(alive)" | tee -a "$OUT/alive.txt"
adb logcat -d > "$OUT/logcat.txt"
grep -inE "panicked|RustStdoutStderr|UnsatisfiedLinkError|Fatal signal|backtrace|SIGSEGV|SIGABRT|AndroidRuntime|nail" "$OUT/logcat.txt" | head -200 > "$OUT/crash.txt" || true
echo "----- crash lines -----"
cat "$OUT/crash.txt"
[ "$first" = alive ] || { echo "::error::$PKG가 켜고 20초 안에 죽었다(위 crash lines·아티팩트 logcat)"; exit 1; }

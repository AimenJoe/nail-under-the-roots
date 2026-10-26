# 0087. Android 앱 아이콘

- 날짜: 2026-10-26
- 상태: 채택(배포 전). 사장: "안드로이드용 아이콘도 없으니 그것도 넣어 줘."

## 지금까지
- APK에 아이콘이 없었다. `Cargo.toml [package.metadata.android]`에 `res`·`icon`이 없어서, 런처가 기본 아이콘을 띄웠다.

## 정한 것
- cargo-quad-apk 설정(README, 고정한 커밋 `d411c8f`): `res = "android/res"`(APK에 넣을 리소스 폴더), `icon = "@mipmap/ic_launcher"`.
- 그림은 iOS(0086)와 같다: Naïl 초상화를 정수배로 키워 어두운 흙빛 + 아래쪽 따뜻한 빛 위에 올린다. 스크립트 하나로 둘을 만든다(`tools/ios/` → `tools/icons/make_icons.py`).
- 옛 런처용 `ic_launcher.png`: mdpi 48 · hdpi 72 · xhdpi 96 · xxhdpi 144 · xxxhdpi 192.
- **적응형 아이콘**(Android 8+, 우리 minSdk 26): `mipmap-anydpi-v26/ic_launcher.xml` = 배경(`ic_launcher_background`) + 앞 그림(`ic_launcher_foreground`, 투명).
  - 칸은 108dp(108~432px)이고, 런처가 원·둥근 네모 등으로 깎아도 늘 보이는 가운데 66dp 안에 얼굴을 둔다.
  - 옛 네모 아이콘만 주면 Android 8+ 런처가 작게 줄여 흰 테두리 안에 넣는 일이 있다(2차, 일반 안내).
- 원형 마스크로 깎아 본 미리 보기에서 얼굴이 잘리지 않았다.

## 확인
- 로컬에는 Android SDK가 없다(0005). APK에 리소스가 들어가는지는 build-android CI(작업 브랜치 push)로 본다. 실제 런처 모양은 기기에서 본다.

# 0074. Android: MainActivity에 android:exported

- 날짜: 2026-10-26
- 상태: 채택

## 배경
사장이 v0.1.4 APK를 폰에 설치하려 하자 "App not installed as package appears to be invalid"가 떴다.
- 내려받은 APK는 깨지지 않았다. 릴리스 파일의 SHA-256이 빌드 로그와 같았고, zip 검사도 통과했다.
- 서명(v1·v2·v3), minSdk 26, targetSdk 36, versionCode 5, arm64는 모두 정상이었다.
- 원인은 매니페스트였다. `MainActivity`에 `intent-filter`(MAIN/LAUNCHER)가 있는데 **`android:exported`가 없었다.**

Android 문서(behavior-changes-12, "Safer component exporting")의 규칙이다. targetSdk 31 이상 앱에서 intent-filter를 쓰는 액티비티·서비스·리시버가 `android:exported`를 명시하지 않으면, **Android 12 이상 기기에 설치할 수 없다.** 설치 화면은 이 파싱 실패(INSTALL_PARSE_FAILED_MANIFEST_MALFORMED)를 "패키지가 잘못됨"으로만 보여 준다.

cargo-quad-apk(커밋 d411c8f)는 `MainActivity`에 exported를 넣지 않는다(서비스에만 넣는다). 우리 targetSdk는 처음부터 36이었다. 그래서 **v0.1.0~v0.1.4 모두** Android 12+에서 설치되지 않았을 것이다. CI의 서명·badging·zipalign 검사로는 이 문제가 안 잡혔고, 실기 설치는 한 번도 못 했다.

## 결정
- `Cargo.toml`의 `[package.metadata.android.activity_attributes]`에 `"android:exported" = "true"`를 넣는다. cargo-quad-apk가 이 표의 키·값을 그대로 `<activity>` 속성으로 옮긴다(`build_attribute_string`). 런처에서 여는 주 화면이라 true다.
- `build-android.yml` 확인 단계에서 `aapt2 dump xmltree`로 매니페스트를 보고, exported가 없으면 빌드를 실패시킨다.
- v0.1.5로 다시 낸다. 그림·게임 내용은 v0.1.4와 같다.

## 남은 것
- 실기 설치 확인은 사장 몫이다(사람이 할 일). 설치가 되면 KB 가이드의 "확인 못 함: 실기 설치"를 지운다.

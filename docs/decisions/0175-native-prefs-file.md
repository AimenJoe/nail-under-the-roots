# 0175. 설정(언어·음량·조작)을 데스크톱·Android·iOS에서도 파일로 저장한다

- 날짜: 2026-10-27
- 상태: 채택. 사장: "앱을 업데이트할 때, 언어 등 환경설정 세팅이 없어지는 것 같은데?"

## 왜
- `prefs.rs`는 웹에서만 저장했다(localStorage 정수 칸, 0022). 데스크톱·Android·iOS의 `store`는 비어 있는 함수라 **켤 때마다 기본값**이었다 — 업데이트 때문이 아니라 처음부터 한 번도 남은 적이 없었다. 사장은 업데이트하고 다시 켤 때 알아챘을 뿐이다.
- 기록(profile)은 이미 기기 폴더에 파일로 둔다(0041: Android `/data/data/<패키지>/files`, iOS `Library/Application Support/nail`, 데스크톱 XDG·APPDATA). 설정도 같은 폴더면 된다. 앱을 덮어 설치해도 그 폴더는 남는다(패키지 이름이 같으면).

## 결정
- `profile::data_dir()`(웹 아님)를 내고, `prefs::store`(웹 아님)는 그 폴더의 **`prefs.txt`** 한 줄에 칸 스무 개를 빈칸으로 띄어 적는다(`read_from`·`write_to`, 임시 파일에 쓰고 이름 바꾸기 — 기록 파일과 같은 방식). 칸 번호·값 약속은 웹과 같다(`encode`·`decode` 하나, `MAGIC` 그대로). 없거나 깨지면 0 = 저장 전 = 기본값.
- 웹은 그대로 localStorage. `load`·`save`는 `store::load_all`·`save_all` 하나로 두 플랫폼을 가른다.
- 설정은 슬롯 밖(0078)이라 슬롯·런 저장과 섞이지 않는다. 지우기 화면에서도 지우지 않는다.

## 확인
- `cargo test` `native_prefs_file_round_trips`(쓴 설정이 그대로 읽힘, 없는 파일은 0, 표식이 깨지면 통째로 무시). 기록 파일 테스트처럼 임시 폴더를 쓴다.
- 사람이 볼 것(사장): Android에서 언어·음량·버튼을 바꾸고 껐다 켜면 남는지, APK를 덮어 설치해도 남는지(패키지 이름이 같으니 남아야 한다). 안 남으면 `/data/data/io.github.aimenjoe.nail/files/prefs.txt`가 생기는지부터.

# 0281 iPhone·iPad: touchstart를 막지 않는다 — 홈 화면 웹앱에서 실제 키보드가 안 듣던 것

- 상태: 채택, **실기 확인 전**(사장 2026-11-05 "일단 해봐야 먹히는지 알 수 있을 듯")
- 관련: 0278(웹 키 입력 — 포커스·빈 `code`), 0269(PWA·홈 화면), 0026(번들 복사본은 고치지 않고 셸에서 끼운다)

## 왜
사장(2026-11-05): 키보드가 안 골라지는 것은 **iPad에서 홈 화면에 추가한 웹앱으로 실행했을 때**였다. 0278에서 고친 두 경로(포커스가 캔버스 밖·빈 `code`)는 키 이벤트가 JS까지는 오는 경우다. 이번에는 키가 웹뷰에 오지 않는 것으로 본다(2차 자료, 원문 사이트는 세션에서 안 열려 검색 요약만):
- WebKit 버그 206887(2020): iPad 실제 키보드 + WKWebView에서 `touchstart`에 `preventDefault()`를 부르면 `keydown`이 아예 안 온다. Safari에서는 된다. 빼면 돌아온다.
- WebKit 버그 276640(2024-07 고침)·280651: iPadOS는 웹 내용 뷰를 글 입력칸처럼 다뤄 **탭할 때까지 키보드 포커스를 미룬다**. cordova-ios #1132(iPadOS 15~16)도 "실행 직후 키 입력이 안 되다 화면을 한 번 탭하면 된다" → 웹뷰를 시작 때 first responder로 만들어 고쳤다.
- 홈 화면 웹앱은 Safari 탭이 아니라 웹뷰로 뜨고, 번들(`mq_js_bundle.js`)은 캔버스 `touchstart`마다 `preventDefault()`라 그 탭이 탭으로 치이지 않는다 — 앞의 둘이 겹친 것으로 본다. 설정에 들어간 뒤 풀린 까닭은 아직 모른다.

## 정한 것
- `web/nail-host.js`: **iOS(iPhone·iPad, 터치 맥 포함)에서만** 창 capture `touchstart`에서 그 이벤트의 `preventDefault`를 아무것도 안 하는 함수로 덮는다(번들 처리기보다 먼저 돈다). 번들은 그대로.
- 스크롤·확대는 CSS `touch-action: none`·`user-scalable=no`, 길게 누르기는 `-webkit-touch-callout: none`·`user-select: none`이 그대로 막는다. 흉내 클릭·마우스는 번들의 `touchend` `preventDefault`가 그대로 막는다(덮지 않는다).
- Safari 탭·앱 안 브라우저(카카오톡 등 WKWebView)도 iOS라 같이 바뀐다 — 앱 안 브라우저도 같은 버그를 탈 것이라 굳이 가르지 않았다.
- 연기 시험 `ipad:`(Playwright iPad Pro 11 흉내): touchstart는 안 막히고 touchend는 막히고 터치는 게임에 닿는다. 고치기 전 파일로는 실패한다. Chromium이라 키가 오는지는 못 본다.

## 안 되면
iPad 웹앱에서만 첫 화면 위에 HTML "탭해서 시작" 칸을 한 번 띄운다(HTML 요소 탭은 막히지 않아 포커스가 잡힌다). 그래도 안 되면 키가 첫 탭 전에는 안 오는 것은 iPadOS 쪽 한계로 적는다.

## 덧붙여 확인한 것(키 자리)
키는 물리 자리로 읽는다: 웹 `e.code`, Windows 스캔코드, macOS `NSEvent.keyCode` — 배열이 달라도(AZERTY 등) WASD 자리가 그대로다. **Linux X11만** 배열을 거친 글자(keysym, `XGetKeyboardMapping`)라 AZERTY면 자리가 달라진다. 화면의 키 이름은 QWERTY 글자로 고정이다(AZERTY에서도 "WASD"). 둘 다 이번에는 고치지 않았다 — 이름은 웹 `navigator.keyboard.getLayoutMap()`(Chrome 계열만)·PC OS API가 필요하다.

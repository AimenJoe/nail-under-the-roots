// 앱인토스 묶음 설정(0300). appName은 콘솔에 등록한 이름과 같아야 올라간다 — 레포에 박지 않고
// CI의 Environment `store` 변수 TOSS_APP_NAME에서 읽는다(tools/toss/build.sh).
import { defineConfig } from "@apps-in-toss/web-framework/config";

const appName = process.env.TOSS_APP_NAME ?? "";

export default defineConfig({
  appName,
  permissions: [],
  webView: {
    // 게임 화면이 튕기거나 당겨서 새로 고침이 되지 않게.
    bounces: false,
    pullToRefreshEnabled: false,
    overScrollMode: "never",
    // 소리는 첫 입력에서 켠다(번들의 잠금 풀기와 같은 길).
    mediaPlaybackRequiresUserAction: false,
    allowsBackForwardNavigationGestures: false,
  },
  webBundleDir: "dist",
});

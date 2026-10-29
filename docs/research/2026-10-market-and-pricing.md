# Naïl: Under the Roots — 시장 조사 (v0.6 계획용)

- 작성: 2026-10-29, Claude(조사 세션). 법률·세무 자문이 아니다.
- **조사 한계**: 이 세션의 네트워크는 모든 외부 페이지 열기를 막았다(partner.steamgames.com, policies.google.com, itch.io, developer.apple.com, support.google.com, gamedeveloper.com, howtomarketagame.com 모두 `connect_rejected`). 그래서 **모든 웹 근거는 검색 엔진 요약으로만 읽었다**. 아래에서 "1차"는 *출처가 공식 페이지·원 데이터*라는 뜻이고, 원문 전체를 직접 읽었다는 뜻은 아니다. 숫자·문구는 결정 전에 원문으로 다시 확인할 것.
- 표기: **[1차]** 공식 문서·정부·원 데이터(검색 요약으로 읽음) · **[2차]** 블로그·기사·포럼·벤더 · **[추정]** 이 보고서의 계산·판단 · **[레포]** nail-under-the-roots·KB 기록 · **[기억]** 모델 지식(2026-06 이전), 재확인 필요.

---

## 한 줄 결론

**돈은 "가능하지만 작다"가 현실적이다.** 지금 그대로 Steam에 $9.99로 내면 마케팅 없이 첫해 수백만 원, 위시리스트를 7천 개쯤 모으면 1~2천만 원대, 그 이상은 운이 섞인 히트다. 그런데 이 게임에는 **팔기 전에 정리해야 할 것이 셋**이 있다. ① 눈에 보이는 그림 거의 전부가 생성형 AI(Gemini)라 Steam에 공개 표시가 붙는데, 이 표시가 판매·평가·행사 참가에서 손해를 본다는 근거가 쌓이고 있다. ② 지금은 게임 전체가 웹에서 무료다. ③ 레포가 공개다. 그래서 v0.6의 "오래 걸리는 큰 것"으로 가장 근거가 많은 후보는 새 콘텐츠가 아니라 **"팔 수 있는 판 만들기"**다. 사람이 그린 얼굴 그림(캡슐·키 아트·주인공)을 넣고, 데스크톱 빌드·Steam 페이지·데모를 갖추는 것이다. 콘텐츠를 더한다면 **다회차 깊이**가 가격대를 한 칸 올리는 근거가 된다.

---

## 1. 수익 현실

### 1.1 분포: 중앙값과 히트는 다른 세계다
| 지표 | 값 | 출처 등급 |
|---|---|---|
| 2025년 Steam 출시작 중앙값 매출(2025-10 시점, 출시 1년이 안 된 묶음) | 총매출 **약 $249**. 셋 중 둘은 $1,000 미만, 열 중 넷은 Steam Direct 수수료 $100도 못 넘김, $100k 넘긴 비율 8% | [2차] Gamalytic 수치를 옮긴 Substack 글 |
| 2019년 출시 묶음의 평생 총매출 중앙값 | **$1,136**, $10k가 68번째 백분위 | [2차] Zukowski가 VG Insights 데이터로 분석한 것을 옮긴 글 |
| 2025년 $1M 넘긴 게임 | 2만 개 넘는 신작 중 **약 300개** | [2차] |
| 인디 몫 | 2025년 Steam 전체 매출의 약 25%(약 $4.4B) | [2차] Alinea Analytics 인용 |
| 오래된 Gamalytic 수치 | Steam 게임 절반 이상이 평생 $1,000 미만 | [2차] Gamalytic 인포그래픽(2023) |

**생존자 편향 주의**: 사람들이 비교하는 Hades, Brotato, Halls of Torment 같은 게임은 상위 0.1~1%다. 회고 글·인터뷰도 잘된 게임이 훨씬 많이 쓴다. 반대로 중앙값에는 며칠 만에 만든 게임, 에셋 뒤집기, 페이지만 있는 게임이 많이 섞여 있다. 그래서 **이 게임(열한 언어, 보스 다섯, 캐릭터 셋, 패드 지원)에 중앙값은 너무 비관적인 기준선**이다. 하지만 "마케팅 없음"이라는 점에서는 중앙값 게임과 같다 [추정]. 결과를 가장 잘 예측하는 것은 품질이 아니라 **출시 때의 위시리스트 수**다(아래).

### 1.2 리뷰 수로 판매량 추정하기 (Boxleiter 방식)
- 리뷰 1개 ≈ 판매 **약 30장**(최근 출시작, 흔한 범위 20~60). 2018년에는 82배, 2020년 58배, 지금은 약 30배로 내려왔다는 정리가 있다. 장르별로는 액션이 높고 인디·서사형이 낮다(20~40배). 로그라이트는 30~50배라는 표가 있다 [2차, 계산기를 파는 벤더 블로그. 원조 관찰은 Simon Carless, 50배 보정은 Jake Birkett로 알려짐].
- 쓰는 법: 비슷한 게임의 리뷰 수 × 30 × 실제 평균 판매가(정가의 50~70%) × 0.7(Steam 몫을 뺌) ≈ 개발자 몫 [추정].
- 오차가 크다. 최근 2년 안에 나온 같은 장르·같은 가격대 게임 10~20개로 범위를 보는 데만 쓴다.

### 1.3 위시리스트 → 판매 전환
| 지표 | 값 | 등급 |
|---|---|---|
| 첫 주 판매 / 출시 때 위시리스트 (예전 기준) | 중앙값 **15~25%**(위시리스트가 많을수록 높음) | [2차] Zukowski·Carless 정리 |
| Gamalytic(2024 무렵) | 첫 주 중앙값 22%, 첫 달 27%. 앞서 해보기(EA) 출시 약 20%, 정식 출시 약 30%(첫 달 500장 미만은 뺌) | [1차 데이터 회사 블로그] gamalytic.com |
| GameDiscoverCo(2024~25) | 위시리스트 2.5만 넘는 게임의 첫 주 중앙값 **0.15배**, **$10 넘는 게임은 0.10배** | [2차] GameDiscoverCo 요약 |
| Zukowski(2023~25 출시작, 개발자가 낸 Steamworks 자료 160개) | 위시리스트 하나당 첫 달 판매가 2020~22 출시작의 **절반 정도** | [2차] 요약 기사 |
| 인기 예정작(Popular Upcoming) | 약 7,000개라는 경험칙이 있지만 Valve는 "고정 문턱 없음"이라고 한다. 2026-06에 문턱이 올랐다는 보도도 있다 | [2차] |
| 반례: Halls of Torment | EA 동안 위시리스트→구매 50~70%(정가 $4.99) | [2차] 개발사 인터뷰 기사 |

계획할 때는 **첫 주 ≈ 출시 위시리스트의 10~15%, 첫해 ≈ 첫 주의 2.5~4배**로 잡는 것이 지금 자료에 맞다 [추정. 첫해 배수는 자료를 찾지 못한 경험칙].

### 1.4 Next Fest 효과
- 한 번 참가에서 얻는 위시리스트 중앙값은 **약 300~1,100개**로 자료마다 다르다. 2025-06 GGInsights 2,582개 → 336개, 2025-02 Zukowski 설문 → 1,079개, 2026-02 HTMAG 설문 182개 → 806개(70번째 백분위 1,839개), GameDiscoverCo 추정 → 약 200개 [2차, 대부분 자가 응답 설문].
- **행사 전 위시리스트가 결과를 가장 잘 예측한다**(스피어만 0.825). 데모를 몇 달 먼저 낸 게임은 행사 주에 낸 게임보다 위시리스트 중앙값이 약 2.5배였다 [2차]. KB `public-demo-and-outside-feedback`의 "데모는 행사 전에"와 맞는다.
- 다가오는 일정: **2027-02-22~03-01**(등록 마감 **2027-01-10**, 모든 항목 마감 02-08) [1차 Steamworks 페이지 검색 요약], **2027-06-14~21** [2차]. 참가 조건은 공개된 Coming Soon 페이지와 행사 때 공개되는 데모 [1차]. "게임마다 한 번만"이라는 말은 3자 글에만 있다 [2차].

### 1.5 이 게임의 시나리오 [추정]
가정: 정가 $9.99. 할인·지역 가격·환불을 반영하면 장당 약 $6.5 → Steam 30%를 빼면 장당 약 **$4.5(≈6,300원, $1=1,400원 가정)**. 세금은 따로.

| 시나리오 | 출시 때 위시리스트 | 첫 주 | 첫해 | 개발자 몫(첫해) |
|---|---|---|---|---|
| 지금처럼(마케팅 없음) | ~500~1,500 | 50~200장 | 200~700장 | **약 $1~3k(130만~450만 원)** |
| Steam 페이지 6~12개월 + 데모 + Next Fest 한 번 | ~5,000~8,000 | 600~1,200장 | 2,000~4,000장 | **약 $9~18k(1,300만~2,500만 원)** |
| 잘됨(스트리머·큐레이터가 잡아 줌) | ~20,000~40,000 | 3,000~6,000장 | 10,000~20,000장 | **약 $45~90k(6천만~1.3억 원)** |
| 히트(Halls of Torment·Tiny Rogues급) | 10만+ | — | 50만+ | 예측 대상이 아니다 |

AI 그림 공개 표시의 영향(§4.1)이 이 표에 들어가 있지 않다. 표시를 그대로 두면 아래 두 줄의 확률이 떨어진다고 보는 것이 자료에 맞다.

---

## 2. 가격

### 2.1 비교 게임
가격은 미국 Steam 정가. 팀 크기와 분량은 [기억/2차]이므로 재확인 필요.

| 게임 | 출시 가격 | 팀 | 분량·형태 | 비고 |
|---|---|---|---|---|
| Vampire Survivors | EA $2.99 → 1.0 $4.99 | 시작은 1인 | 서바이버형, 수십~수백 시간 | 모바일은 무료(결제 없음) [2차] |
| Brotato | $4.99 | 1인(Blobfish) | 서바이버형 | 모바일은 출시 때 $4.99 유료, 지금은 무료판과 유료판이 함께 있음, 수백만 장 [2차] |
| Halls of Torment | EA $4.99 → 1.0 $6.66 | 5인 | 서바이버형 | 1.0 때 90.5만 장(환불 3%), 2024-10에 100만 장 [2차] |
| Tiny Rogues | EA $9.99(2022-09~) | 1인 또는 소수(RubyDev, 확인 못 함) | 액션 로그라이트, 확장마다 값을 올린다고 공지 | SteamSpy 50만~100만 소유자 [2차] |
| Enter the Gungeon | $14.99 (2016) | 약 4~5인(Dodge Roll) | 탑다운 슈팅 로그라이트, 매우 깊음 | [기억] |
| Wizard of Legend | $15.99 (2018) | 2인 | 탑다운 액션 로그라이트 | [기억] |
| Death's Door | $19.99 (2021) | 2인(Acid Nerve) | 손으로 만든 탑다운 액션(로그라이트 아님) 8~12시간 | 첫 주 10만 명 [2차] |
| Ember Knights | 1.0 $19.99 (2023) | 소수(Doom Turtle) | **4인 협동** 액션 로그라이트 | [2차] |
| Moonlighter | $19.99 (2018) | 약 10인(Digital Sun) | 가게 + 던전 | [기억] |
| Hades | EA $19.99 → 1.0 $24.99 | 약 20인(Supergiant) | 무기 6×형태, 지대 4, 엔딩까지 20시간 넘음 | [기억] |
| Dead Cells | EA → 1.0 $24.99 (2018) | 약 10인(Motion Twin) | 빌드 깊이, 지대 다수 | 모바일 $8.99 [기억] |
| Cult of the Lamb | $24.99 (2022) | 소규모(Massive Monster) + Devolver | 로그라이트 + 마을 경영 | 첫 주 100만 명 [2차] |
| Rogue Legacy 2 | EA → 1.0 $24.99 (2022) | 소규모(Cellar Door) | 계승형 로그라이트 | [기억] |

### 2.2 판단
- Steam 가격 분포를 보면 2020년 출시작의 53%가 $5 이하였다 [2차]. 로그라이트 출시가는 대체로 $10~20 사이에 모인다 [2차, 근거가 약함].
- **$5 이하에서 크게 팔린 게임은 모두 "서바이버형"**이다. 이 장르는 조작이 단순하고 반복이 길다. 이 게임(손맛 근접 + 패턴 보스)은 Death's Door·Gungeon 쪽 문법이다. 그 문법에서 2인 팀 게임이 $15~20을 받은 것은 분량과 다듬기가 깊어서다 [추정].
- GameDiscoverCo는 "$10을 넘는 게임은 위시리스트 전환이 더 낮다", "새 게임은 낮은 가격·낮은 복잡도가 힘을 얻는다", "'로그라이트처럼 값을 매겼다가 덜 팔렸다'는 사례"를 든다 [2차].
- **추천 가격대: $7.99~$9.99**(한국 원화는 Steam 권장 지역 가격표를 따른다). 데모가 있으니 출시 할인은 10~20%로 [추정].
  - 지금 분량: 캐릭터 3, 지대 4, 보스 5 + 가장 깊은 곳, 런 10~15분. 엔딩까지 걸리는 시간은 아직 재지 않았다. **v0.6 전에 처음 하는 사람이 엔딩까지 걸리는 시간과 해금을 다 여는 시간을 재야 한다.** 엔딩까지 4~6시간이면 $9.99가 맞다 [추정].
- **$12.99~14.99로 올리려면** [추정, 비교 게임에 비춘 판단]
  1. 다회차 깊이: 난이도 사다리(Hades의 열기, Dead Cells의 보스 세포 같은 것) + 저널·도감을 다 채우는 데 20시간 이상.
  2. 캐릭터 4~5. 각자 회피와 특기가 다르다.
  3. 지대 5~6, 지대마다 보스 갈래가 둘.
  4. 사람이 그린 얼굴 그림. AI 표시가 없거나 줄어든 상태.
  5. Windows 데스크톱 빌드 + Steam Deck 확인 + 업적·클라우드 저장.
- **$19.99 이상**은 Hades·Dead Cells급 빌드 깊이나 협동(Ember Knights)이 필요하다. 1인 개발의 v0.6 한 판으로는 닿지 않는다 [추정].
- Tiny Rogues처럼 **EA를 싸게 시작하고 확장마다 값을 올리는 방식**은 가격을 올릴 수 있는 길을 처음부터 열어 둔다. 단 EA의 함정은 §3을 보라.

---

## 3. 비즈니스 모델 비교

| 모델 | 비용·수수료(정책 날짜) | 이 게임에 맞나 | 근거·메모 |
|---|---|---|---|
| **Steam 유료(1.0 바로)** | Steam Direct **$100**(조정 총매출 $1,000을 넘으면 돌려줌). 수수료 30%, $10M 넘으면 25%, $50M 넘으면 20%(구간별) [2차, 2018 정책]. Coming Soon 페이지는 **출시 2주 전부터** 공개해야 함 [1차 Steamworks]. 수수료 결제 뒤 출시까지 30일을 기다린다는 말은 [2차] | **높음**(가장 큰 PC 시장, 패드 지원·Deck 맞음) | Windows/Linux 네이티브 빌드가 필요하다(지금은 웹·Android뿐) [레포]. 부가세는 Valve가 나라별로 걷는다고 알려짐 [기억] |
| **Steam 앞서 해보기(EA)** | 위와 같음 | **중간** | 2026년에 EA를 졸업한 91개 중 **21%만** 1.0 첫 30일 매출이 EA 첫 30일보다 많았다(2025년 20%). 2025년 졸업작은 1.0 달 매출 중앙값이 EA 첫 달의 40%. EA 환불률 중앙값 12.4%(정식 8%) [2차, GameDiscoverCo]. → "1.0 때 한 번 더 뜬다"는 기대는 틀리기 쉽다. 이미 엔딩이 있는 게임이라 EA를 할 이유가 약하다 [추정] |
| **Steam 데모 + Next Fest** | 무료 | **높음** | §1.4. 데모는 행사 몇 달 전에 |
| **웹 무료(Netlify, 지금)** | 0 | 지금은 전체 판이다 | 유료판과 어떤 관계로 둘지 정해야 한다(§4.3) |
| **itch.io**(HTML5 무료, 원하는 만큼 내기, 유료) | 수수료를 개발자가 고른다(기본 10%로 알려짐), 결제 Stripe·PayPal, 미국 세금 양식 [1차 FAQ + 2차] | **보조** | 찾는 사람이 적다. 웹 데모를 걸고 Steam 페이지로 이어 주는 길로 쓴다. 0223 A5와 같다 |
| **Google Play 유료** | 가입비 **$25 한 번**[기억]. 수수료 15%(매해 첫 $1M, 2021~) [1차] → Epic 합의로 **기본 10%**로 바뀌는 중, 한국은 **2026-12-31까지** 적용 예정(법원 최종 승인 전) [2차]. **targetSdk 36**(새 앱·업데이트, 2026-08-31~) [1차] — 레포는 이미 36 [레포]. 2023-11-13 뒤 만든 개인 계정은 **테스터 12명 × 14일 비공개 테스트**를 거쳐야 정식 출시 [1차] | **낮음~중간** | 게임이 컨트롤러를 전제로 한다(터치는 보조) [레포]. 폰 사용자 다수와 맞지 않는다. 휴대 게임기 사용자는 소수다 |
| **모바일 무료 + 한 번 결제로 전체 열기** | 결제 연동(Play Billing·StoreKit)을 macroquad에서 JNI 등으로 직접 붙여야 함 [추정, 비용 큼] | 중간(구조상 맞음) | Loop Hero $6.99 열기, Sparklite "첫 보스까지 무료" [2차]. 이 게임의 해금 구조(첫 길 → 나머지)와 잘 맞는다. Balatro는 $9.99 유료로 첫 주 $55만 [2차]지만 예외적 히트다 |
| **광고·F2P** | 광고 SDK, 개인정보 동의 화면 | **낮음** | 0223의 "몰래 재는 통계 없음" 원칙과 부딪친다. 손맛 액션과 광고 끊김이 안 맞는다 [추정] |
| **App Store** | **$99/년**(개인 면제 없음) [1차 Apple 한국어 페이지], Small Business Program 15%(매해 신청) [1차/2차] | **낮음(지금)** | iOS에서는 패드가 안 된다(gilrs에 iOS 백엔드 없음) [레포] |
| **콘솔(Switch 등)** | 개발 키트 승인, macroquad 이식 경로 없음 [추정] | 나중 | 퍼블리셔·이식 협력사와 함께. 1.0 뒤 |
| **Kickstarter** | 5% + 결제 수수료 약 3~5%, 다 모아야만 받음 [2차] | **낮음** | 이미 볼 사람이 있어야 된다. 지금은 없다 |
| **Patreon / Ko-fi** | Patreon: 2025-08-04 뒤 새 창작자 **10% + 결제 수수료** [2차]. Ko-fi: 후원 0~5% + 결제 수수료(기본 설정이 출처마다 다름) [2차] | 보조(용돈 수준) | "공개 개발" 이야기와 맞는다. 후원을 받으면 등급분류 "비영리" 면제(§4.5)가 흔들릴 수 있다 |
| **퍼블리셔** | 보통 매출의 30~50%(투자금 회수 뒤) [기억] | 중간 | 마케팅·콘솔 이식을 맡는다. AI 그림을 꺼리는 곳이 많을 것이다 [추정] |
| **번들(Humble·itch)** | 큰 할인 | 긴 꼬리 | 출시 1~2년 뒤 |
| **STOVE(스토브인디)** | 수수료는 찾지 못함(계약서 양식에서 수치가 가려져 있음) | 보조(국내) | 자체등급분류 권한이 있다는 2차 보도가 있다. 국내 이용자 |
| **Epic Games Store** | 12%[기억] | 보조 | Epic은 AI 표시를 요구하지 않는다(Sweeney가 Steam 표시를 비판함) [2차] |

### 3.1 한국 개인(또는 개인사업자)의 세금·정산 — 모두 [2차], 세무사 확인 필수
- **사업자등록**: 계속·반복해서 팔면 사업소득이다. 해외 매출 영세율을 받으려면 **일반과세자**가 낫다(간이과세자는 영세율이 안 된다). 매출은 플랫폼 수수료를 빼기 전 총액으로 잡고, 수수료는 경비로 넣는 것이 일반 설명이다. 국가별 실적은 Steamworks 보고서에서.
- **미국 원천징수**: Valve에 **W-8BEN**(개인)을 내고 한미 조세조약을 적용받는다. 양식이 없으면 30%. 조약 세율은 소득을 어떻게 분류하느냐에 따라 10%/15% 설명이 섞여 있고 확인하지 못했다. 미국에서 뗀 세금은 국내에서 외국납부세액공제.
- 종합소득세는 다음 해 5월. Steam 정산은 USD로 받으니 환율 기록을 남긴다.
- 게임산업법상 **게임제작업 등록**(시·군·구청)이 필요하다는 보도가 있다. 등급분류 신청 때 등록증을 낸다는 안내도 있다(§4.5).

---

## 4. 이 게임의 위험·막는 것

### 4.1 Steam AI 공개 표시 — **가장 큰 위험**
- **정책** [1차 Steamworks Content Survey, 검색 요약. 2024-01 틀, **2026-01-16~17 개정**]
  - "Pre-Generated": 개발 중에 AI 도구로 만들어 게임과 함께 나가고 플레이어가 보는 모든 콘텐츠.
  - "Live-Generated": 실행 중에 만드는 콘텐츠. 불법 콘텐츠를 막는 장치를 설명해야 한다.
  - 2026-01 개정으로 코드 도우미 같은 개발 도구는 공개 대상에서 빠졌다 [2차 보도 여럿].
  - 상점 페이지에 "AI Generated Content Disclosure" 칸이 생기고, 플레이어는 오버레이로 위반을 신고할 수 있다 [2차].
  - → **이 게임은 스프라이트·초상화·일러스트·타이틀이 모두 Pre-Generated에 해당해 반드시 공개해야 한다.** 코드로 도트에 옮겼다고 빠지지 않는다 [추정, 정의에 비춤]. Claude가 쓴 코드는 개정 뒤 기준으로는 공개 대상이 아니다 [2차]. 마케팅 그림이 예외인지는 확인하지 못했다(법률 블로그 한 곳의 주장뿐).
- **플레이어 반응** [2차]
  - GameDiscoverCo 설문(2026-06-25~07-02, Steam을 많이 쓰는 응답자): 약 31% 부정, **8%는 "AI를 쓴 게임은 절대 안 산다"**. 개발자가 AI 사용을 다 밝힌다고 믿는 사람은 17%뿐.
  - Quantic Foundry(2025-12, 약 1,800명): **85%가 중립보다 부정적**.
  - Sulka Haro 전수 조사(2023 중반~2026 중반, 출시작 53,597개, 2026-07 보도): AI 표시 비율이 2024년 10.9% → 2025년 19.9% → 2026년 30.8%. **AI 표시 게임은 신작의 3분의 1인데 추정 매출에서는 10~27%**. 원인을 AI 하나로 가르지는 못했다(전반적으로 품질이 낮을 수 있다).
  - Game Oracle(2025년 1~10월 출시 약 1만 개): 첫 달 리뷰 수가 AI 표시 게임 평균 4개, 아닌 게임 7개.
- **행사·상** [2차]: Indie Game Awards는 2025-12에 Clair Obscur의 상을 거뒀다. 임시 텍스처에 생성형 AI를 쓴 것만으로도 규칙 위반이었다("개발 어느 단계에서든 생성형 AI를 쓰면 자격 없음"). 인디 상·쇼케이스는 같은 규칙을 둔 곳이 있다고 봐야 한다.
- **이 게임에 더 아픈 이유** [추정]: "귀여운 도트 게임"을 사는 사람은 손으로 찍은 점을 가치로 보는 경우가 많다. 그리고 표시는 상점 페이지 첫 화면 근처에 뜬다.
- **줄이는 길**(효과가 큰 순) [추정]
  1. **얼굴부터 사람 손으로**: Steam 캡슐(여러 크기)·키 아트. 외주 시세는 맞춤 캡슐 한 벌 **$300~700** [2차]. 계약은 KB `guides/commissioning-game-art.md`(2차적저작물작성권 특약).
  2. 주인공 Naïl 스프라이트·초상화를 `tools/pixel`(.pxl)로 사장이 직접 다시 찍는다. 많이 보이는 것부터.
  3. 나머지는 솔직하고 구체적으로 공개한다("캐릭터 원본 그림은 Google Gemini로 만들고, 사람이 도트로 옮기고 다듬었다"). 숨기다가 들키면 신고·리뷰 공격으로 더 크게 잃는다.
  4. 전부 바꾸면 공개 표시를 뗄 수 있다. 이것이 v0.6 "큰 것"의 후보다(§5).

### 4.2 Gemini(나노바나나) 결과물을 팔아도 되나
- [1차, 검색 요약] **Gemini API Additional Terms**(보관본 2024-12-20·2025-05-20·2025-09-25. 최신본은 **2026-03-23 시행**이 있으나 내용을 확인하지 못함):
  - "Google won't claim ownership over that content." 같거나 비슷한 결과를 남에게도 만들 수 있다(독점 아님). 결과물을 어떻게 쓰는지는 사용자 책임.
  - API·AI Studio는 "professional or business purposes"용이고 18세 이상만 쓸 수 있다.
  - **무료(결제 계정 없음)로 쓰면 입력과 결과를 Google이 제품 개선에 쓴다.** 결제 계정을 연결하면 유료 서비스로 취급된다.
  - 금지 사용 정책(Prohibited Use Policy)을 따로 지켜야 한다.
- [2차] Google 서비스 약관(2024-05 개정)도 생성물의 소유권을 주장하지 않는다(소비자용 Gemini 앱).
- **결론**: 계약상 상업 판매는 막혀 있지 않다(Google이 권리를 주장하지 않음). 할 일은 둘이다. **어느 제품(API·AI Studio·Gemini 앱)을 어느 날 썼는지 기록**하고, 그때의 약관 사본을 남긴다. `assets/LICENSES.md`에 "확인 필요"로 남아 있는 칸을 이걸로 닫는다 [레포].
- **저작권은 약하다**:
  - 미국: Thaler v. Perlmutter(D.C. Cir. 2025-03, 대법원 상고 기각 2026-03-02)와 저작권청 AI 보고서 Part 2(2025-01-29). 사람의 창작 기여가 없는 AI 결과물은 보호되지 않고, 프롬프트만으로는 모자란다 [1차 판결·정부 보고서, 검색 요약].
  - 한국: 문체부·한국저작권위원회 안내서(2025-06). AI 산출물 자체는 저작물이 아니다. 사람이 고치거나 더한 창작 부분, 그리고 선택·배열(편집저작물)만 보호될 수 있다 [1차 정부 안내, 2차 요약].
  - → **공개 레포의 AI 원본 PNG는 남이 가져가 써도 막기 어렵다**(§4.3) [추정].
- SynthID·C2PA 표시는 지우지 않는다(0138) [레포]. 이 판단이 맞다.

### 4.3 공개 레포 + 판매
- 지금 상태 [레포]: 오픈소스 라이선스가 없다(모든 권리 보유). AI 원본 그림, 변환한 스프라이트, 글이 공개되어 있다. Sunnyside만 비공개 레포(`AimenJoe/nail-assets`)에 있고, 대체 타일(`tiles.py`)이 있다. → **누구나 레포를 받아 거의 완전한 게임을 빌드할 수 있다.**
- 위험 [추정]:
  - Play 스토어·itch에 복제판이 올라온다(법적으로는 위반이지만 신고·대응은 사장 몫).
  - "레포에서 공짜로 빌드할 수 있다"는 것이 유료판 전환을 깎는다. 다만 빌드하는 사람은 아주 적다.
  - AI 그림은 저작권이 약해 무단 사용을 막기 어렵다.
- 비슷한 사례:
  - **Mindustry**: GPL, GitHub에 전체 공개 + itch 무료 + Steam 유료. Steam 판은 워크숍·통합으로 돈을 받는다 [2차].
  - **Shattered Pixel Dungeon**: GPL, 앱스토어·Steam에서 판다 [2차].
  - **고전 방식**(Doom 등): 코드는 공개하고 데이터(그림·레벨)는 판다 [기억].
  - Steamworks 문서는 GPL 같은 카피레프트 코드가 Steamworks SDK와 충돌할 수 있다고 경고한다 [1차 Steamworks 검색 요약]. 업적·클라우드를 위해 SDK를 붙일 거면 GPL로 가지 않는 편이 낫다.
- 고를 것(사장)
  - (A) **코드는 공개, 콘텐츠는 비공개**: 스프라이트·일러스트·글 표를 `nail-assets`로 옮긴다. Sunnyside와 같은 길이다. **추천**. 공개 개발 이야기를 지키면서 빌드 한 번으로 게임이 나오지 않는다. 단, 이미 공개된 역사는 되돌릴 수 없다.
  - (B) 판매를 시작할 때 레포를 비공개로. 가장 단순하지만 KB·공식 페이지·0228 클립 같은 "보여 주는 개발" 가치를 잃는다.
  - (C) GPL 등으로 활짝 연다(Mindustry 방식). 커뮤니티가 있어야 통한다. 지금은 없다.
- **웹 무료 전체판과 유료판의 관계**도 함께 정해야 한다. 0223은 "전체 그대로" 공개를 추천했다. 하지만 팔 거라면 그 추천과 부딪친다. 증거는 일화뿐이다: 무료 itch판이 Steam 출시를 크게 깎지 않았다는 사례, 오히려 도왔다는 사례, 약간 손해였다는 사례가 섞여 있다 [2차]. 길은 둘이다 [추정].
  - 웹은 데모로 줄인다(튜토리얼 + 첫 길 + 마을, 0223 ②안). 끝에 Steam 위시리스트 단추.
  - 웹은 전체 무료로 두고, 유료판에 네이티브 빌드·업적·클라우드·추가 캐릭터를 더한다.
  - 1인 개발에서 빌드 갈래가 늘어나는 비용(KB `action-roguelite-for-solo-dev`)은 기능 깃발(`--features demo`) 하나로 줄인다.

### 4.4 Sunnyside World·CC0·OFL
- **Sunnyside World**(danieldiggle) [1차, 사장이 2026-10-25 itch 페이지에서 옮긴 V1 원문, `assets/LICENSES.md`]:
  - 무료·상업 사용 가능, 수정 가능, 크레딧 불필요.
  - 수정해도 재포장·재판매 금지, 튜토리얼·교육 외 재배포 금지, **AI 학습 금지**.
  - → **판매용 게임에 넣는 것은 괜찮다.** 할 일: 지금 쓰는 팩은 **V2.1**인데 기록된 원문은 V1이다. 출시 전에 페이지 원문을 다시 떠서 날짜와 함께 보관한다. 팩 그림을 나노바나나 참고 그림으로 넣지 않는 규칙(0064)을 지킨다. "AI 학습"과 "생성 참고"의 경계가 흐리니 지금처럼 피하는 것이 안전하다.
- **CC0**(OpenGameArt 음악, Kenney 효과음): 판매에 지장 없다 [레포·CC0 정의].
- **OFL 1.1**(Galmuri·Ark Pixel·Cubic 11): 게임에 넣어 팔 수 있다. 원문·고지를 함께 배포하고, 폰트만 따로 팔지 않는다. 수정본에 예약 이름을 쓰지 않는다 [레포].
- 스토어에 내기 전에 **게임 안 크레딧·라이선스 화면**이 필요하다. APK·iOS에서는 고지 파일이 앱 안에만 있다 [레포, LICENSES.md가 이미 적어 둠].

### 4.5 한국 등급분류
- **원칙**: 게임산업법 제21조. 유통하거나 이용에 제공하려면 등급분류를 받는다 [1차 법령, 2차 요약].
- **비영리 면제**: 2019-09 시행령 개정으로 "비영리 단순공개 목적으로 창작한 게임물"은 면제된다 [1차 문체부 보도자료, 검색 요약]. 지금의 무료 웹·APK는 여기 해당할 가능성이 높다. 하지만 **판매·후원·광고를 붙이면 빠질 수 있다**. 현행 조문은 확인하지 못했다.
- **Google Play·App Store**: 자체등급분류사업자다. Play는 콘솔의 IARC 설문으로 등급을 받는다 [2차·기억]. 한국 배포도 그 등급으로 된다는 것이 통상 설명이니 확인 필요.
- **Steam**: Valve는 자체등급분류사업자로 지정되지 않았다(2024 보도, "관심 있음"). 2026년 현재 상태는 확인하지 못했다. 게임위는 "국내 유통 목적이면 등급을 받아야 한다"는 입장이다 [2차]. → **한국 개발자가 Steam에서 한국 이용자에게 팔려면 게임위 등급분류를 받아 두는 것이 안전하다.**
  - 수수료는 개인이 용량별로 약 2만 1천 원(10MB 미만)~16만 8천 원(300MB 이상). PC 300MB 이상은 36만 원이라는 보도도 있고, 중소기업은 30% 감면 [2차, 오래된 기사]. 웹 빌드가 약 68MB이니 중간 구간이다 [레포].
  - 개인사업자로 신청하면 지자체 등록증을 내라는 안내가 있다 [2차].
  - 2025-10-23 시행 개정은 국내 법인이 없는 *해외* 업체의 국내 대리인 지정에 관한 것이다 [2차].
  - 2025-04 공포 개정으로 경미한 내용 수정은 신고가 면제된다. 패치가 잦은 게임에는 좋은 소식이다 [2차].
- **STOVE**는 자체등급분류 권한이 있다는 2차 보도가 있다.

### 4.6 Android: 직접 설치(사이드로드)와 Play 스토어
- **개발자 확인(Developer Verification)** [1차 Android Developers 블로그·문서, 검색 요약]
  - 인증 기기(Android 7+)에서 확인되지 않은 개발자의 앱 설치를 막는 제도다. **2026-09-30 브라질·인도네시아·싱가포르·태국에서 시작, 2027년에 전 세계로**. 한국은 2027년.
  - 확인되지 않은 앱은 "고급 흐름"으로만 깔 수 있다: 개발자 옵션 → 경고 → **24시간 기다림** → 설치·업데이트 때마다 경고. ADB 설치는 그대로 된다 [2차].
  - 작은 개발자·취미용 **"제한 배포 계정"은 기기 20대까지, 신분증·수수료 없음**(2026-08 출시 예정이었고 실제 출시는 확인하지 못함).
  - → **2027년부터 GitHub APK를 받는 휴대 게임기 사용자에게 큰 문턱이 생긴다.** 2027년 전에 Android Developer Console에서 개발자 확인을 마쳐 APK 서명 키를 등록하는 것이 가장 싸다. 비용·필요 서류는 확인 필요.
- **Play 스토어**: targetSdk 36 요구(2026-08-31~)는 이미 맞췄다 [레포 `Cargo.toml`]. 개인 계정의 12명 × 14일 비공개 테스트, 가입비 $25 [기억]. 결제를 넣으려면 Play Billing 연동이 필요하다.

---

## 5. 추천 순서

### 0. 먼저 정할 것(사장)
**"팔 것인가"**. 팔지 않는다면 지금 구조(무료 웹·APK + 공식 페이지)가 맞고, 0223 공개 데모(itch + 의견 받기)를 진행하면 된다. 판다면 아래 순서로 간다. 이 조사의 결론은 "팔 수는 있지만, 그림 공개 표시를 그대로 둔 채 오래 준비하면 손해가 크다"이다.

### 1. 권리 정리 (몇 시간~며칠, 돈 거의 안 듦) — v0.5.x로 바로 할 수 있다
1. Gemini를 어떤 제품으로 언제 썼는지 기록하고, 그때의 약관 사본을 남긴다. `LICENSES.md`의 "확인 필요"를 닫는다.
2. Sunnyside V2.1 라이선스 원문을 다시 떠서 보관한다.
3. 게임 안 크레딧·라이선스 화면을 만든다(스토어 전 필수).
4. 레포 공개 범위 (A)/(B)/(C) 결정. 추천은 (A) 콘텐츠를 비공개 레포로.
5. 웹 무료판의 역할 결정(데모로 줄이기 또는 전체 유지).

### 2. Steam 페이지를 일찍 연다 (가장 큰 지렛대)
- Steamworks 가입(신분·은행·세금 W-8BEN), $100, 사업자등록·게임제작업 등록 확인, 세무사 상담 1회.
- 페이지에는 캡슐·키 아트(**사람 손**, $300~700), 30~60초 트레일러(KB `indie-game-landing-page`의 가짜 시계 클립), AI 공개 문구가 필요하다.
- **위시리스트는 페이지를 연 날부터 쌓인다.** 출시 6~12개월 전에 연다. 공식 페이지·웹판·APK 페이지에 "Steam에서 찜하기"를 단다.

### 3. 데모와 행사
- Steam 데모(웹 데모와 같은 빌드 깃발) → 행사 **몇 달 전에** 공개 → **Next Fest 2027-06**(현실적. 2027-02는 등록 마감 01-10이라 페이지·그림·데스크톱 빌드를 맞추기 빠듯하다).
- itch.io 페이지(0223 A5)는 Steam 페이지로 가는 길로.
- 0223의 A2(처음 15분)·A3(기술 점검)·A4(의견 보내기)는 데모의 품질 그 자체다. 그대로 쓴다.

### 4. 잴 것
| 지표 | 기준 |
|---|---|
| Steam 페이지 노출 → 방문 클릭률 | 2% 낮음, 7%+ 좋음 [2차] |
| 방문 → 위시리스트 비율, 하루 위시리스트 수 | — |
| 데모 플레이 → 위시리스트 전환(Steamworks 지표) | 중앙값 약 19% [2차, 자가 응답] |
| 데모 플레이 시간 중앙값, 첫 쉼터·첫 보스까지 간 비율 | 0223 A6 |
| 출시 뒤 환불률 | 정식 출시 중앙값 약 8% [2차] |
| 리뷰 수 × 30 | 판매 추정 |

### 5. v0.6 "오래 걸리는 큰 것" 후보 — 판매에 주는 근거로 순위
1. **"얼굴 그림을 사람 손으로" + 팔 수 있는 판** [근거 가장 많음, §4.1]
   - 캡슐·키 아트 외주, Naïl·모모·도토 스프라이트·초상화를 사람이 다시 찍기(`tools/pixel`), 가능하면 일러스트까지 → AI 표시를 줄이거나 없앤다.
   - 같은 판에 Windows/Linux 네이티브 빌드, Steam Deck 확인, (선택) 업적·클라우드.
   - 오래 걸리고 한 줄로 설명된다. 0227의 "큰 것 하나"에 맞는다.
2. **다회차 깊이**: 난이도 사다리 + 넷째 캐릭터 + 보스 갈래 [근거는 비교 게임 정도, §2.2]. 가격대를 $9.99 → $14.99로 올릴 근거가 된다. 리뷰에서 "짧다"는 말을 막는다.
3. **로컬 협동(2인, 같은 화면) + Steam Remote Play Together** [근거 약함]
   - 협동 게임 비율이 늘고 있다(Steam 신작 설명에 "co-op"이 쓰인 비율 2023년 1.4% → 2025년 2.1%) [2차 GameDiscoverCo]. 협동 히트작도 있다. Remote Play Together는 로컬 협동을 온라인으로 무료로 넓혀 준다(주인만 게임을 가지면 됨) [2차].
   - 하지만 협동이 판매를 끌어올린다는 체계적 자료는 찾지 못했다.
   - 카메라·밸런스·UI·튜토리얼을 다 다시 손봐야 해 비용이 크다. 온라인 넷코드는 1인에게 권하지 않는다 [추정].
4. 모바일 정식(Play 유료나 무료 + 열기) [근거 약함, 패드 전제와 안 맞음] — 1.0 뒤.

**사장에게 드리는 제안** [추정]: v0.6 = 1번(사람 그림 + 팔 수 있는 판), v0.7 = 2번(깊이), 그다음 Steam 출시 또는 EA. 2번을 먼저 하면 AI 표시를 단 채 출시를 미루게 되고, 그동안 쌓을 수 있었던 위시리스트를 잃는다.

### 6. KB에 남길 가치(KB 규칙 4)
"인디 게임 판매 경로와 숫자"(Steam 수수료·전환·Next Fest·AI 공개 규칙·한국 등급·세금)는 이 프로젝트가 사라져도 쓸모 있다. 원문을 열 수 있는 세션에서 1차 확인을 마친 뒤 `reference/`(정책·숫자)와 `concepts/`(판매 전략) 노트로 남기기를 제안한다. 이번 세션에서는 KB를 고치지 않았다.

---

## 출처

### 1차(공식·원 데이터. 모두 검색 요약으로만 읽음, 접근 날짜 2026-10-29)
- Steamworks Content Survey(AI 공개) — https://partner.steamgames.com/doc/gettingstarted/contentsurvey
- Steam Coming Soon 2주 규칙 — https://partner.steamgames.com/doc/store/coming_soon , Steam Direct — https://partner.steamgames.com/steamdirect
- Next Fest 2027-02 — https://partner.steamgames.com/doc/marketing/upcoming_events/nextfest/feb_2027 , 참가 조건 — https://partner.steamgames.com/doc/marketing/upcoming_events/nextfest
- Steamworks 오픈소스 배포 — https://partner.steamgames.com/doc/sdk/uploading/distributing_opensource
- Gemini API Additional Terms(보관본) — https://ai.google.dev/gemini-api/terms-archive/terms_12_20_24 , https://ai.google.dev/gemini-api/terms-archive/terms_05_20_25 , https://ai.google.dev/gemini-api/terms-archive/terms_09_25_25 , 현행 https://ai.google.dev/terms
- Google Play 수수료 15% — https://support.google.com/googleplay/android-developer/answer/10632485
- Play target API — https://developer.android.com/google/play/requirements/target-sdk
- Play 비공개 테스트 12명 — https://support.google.com/googleplay/android-developer/answer/14151465
- Android 개발자 확인 — https://android-developers.googleblog.com/2026/06/android-developer-verification.html , https://developer.android.com/developer-verification , https://support.google.com/android-developer-console/answer/16561738 , https://android-developers.googleblog.com/2025/08/elevating-android-security.html
- Apple 회비(한국어) — https://developer.apple.com/kr/support/membership-fee-waiver/ , Small Business Program — https://developer.apple.com/app-store/small-business-program/
- itch.io 개발자 FAQ — https://itch.io/developers/faq , 지급·VAT 공지 — https://itch.io/updates/a-new-payout-system-and-an-update-to-handling-vat
- Gamalytic 위시리스트/판매 — https://gamalytic.com/blog/exploring-steam-wishlist-sale-ratio , 인포그래픽 — https://gamalytic.com/blog/steam-revenue-infographic
- 미국 저작권청 AI 보고서 Part 2 — https://copyright.gov/newsnet/2025/1060.html
- Thaler v. Perlmutter 정리 — https://ipwatchdog.com/2025/03/24/d-c-circuit-upholds-human-authorship-requirement-thaler-v-perlmutter/ (판결 요약은 2차 법률 매체)
- 문체부 보도자료(비영리 게임물 등급분류 면제) — https://www.mcst.go.kr/site/s_notice/press/pressView.jsp?pSeq=17434
- 게임산업법(위키문헌) — https://ko.wikisource.org/wiki/게임산업진흥에_관한_법률
- [레포] `assets/LICENSES.md`(Sunnyside V1 원문 2026-10-25), `Cargo.toml`(targetSdk 36), `README.md`(모든 권리 보유), docs/decisions 0223·0227, docs/releases/v0.5.3
- [KB] concepts/public-demo-and-outside-feedback.md, guides/indie-game-landing-page.md, concepts/action-roguelite-for-solo-dev.md, guides/commissioning-game-art.md, guides/pixel-art-without-an-artist.md, guides/image-model-for-game-illustrations.md

### 2차(기사·블로그·벤더·포럼)
- 매출 분포: https://www.steampageanalyzer.com/blog/indie-game-revenue-data , https://www.steampageanalyzer.com/blog/median-steam-game-2026 , https://gamedevreports.substack.com/p/gamalytic-67-of-games-on-steam-earned , https://gamedevreports.substack.com/p/video-game-insights-indie-games-on , https://x.com/gamevestor/status/2046608611221049553
- Boxleiter: https://www.steampageanalyzer.com/blog/boxleiter-method-explained , https://www.steampageanalyzer.com/blog/steam-review-to-sales-multiplier , https://steamforecast.app/tools/steam-revenue-calculator
- 위시리스트 전환: https://gamedevreports.substack.com/p/gamediscoverco-the-state-of-steam , https://dev.to/indietooling/your-steam-wishlists-may-be-worth-half-what-they-were-bbm , https://howtomarketagame.com/2022/09/26/how-many-wishlists-should-i-have-when-i-launch-my-game/ , https://howtomarketagame.com/2024/01/29/do-wishlists-matter-any-more/ , https://www.steampageanalyzer.com/blog/how-many-wishlists-before-launch
- Next Fest: https://news.ycombinator.com/item?id=44489325 , https://www.steampageanalyzer.com/blog/do-steam-demos-increase-sales , https://www.steampageanalyzer.com/blog/after-steam-next-fest , https://fullcleared.com/news/steams-2027-spring-and-summer-sale-dates-are-locked-in/
- EA: https://newsletter.gamediscover.co/p/what-happened-to-early-access-graduates , https://nichegamer.com/new-data-shows-only-21-of-early-access-games-perform-better-after-full-release/ , https://gamedevreports.substack.com/p/gamediscoverco-steam-refunds-research
- 가격·비교 게임: https://newsletter.gamediscover.co/p/is-this-the-future-of-indie-game , https://gameworldobserver.com/2021/04/20/graph-only-1-of-games-released-on-steam-in-2020-cost-over-50-while-53-are-5-or-less , https://en.wikipedia.org/wiki/Halls_of_Torment , https://www.gamingonlinux.com/2023/06/halls-of-torment-hits-over-70k-sales-in-the-first-week/ , https://www.w4games.com/blog/w4-games-news-1/interview-with-chasing-carrots-developer-of-halls-of-torment-120 , https://en.wikipedia.org/wiki/Brotato , https://toucharcade.com/2023/03/28/top-down-roguelite-shooter-brotato-is-out-now-on-ios-and-android-as-a-premium-release/ , https://steamspy.com/dev/RubyDev , https://steambase.io/apps/tiny-rogues , https://en.wikipedia.org/wiki/Death%27s_Door_(video_game) , https://www.pcgamer.com/au/deaths-door-reaches-over-100000-players-in-its-first-week , https://rpgamer.com/2023/07/ember-knights-fully-released/ , https://www.levelup.com/en/news/cult-of-the-lamb-got-a-million-players-in-its-first-week/
- 모바일: https://www.pocketgamer.biz/balatro-approaches-1-million-in-seven-days-on-mobile , https://kotaku.com/vampire-survivors-free-iphone-steam-mobile-smartphone-1849955308 , https://www.pocketgamer.biz/vampire-survivors-developer-takes-new-approach-to-monetisation
- Play 수수료 변경(Epic 합의): https://www.neowin.net/amp/google-begins-rolling-out-its-post-epic-play-store-billing-model-next-week/ , https://www.notebookcheck.net/Google-cuts-Play-Store-fees-as-Epic-settlement-reshapes-Android-app-payments.1243278.0.html
- 사이드로드 고급 흐름: https://9to5google.com/2026/03/19/android-advanced-flow-sideloading/ , https://9to5google.com/2026/08/18/google-gradually-rolling-out-androids-advanced-sideloading-ahead-of-developer-verification/ , https://www.androidauthority.com/android-sideloading-developer-verification-first-wave-rollout-3717921/
- Steam 수수료: https://techcrunch.com/?p=1753729 , https://presskit.gg/field-guides/how-much-does-steam-take
- AI 공개 정책 보도: https://gamedeveloper.com/business/valve-tweaks-and-clarifies-ai-disclosure-rules-for-steam , https://videogameschronicle.com/news/valve-has-significantly-rewritten-steams-rules-for-how-developers-much-disclose-ai-use , https://gigazine.net/gsc_news/en/20260120-steam-updates-ai-disclosure-guidelines , https://legalmoveslawfirm.com/steam-ai-policy/
- AI 반응·판매: https://wnhub.io/news/analytics/item-51398 , https://otakukart.com/steam-survey-finds-majority-players-accept-ai-in-game-development-but-31-still-oppose-ai-generated-content/ , https://boingboing.net/2025/12/20/survey-finds-very-negative-attitude-toward-gen-ai-in-games.html , https://www.pcgamer.com/gaming-industry/steam-week-in-review-take-cover-because-it-looks-like-more-than-half-of-steam-games-will-have-an-ai-disclosure-by-2027-2028/ , https://app.cinevva.com/news/2026-07-20-steam-ai-disclosure-study , https://www.techpowerup.com/350230/games-made-with-gen-ai-suffer-up-to-53-worse-sales-on-steam , https://www.windowscentral.com/gaming/pc-gaming/ai-game-development-stigma-study
- Indie Game Awards: https://tbreak.com/indie-game-awards-clair-obscur-expedition-33-disqualified-gen-ai/ , https://www.pushsquare.com/news/2025/12/ps5-fave-expedition-33-stripped-of-indie-game-awards-but-not-for-the-reasons-you-may-think
- Google 약관 보도: https://9to5google.com/2024/04/16/google-tos-ai/ , https://cloud.google.com/terms/service-terms
- 한국 저작권 안내서: https://v.daum.net/v/20250613082054349?f=p , https://ditoday.com/gai-copyright/ , https://www.kadh.org/wp-content/uploads/2024/02/최종본-생성형-AI-저작권-안내서.pdf
- 한국 등급분류: https://www.gamemeca.com/view.php?gid=1751893 , https://www.edaily.co.kr/news/read?newsId=04414886622386256&mediaCodeNo=257 , https://www.globalgamers.co.kr/view.php?ud=202407031613365617c5fa75ef86_12 , https://www.inven.co.kr/webzine/news/?news=297165 , https://www.dehenglaw.com/CN/newscontent/0008/035090/2.aspx?MID=0902 , https://www.lawtimes.co.kr/news/208672 , https://www.shinkim.com/kor/media/newsletter/2774
- 한국 세금: https://ai.bznav.com/contents/3363748 , https://beancount.io/ko/blog/2026/07/09/steam-revenue-share-indie-game-developer-bookkeeping-guide , https://ai.bznav.com/contents/2713570 , https://wikidocs.net/446456 , https://www.taxesforexpats.com/country-guides/korea/us-korea-tax-treaty.html
- 오픈소스 상업 게임: https://www.gamingonlinux.com/2019/09/mindustry-the-factorio-like-tower-defense-game-is-now-on-steam/
- 무료 itch판과 Steam: https://itch.io/devlog/1142956/itchio-sales-reality-check-and-why-it-doesnt-mean-your-game-is-bad.amp , https://itch.io/devlog/1030005/gravity-storm-a-week-after-release.amp
- 협동: https://newsletter.gamediscover.co/p/what-game-descriptions-tell-us-about , https://game-wisdom.com/critical/remote-couch-coop
- 캡슐 외주 시세: https://www.steampageanalyzer.com/blog/indie-game-marketing-1000-budget , https://howtomarketagame.com/2020/04/13/how-one-new-image-increased-sales-by-20x/
- 후원 플랫폼: https://schoolmaker.com/blog/patreon-pricing , https://www.builtbyfoundry.io/blog/patreon-vs-ko-fi-2026-same-1000-month-189-or-101
- STOVE 계약 양식: https://www.lawinsider.com/ko/contracts/eVbxtU0FjsR

### 확인하지 못한 것
- Steam AI 공개 문구 원문(2026-01 개정판). 마케팅 그림이 예외인지.
- Gemini API 약관 2026-03-23판 본문.
- Valve의 한국 자체등급분류 지정 여부(2026). 현행 제21조 비영리 면제 문구. 게임위 수수료표(최신).
- Steam 정산의 미국 원천징수 세율(10%/15%)과 분류.
- STOVE 수수료. Google Play 가입비. Android 개발자 확인 비용.
- Hades·Dead Cells·Gungeon 등의 팀 크기와 출시가(기억에 의존).
- 첫해 판매 = 첫 주 × 2.5~4라는 배수(경험칙).

---

## 교차 확인(2026-10-29, 본 세션)
- **확인됨(1차, 원문을 직접 읽음)**: Android 개발자 확인 — 2026-08 제한 배포 계정·고급 흐름, 2026-09-30 브라질·인도네시아·싱가포르·태국, **2027년 전 세계**. 학생·취미 개발자는 **기기 20대까지 신분증·수수료 없이** 나눌 수 있다(https://developer.android.com/developer-verification).
- **못 읽음(세션 네트워크가 막음)**: partner.steamgames.com(AI 공개 문구·Next Fest·Steam Direct), gamalytic.com. 이 둘에 기댄 숫자·문구는 위 표기 그대로 "검색 요약"이다. 세션 환경의 허용 도메인에 넣으면 다시 확인한다.
- 레포 사실 확인: `Cargo.toml` targetSdk·`assets/LICENSES.md`의 Sunnyside V1 원문·README "모든 권리 보유"는 레포에서 직접 본 것과 같다.

## 교차 확인 2차(2026-10-29, 네트워크 허용 뒤 — 원문을 직접 받아 읽음)
| 주장(위 본문) | 원문 확인 | 결과 |
|---|---|---|
| Steam AI 공개: Pre-Generated = 개발 중 AI로 만들어 게임과 함께 나가고 플레이어가 보는 것(그림·소리·서사·현지화), Live-Generated는 가드레일 설명 추가 | Steamworks Content Survey | **확인**. "효율을 위한 AI 개발 도구는 이 절의 관심이 아니다" — 코드 도우미는 대상 아님. 마케팅 자료는 "게임이 마케팅 자료와 맞아야 한다"는 약속으로만 나오고, 공개 대상인지는 문서에 없다(본문 "확인 못 함" 그대로). 페이지에 개정 날짜 표기 없음 |
| AI 표시 게임: 신작 3분의 1, 매출 몫 10~27% | Sulka Haro 원문(fragwyz.substack.com, 2023-07~2026-07 전수 53,597개) | **확인**. 더해서: AI 표시 게임이 리뷰 100개(≈판매 3천) 이상에 닿는 비율은 비AI의 **약 55%**, 시기를 맞춰 비교해도 나아지지 않음. 망한 AI 게임 72%가 그림에 AI(성공작 57%), 망한 쪽 공개 문구는 짧다(중앙값 13단어 vs 22), 성공작은 "사람이 검토·다듬었다" 같은 문구가 더 많다(21% vs 12%). 매출은 리뷰×30 추정 |
| AI 표시 비율 10.9% → 19.9% → 30.8% | PC Gamer(Haro 인용) | **확인** |
| Steam Direct $100, $1,000 넘으면 돌려받음, 30일 대기 | Steamworks Steam Direct | **확인**. 은행·세금·신원 서류에 15~30일 |
| Coming Soon 2주 | Steamworks Coming Soon | **확인**. 검토 요청은 공개 7영업일 전. "2년 전에 찜한 사람도 2주 전에 찜한 사람만큼 산다" |
| Next Fest 2027-02-22~03-01, 등록 마감 2027-01-10 | Steamworks Next Fest Feb 2027 | **확인**(10:00 PST). 데모 빌드 검토는 3주 전, 보도 미리 보기용은 01-18 |
| "게임마다 한 번만"(본문 2차) | Steamworks Next Fest | **확인 — 1차**: "titles may only participate in ONE Next Fest". 페이지 공개·행사 시작 때 공개 데모·행사 끝까지 미출시. 데모 길이·독점 조건 없음 → **한 번뿐이니 시기를 고른다** |
| GPL은 Steamworks SDK와 문제 | Steamworks 오픈소스 배포 | **확인**. MIT·BSD·Apache 2.0은 괜찮다 |
| Gamalytic 첫 주 22%·첫 달 27%, EA ~20%·정식 ~30% | Gamalytic 블로그(2024-04-29) | **확인**. 상위 10%는 첫 달 134%, 하위 10%는 6%. 출시 할인 10~15%는 큰 차이 없음 |
| EA 졸업 91개 중 21%만 1.0이 더 팔림 | GameDiscoverCo 뉴스레터 | **확인**. EA 기간 중앙값 1년 3개월 |
| Gemini API 약관: 소유권 주장 안 함, 무료 사용분은 입력·결과를 제품 개선에 씀, 18세 이상·업무용 | ai.google.dev/gemini-api/terms | **확인 — 현행판 2026-03-23 시행**. 단 사장이 쓴 것이 Gemini **앱**(소비자)이면 이 약관이 아니라 Google 서비스 약관이다 → 어느 제품을 썼는지 기록이 필요하다는 결론은 그대로 |
| Google Play 수수료 15%(첫 $1M) → Epic 합의로 10% | Play Console 도움말(서비스 수수료) | **고침**: 새 구조(첫 $1M 10% + 결제 수수료 5% 등)는 **EEA·영국·미국(2026-06-30)·호주·일본(2026-09-30)** 사용자 거래에만. **한국을 포함한 나머지 시장은 아직 15% 구간**(첫 $1M 15%, 넘으면 30%). 한국은 대체 결제를 쓰면 4%p 낮다 |
| Play 개인 계정 12명 × 14일 비공개 테스트, targetSdk 36(2026-08-31~) | Play Console 도움말, Android Developers | **확인** |
| Android 개발자 확인: 2026-09-30 첫 나라들, 2027년 전 세계, 제한 배포 20대 무료 | Android Developers·Developer Console 도움말 | **확인**. 정식 배포 계정은 **$25**, 제한 배포 계정(학생·취미)은 수수료·신분증 없이 기기 20대. ADB 설치는 그대로 |
| Apple $99/년, Small Business 15% | Apple Developer | **확인**(15%는 전년도 수익 $1M 이하·신규. 회비 면제는 비영리·교육기관·정부만) |
| itch 수익 배분 | itch.io FAQ·공지 | **확인**: 개발자가 정한다, **기본 10%** |
| 미국 원천징수(W-8BEN) 10%/15% 불확실 | Steamworks 세금 FAQ + IRS 조세조약 표 1(2023-05판) | **확인**: 조약 없으면 30%, 한국은 **저작권 사용료 10%**(특허·노하우 15%, 영화·TV 10%). Steam은 "royalty copyright" 세율을 게임 판매에 쓴다. TIN을 나중에 넣어도 이미 뗀 것은 돌려주지 않는다 → 첫 정산 전에 W-8BEN에 납세자번호를 넣는다 |
| Sunnyside V2.1인데 기록은 V1 | danieldiggle.itch.io/sunnyside | **확인**: 지금 페이지의 라이선스도 **V1**(상업 사용·수정 가능, 재포장·재판매 금지, AI 학습 금지). V2.1은 팩 버전이다. 페이지에 "No generative AI was used" |
| 비영리 단순공개 게임 등급분류 면제(2019-09-03 시행령) | 문체부 보도자료 | **확인**(청소년이용불가 내용이면 제외) |
| Valve 자체등급분류사업자 | 검색(2020·2024 보도) | **여전히 확인 못 함**. 2024-07 "밸브가 관심" 뒤 지정 보도 없음 |
| 게임위 등급분류 수수료(개인·용량별) | 검색 | **못 찾음**. 플랫폼별 기초가액 × 계수 방식. 게임위 신청 화면의 모의 수수료 계산으로 확인 |
| STOVE 수수료 | — | 못 찾음(공개 자료 없음) |

바뀐 결론: Google Play를 쓴다면 한국 기준 수수료는 15%(10% 아님). 나머지 결론(가격대·AI 표시 위험·Next Fest 전략·권리 정리 순서)은 원문과 맞다. Next Fest가 **게임당 한 번**이라는 것이 1차로 확인돼, "데모를 몇 달 먼저 내고 준비된 판으로 한 번"이 더 중요해졌다.

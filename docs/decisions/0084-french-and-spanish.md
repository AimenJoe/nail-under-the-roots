# 0084. 프랑스어·스페인어를 기본 지원 언어로

- 날짜: 2026-10-26
- 상태: 채택(v0.1.14)
- 사장: "french랑 Spanish도 지원하자."

## 결정
1. `Lang::Fr`("Français")·`Lang::Es`("Español")를 브라질 포르투갈어 다음, 가짜 언어 앞에 둔다. 저장 칸 값은 6·7(`prefs.rs`)이다. 칸 형식은 그대로라 MAGIC은 그대로 둔다. 옛 저장의 언어 값은 바뀌지 않는다.
2. 기기 언어: `fr-*`(프랑스·캐나다·벨기에·스위스)는 모두 프랑스어, `es-*`(스페인·중남미 `es-419`)는 모두 스페인어로 받는다. 스페인어는 지역색이 옅은 중립 스페인어다.
3. 폰트: Galmuri11 Condensed. Latin-1은 다 있었지만 프랑스어의 **œ Œ Ÿ**가 서브셋에 없었다. 원본(quiple/galmuri `dist/Galmuri11-Condensed.ttf`, 같은 2.404판)에서 "지금 있는 글자 + 셋"으로 다시 줄였다. 이미 있던 글자 2,644개의 윤곽이 원본과 같은 것을 fontTools로 비교했다. 파일은 452KB → 339KB가 됐다(이름 표·글리프 이름 정리).
4. 번역: 손으로 썼다(기계 번역 없음, CLAUDE.md). 언어마다 따로 맡겨 영어 표를 바탕으로, 뜻이 모호한 곳은 한국어 표와 코드에서 쓰이는 자리를 보고 정했다. 브라질 포르투갈어(0033)의 규칙을 따른다.
   - 말투는 반말(tu / tú)이다. **플레이어 성별을 정하지 않는다**(프랑스어 "Bienvenue!"는 성이 없는 명사형이라 그대로, 스페인어는 "Bienvenido" 대신 "¡Pasa, pasa!", 다침·쓰러짐은 성이 붙지 않는 말로 — "Au tapis…").
   - 스페인어에서 Naïl은 늙은 대장 보리에게만 usted를 쓴다(한국어의 "요"에 맞춤).
   - 이름은 영어 표 그대로 둔다(누룩 할매 = Mamie Nuruk · Abuela Nuruk, 엄마 = Maman · Mamá).
5. 용어(FR / ES):

   | 한국어 | 프랑스어 | 스페인어 |
   |---|---|---|
   | 굴 | Terrier | Madriguera |
   | 굴파기 | Creuser | Cavar |
   | 솟구치기 | jaillir | emerger |
   | 쉼터 | Refuge | Refugio |
   | 샘 | Source | Manantial |
   | 저장고 | Réserve | Despensa |
   | 길의 주인 | Gardien | Guardián |
   | 뿌리 길 | Chemin des racines | Camino de raíces |
   | 가장 깊은 곳 | le Tréfonds | lo más profundo |
   | 반딧불 | Lucioles | Luciérnagas |
   | 씨앗 | Graines | Semillas |
   | 장신구 | Breloques | Amuletos |
   | 뿌리 심장 | Cœur de racine | Corazón de raíz |
   | 산딸기 | Framboise | Frambuesa |
   | 쥬시 지렁이 | Ver juteux | Lombriz jugosa |
   | 패링 | Parade | Desvío |
   | 분노 | Rage | Furia |
   | 드릴 돌진 | Ruée foreuse | Taladrazo |
   | 실뜨기 | Jeu de ficelle | Cuna de gato |
   | 저널 | Journal | Diario |
   | 도감 | Bestiaire | Bestiario |
   | 큰 나무 | Grand Arbre | Gran Árbol |
   | 설정 | Options | Ajustes |
   | 패드 | Manette | Mando |
   | 체력 | PV | PV |
6. 넘침: 헤드리스 Chromium(locale fr-FR·es-ES)으로 타이틀·슬롯·설정·조작·쉼터·일시정지 여섯 탭을 찍었다. 그렇게 고친 것:
   - 조작 안내 줄을 줄였다.
   - 가방의 콤보 줄은 칸 폭에 맞추게(`ui.fit`) 코드를 고치고 글도 줄였다.
   - 스페인어 "Recarga cavar", "Lombr.", "Guardián"을 줄였다.
   - 다른 라벨들은 영어·포르투갈어보다 조금 길지만, 고정 칸은 `ui.fit`(…)이 막고 긴 글은 줄을 바꾼다.
7. 프랑스어 문장부호: ! ? : 앞에 띄어쓰기를 하지 않았다(작은 화면에서 줄바꿈이 부호만 다음 줄로 넘기지 않게). 프랑스어 표기 관례와는 다르다. 원어민 검수 때 정한다.

## 남은 일
- 원어민 검수: 특히 대사 말투, 쥐 종류 이름, 프랑스어 문장부호 띄어쓰기, 스페인어 "Mando"(스페인) / "Control"(중남미).
- 넘침을 찍지 않은 화면이 있다: 가게·큰 나무·실의 꿈·길 지도·실뜨기 고르기·컷씬 대사창.
- 글을 넣거나 바꿀 때 이제 **일곱 언어**를 다 쓴다(CLAUDE.md).

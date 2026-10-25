# 0033: 브라질 포르투갈어를 기본 지원 언어로

- 상태: 채택 (2026-10-25, 사장 "포르투갈(브라질)어도 기본으로 지원해줘"). 0014가 "글 동결 뒤"로 미뤄 둔 1차 언어 중 하나를 앞당긴다.

## 결정
1. `Lang::PtBr`(설정 → 언어 "Português (BR)", 간체 다음·가짜 언어 앞). 저장 칸 번호 5(`prefs.rs`, 칸 형식은 그대로라 MAGIC 유지).
2. 기기 언어: `pt-*`는 모두 브라질 포르투갈어로(포르투갈 pt-PT 포함). 글자는 같고 낱말·존댓말이 조금 다를 뿐이라 영어보다 가깝다. 번체를 간체로 바꾸지 않는 것(0014)과 다른 이유: 그쪽은 글자 체계가 다르다.
3. 폰트: 한국어·영어와 같은 Galmuri11 Condensed. ã õ â ê ô á é í ó ú à ç와 대문자가 다 있다(fontTools로 cmap 확인, 테스트 `every_glyph_is_in_the_font`). CJK처럼 서브셋을 다시 만들 필요 없음.
4. 번역 과정: 영어 표 → DeepL(무료, 사용자 지침 무시됨) 초안 → 한 줄씩 손으로 검토(한국어·영어 대조). 초안이 틀린 것이 많았다 — Resume → "Currículo"(이력서), On → "Em", Spring(샘) → "Primavera"(봄), Drill Dash → "Corrida de Exercícios", Back closes → "Fechamento das vendas". 짧은 UI 조각일수록 틀린다.
5. 말투: 반말에 가까운 "você", 짧게. **플레이어 성별을 정하지 않는다** — 포르투갈어는 형용사가 성을 따르므로 "Bem-vindo" 대신 "Boas-vindas", "você está machucado" 대신 "você se machucou". Naïl은 "toupeirinha"(toupeira는 문법상 여성 명사라 성별과 무관). 모모는 아직 이름을 모르므로 "aquela criaturinha".
6. 용어: 쉼터 Refúgio · 샘 Nascente · 저장고 Despensa · 둥지 Ninho · 길의 주인 Dono da trilha · 뿌리 길 Trilha de raízes · 굴파기 Cavar · 솟구치기 emergir · 반딧불 Vaga-lumes · 씨앗 Sementes · 장신구 Amuletos · 뿌리 심장 Coração de raiz · 산딸기 Framboesa · 쥬시 지렁이 Minhoca suculenta · 패링 Aparar · 분노 Fúria · 시궁쥐 Ratazana · 다람쥐 Esquilo-listrado · 청설모 Esquilo-vermelho · 패드 Controle · 설정 Opções.
7. 넘침: 헤드리스 Chromium(locale pt-BR)으로 타이틀·설정·조작·쉼터·가게·일시정지 6탭을 찍어 넘침 3곳을 줄였다("Configurações" → "Opções", 능력치 이름 둘, 조작 안내).

## 남은 일
- 원어민 검수(특히 대사 말투, 쥐 종류 이름).
- 글을 넣거나 바꿀 때 이제 **다섯 언어**(CLAUDE.md).

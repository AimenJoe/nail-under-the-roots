"""EOS 업적 일괄 올리기 zip(0276 E3).

실행: python3 tools/epic/make_eos_zip.py [출력.zip]   (기본: target/eos-achievements.zip — 레포에 넣지 않는다)

묶는 것: docs/epic/achievementDefinitions.csv · achievementLocalizations.csv · 빈 stats.csv(머리줄만)(같은 시험
`cargo test write_steam_achievement_sheet -- --ignored`가 만든다) + docs/steam/icons/<API>_ACHIEVED.png·_UNACHIEVED.png.
꼴은 Epic 개발자 포털의 BulkTemplate.zip(2026-11-04 받아 봄): 모든 파일이 zip 맨 위, 언어 표의 아이콘 칸이 파일 이름.
포털: Epic Online Services → Player & Game Data → Achievements → Create achievement → Bulk Import.
Epic 상점 업적(XP)은 따로 올린다 — docs/epic/epic-store-xp.csv를 포털의 Epic Games Store 탭 템플릿에 옮긴다.
"""
import csv
import os
import sys
import zipfile

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else os.path.join(ROOT, 'target', 'eos-achievements.zip')
    epic = os.path.join(ROOT, 'docs', 'epic')
    icons = os.path.join(ROOT, 'docs', 'steam', 'icons')
    loc = os.path.join(epic, 'achievementLocalizations.csv')
    need = set()
    with open(loc, encoding='utf-8') as f:
        for row in csv.DictReader(f):
            need.add(row['lockedIcon'])
            need.add(row['unlockedIcon'])
    missing = [n for n in sorted(need) if not os.path.isfile(os.path.join(icons, n))]
    if missing:
        sys.exit(f'아이콘이 없다: {missing[:5]} … (python3 tools/steam/achievement_icons.py)')
    os.makedirs(os.path.dirname(out), exist_ok=True)
    with zipfile.ZipFile(out, 'w', zipfile.ZIP_DEFLATED) as z:
        for name in ('achievementDefinitions.csv', 'achievementLocalizations.csv'):
            z.write(os.path.join(epic, name), name)
        # 통계(stat)는 쓰지 않지만(업적은 게임이 바로 연다) 포털이 stats.csv가 없으면 "File not found: stats.csv"로 거절한다
        # (2026-11-06) — 머리줄만. 칸은 BulkTemplate.zip 그대로.
        z.writestr('stats.csv', 'name,aggregationType\n')
        for n in sorted(need):
            z.write(os.path.join(icons, n), n)
    print(out, len(need), 'icons')


if __name__ == '__main__':
    main()

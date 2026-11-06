"""EOS 업적 일괄 올리기 zip(0276 E3).

실행: python3 tools/epic/make_eos_zip.py [--egs] [출력.zip]   (기본: target/eos-achievements.zip · --egs면 target/egs-achievements.zip — 레포에 넣지 않는다)

묶는 것: docs/epic/achievementDefinitions.csv · achievementLocalizations.csv · 빈 stats.csv(머리줄만)(같은 시험
`cargo test write_steam_achievement_sheet -- --ignored`가 만든다) + docs/steam/icons/<API>_ACHIEVED.png·_UNACHIEVED.png.
꼴은 Epic 개발자 포털의 BulkTemplate.zip(2026-11-04 받아 봄): 모든 파일이 zip 맨 위, 언어 표의 아이콘 칸이 파일 이름.
포털: Epic Online Services → Player & Game Data → Achievements → Create achievement → Bulk Import.
--egs: 포털 Achievements의 Epic Games Store 탭 일괄 올리기(BulkTemplateEGS.zip, 2026-11-06 받아 봄) — 꼴은 같고
정의 표 끝에 칸 `user_epic_achievements_xp` 하나가 붙는다. 값은 docs/epic/epic-store-xp.csv(합 1,000, 올린 뒤 못 바꿈).
"""
import csv
import os
import sys
import zipfile

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def egs_definitions(epic):
    """정의 표 + XP 칸(BulkTemplateEGS.zip 머리줄 그대로)."""
    with open(os.path.join(epic, 'epic-store-xp.csv'), encoding='utf-8') as f:
        xp = {r['name']: r['xp'] for r in csv.DictReader(f)}
    with open(os.path.join(epic, 'achievementDefinitions.csv'), encoding='utf-8') as f:
        rows = list(csv.DictReader(f))
    if set(xp) != {r['name'] for r in rows}:
        sys.exit('epic-store-xp.csv와 achievementDefinitions.csv의 업적이 다르다')
    if sum(int(v) for v in xp.values()) != 1000:
        sys.exit('XP 합이 1,000이 아니다')
    lines = ['name,hidden,statThresholds,user_epic_achievements_xp']
    lines += [f"{r['name']},{r['hidden']},{r['statThresholds']},{xp[r['name']]}" for r in rows]
    return '\n'.join(lines) + '\n'


def main():
    args = [a for a in sys.argv[1:] if a != '--egs']
    egs = '--egs' in sys.argv[1:]
    default = 'egs-achievements.zip' if egs else 'eos-achievements.zip'
    out = args[0] if args else os.path.join(ROOT, 'target', default)
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
        if egs:
            z.writestr('achievementDefinitions.csv', egs_definitions(epic))
        else:
            z.write(os.path.join(epic, 'achievementDefinitions.csv'), 'achievementDefinitions.csv')
        z.write(os.path.join(epic, 'achievementLocalizations.csv'), 'achievementLocalizations.csv')
        # 통계(stat)는 쓰지 않지만(업적은 게임이 바로 연다) 포털이 stats.csv가 없으면 "File not found: stats.csv"로 거절한다
        # (2026-11-06) — 머리줄만. 칸은 BulkTemplate.zip 그대로.
        z.writestr('stats.csv', 'name,aggregationType\n')
        for n in sorted(need):
            z.write(os.path.join(icons, n), n)
    print(out, len(need), 'icons')


if __name__ == '__main__':
    main()

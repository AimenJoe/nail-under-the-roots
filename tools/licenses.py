"""제3자 라이선스 고지 파일 생성기. 의존성이 바뀌면 다시 돌린다(CLAUDE.md "커밋 전").

실행: python3 tools/licenses.py  → assets/THIRD_PARTY_LICENSES.txt
- cargo metadata로 모든 크레이트(모든 플랫폼)의 이름·버전·SPDX를 모으고, 크레이트 폴더의 LICENSE*/COPYING*/NOTICE* 원문을 붙인다.
- 같은 원문은 한 번만 싣고 해당 크레이트를 나열한다.
- 웹 JS 복사본(mq_js_bundle.js = macroquad, macroquad-gamepads = gamepads)과 폰트(Galmuri, OFL)도 싣는다.
MIT·BSD·Apache·ISC·Unicode 라이선스는 바이너리 배포에도 저작권 고지와 라이선스 원문을 함께 줄 것을 요구한다.
"""
import hashlib
import json
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, 'assets', 'THIRD_PARTY_LICENSES.txt')
PREFIXES = ('license', 'licence', 'copying', 'notice', 'unlicense')


def license_texts(pkg_dir):
    out = []
    try:
        names = sorted(os.listdir(pkg_dir))
    except FileNotFoundError:
        return out
    for n in names:
        if n.lower().startswith(PREFIXES):
            p = os.path.join(pkg_dir, n)
            if os.path.isfile(p):
                with open(p, encoding='utf-8', errors='replace') as f:
                    out.append((n, f.read().strip()))
    return out


def main():
    meta = json.loads(subprocess.check_output(
        ['cargo', 'metadata', '--format-version', '1'], cwd=ROOT))
    groups = {}   # hash -> (filename, text, [crates])
    no_file = []
    for p in sorted(meta['packages'], key=lambda p: (p['name'], p['version'])):
        if p['name'] == 'nail':
            continue
        label = f"{p['name']} {p['version']} ({p.get('license') or 'license file'})"
        texts = license_texts(os.path.dirname(p['manifest_path']))
        if not texts:
            no_file.append(label)
            continue
        for fname, text in texts:
            h = hashlib.sha256(text.encode()).hexdigest()
            groups.setdefault(h, [fname, text, []])[2].append(label)

    lines = [
        'Naïl: Under the Roots — 제3자 라이선스 고지 / Third-party license notices',
        '이 파일은 tools/licenses.py가 만든다. 손으로 고치지 않는다.',
        '',
        '== 웹 JS ==',
        'web/mq_js_bundle.js: macroquad(not-fl3) — MIT OR Apache-2.0 (아래 macroquad 항목의 원문)',
        'web/macroquad-gamepads-0.1.js: gamepads(fornwall) — MIT OR Apache-2.0 (아래 gamepads 항목의 원문)',
        '',
        '== 폰트 ==',
        'assets/fonts/Galmuri11-Condensed.ttf — Galmuri, Copyright (c) 2019-2025 Lee Minseo, SIL Open Font License 1.1.',
        '원문: assets/fonts/OFL.md (예약 폰트 이름 없음. 글리프를 추린 서브셋)',
        '',
        '== Rust 크레이트 ==',
    ]
    for fname, text, crates in sorted(groups.values(), key=lambda g: g[2][0]):
        lines.append('-' * 72)
        lines.append(f'[{fname}] 적용 대상:')
        lines.extend('  ' + c for c in crates)
        lines.append('')
        lines.append(text)
        lines.append('')
    if no_file:
        lines.append('-' * 72)
        lines.append('라이선스 파일이 크레이트에 없는 항목(SPDX 표기만):')
        lines.extend('  ' + c for c in no_file)
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, 'w', encoding='utf-8') as f:
        f.write('\n'.join(lines) + '\n')
    print(f'{OUT}: {len(groups)} texts, {sum(len(g[2]) for g in groups.values())} crate entries, {len(no_file)} without file')


if __name__ == '__main__':
    sys.exit(main())

#!/usr/bin/env python3
"""웹 배포 폴더 조립(0269): deploy-web.yml·smoke-web.yml·README가 같이 쓴다(전엔 세 곳에 같은 cp 줄이 있었다).

    python3 tools/web/assemble.py dist <빌드 아이디> <release|dev>

- web/의 페이지·JS·매니페스트·_headers·아이콘 + nail.wasm + assets(웹은 mp3만 — 네이티브 ogg는 뺀다, 0235 R20) + 제3자 고지.
- sw.js의 __BUILD__ 를 빌드 아이디로(배포마다 sw.js가 바뀌어 새 워커가 깔린다), __PRECACHE__ 자리에 미리 받을 파일 목록
  (음악·팩 원본 빼고 — 음악은 23MB라 들을 때, 팩은 처음 읽을 때 담는다).
- build.txt = "<빌드 아이디> <갈래>"(배포 확인용).
- 개발판(dev): 탭·설치 이름에 "(dev)", 검색에 안 잡히게(noindex).
"""
import json
import os
import shutil
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def main():
    if len(sys.argv) != 4 or sys.argv[3] not in ('release', 'dev'):
        sys.exit(__doc__)
    out, build, flavor = os.path.abspath(sys.argv[1]), sys.argv[2], sys.argv[3]
    web = os.path.join(ROOT, 'web')
    shutil.rmtree(out, ignore_errors=True)
    os.makedirs(out)
    for name in os.listdir(web):
        src = os.path.join(web, name)
        if name.endswith(('.html', '.js', '.webmanifest')) or name == '_headers':
            shutil.copy(src, out)
    shutil.copytree(os.path.join(web, 'icons'), os.path.join(out, 'icons'))
    shutil.copy(os.path.join(ROOT, 'assets', 'THIRD_PARTY_LICENSES.txt'), out)
    shutil.copy(os.path.join(ROOT, 'target', 'wasm32-unknown-unknown', 'release', 'nail.wasm'), out)
    shutil.copytree(os.path.join(ROOT, 'assets'), os.path.join(out, 'assets'),
                    ignore=shutil.ignore_patterns('.git', '*.ogg', '__MACOSX'))
    with open(os.path.join(out, 'build.txt'), 'w') as f:
        f.write(f'{build} {flavor}\n')

    # 미리 받을 목록: 음악·고지·확인 파일 빼고. 경로는 페이지 기준 상대.
    pre = []
    for dirpath, _, files in os.walk(out):
        for fn in files:
            rel = os.path.relpath(os.path.join(dirpath, fn), out).replace(os.sep, '/')
            # 팩 원본(assets/vendor, 0010)은 처음 읽을 때 담는다 — 빌드마다 팩 폴더의 크기가 달라(로컬은 팩 전체) 목록을 키우지 않게.
            if rel.startswith(('assets/music/', 'assets/vendor/')) or rel in ('sw.js', '_headers', 'build.txt', 'THIRD_PARTY_LICENSES.txt') \
                    or rel.endswith(('.md', '.txt')):
                continue
            pre.append(rel)
    pre.sort()
    sw = os.path.join(out, 'sw.js')
    with open(sw, encoding='utf-8') as f:
        text = f.read()
    assert '__BUILD__' in text and '/*__PRECACHE__*/[]' in text, 'sw.js 자리표가 없다'
    text = text.replace('__BUILD__', build).replace('/*__PRECACHE__*/[]', json.dumps(pre))
    with open(sw, 'w', encoding='utf-8') as f:
        f.write(text)

    if flavor == 'dev':
        page = os.path.join(out, 'index.html')
        with open(page, encoding='utf-8') as f:
            html = f.read()
        html = html.replace('<title>Naïl: Under the Roots</title>', '<title>Naïl: Under the Roots (dev)</title>', 1)
        html = html.replace('<meta charset="utf-8">', '<meta charset="utf-8">\n  <meta name="robots" content="noindex, nofollow">', 1)
        with open(page, 'w', encoding='utf-8') as f:
            f.write(html)
        man = os.path.join(out, 'manifest.webmanifest')
        with open(man, encoding='utf-8') as f:
            m = json.load(f)
        m['name'] += ' (dev)'
        m['short_name'] += ' dev'
        with open(man, 'w', encoding='utf-8') as f:
            json.dump(m, f, ensure_ascii=False, indent=2)
    size = sum(os.path.getsize(os.path.join(out, p)) for p in pre)
    print(f'{out}: {len(pre)} files precached ({size / 1e6:.1f} MB), flavor {flavor}, build {build}')


if __name__ == '__main__':
    main()

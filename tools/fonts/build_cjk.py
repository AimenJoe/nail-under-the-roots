"""일본어·중국어(간체·번체) UI 폰트 생성기(docs/decisions/0014, 번체 0144). 산출물(TTF)을 커밋하고, 이 파일이 원본이다.

실행: python3 tools/fonts/build_cjk.py <ark-pixel-font 체크아웃> <Cubic_11.woff2>
  예: git clone --depth 1 https://github.com/TakWolf/ark-pixel-font /tmp/ark && python3 tools/fonts/build_cjk.py /tmp/ark /tmp/Cubic_11.woff2
  Cubic_11.woff2: 俐方體11號(https://github.com/ACh-K/Cubic-11, OFL 1.1) 릴리스의 fonts/web/Cubic_11.woff2(v1.500).
  → assets/fonts/NailCJK-ja.ttf, assets/fonts/NailCJK-zh-Hans.ttf, assets/fonts/NailCJK-zh-Hant.ttf

- 글자 원본: Ark Pixel Font 12px 비례(proportional)·공통(common) PNG(OFL 1.1, TakWolf). 파일 이름의 언어 표시
  (`76F4 zh_hant,ja,ko.png`)로 그 언어의 모양을 고르고, 없으면 기본 모양(`76F4.png`).
- 넣는 글자: src/i18n.rs의 그 언어 표에 실제로 쓰인 글자 + ASCII + 코드의 UI 기호(▶ ♥ 등) + (일본어) 가나 전부.
  표가 바뀌면 다시 돌린다. 빠진 글자는 `cargo test`(i18n::tests::every_glyph_is_in_the_font)가 잡는다.
- 번체(0144): Ark에는 번체 한자가 많이 빠졌다(擊 裝 沒 換 層 …, Ark README "仍然缺少大量字符"). Ark에 없으면 俐方體11號(Cubic 11)를
  12px로 찍어 메운다. Ark의 자매 폰트 fusion-pixel-font가 12px 번체를 메우는 방법 그대로(크기 12, 찍는 자리 (-1, +1),
  위아래 빈 줄을 덜어 12줄, tools/resource/dump_service.py). 간체·일본어는 Cubic을 쓰지 않는다(모양이 다르다).
- Ark에 없는 글자: 기호·라틴은 Galmuri11 Condensed(assets/fonts, OFL)에서 빌려 온다. 한자·가나가 없으면 실패한다 →
  문구를 바꾸거나(攻撃 → こうげき) tools/fonts/patch/<언어>/<16진수>.png(12×12, 검은 칸 = 잉크)를 직접 찍는다.
- 픽셀 하나 = 100 유닛(upem 1200 = 12px). 기준선 계산은 Ark 빌더(pixel-font-knife)와 같다.
- OFL: 서브셋도 수정본이다. 이름을 바꿔(Nail CJK) 내고, 원 저작권·라이선스를 name 테이블과 고지 파일에 남긴다.
"""
import glob
import math
import os
import re
import subprocess
import sys

from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.ttLib import TTFont
from PIL import Image, ImageDraw, ImageFont

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
UPEM, U = 1200, 100
EM, BASELINE = 12, 11            # Ark assets/configs/fonts/font-12px.yaml (proportional)
ASCENT, DESCENT = 1200, -200     # Galmuri11 Condensed와 같게(줄 높이·기준선이 언어마다 같도록)
KANA = [chr(c) for c in range(0x3041, 0x3097)] + [chr(c) for c in range(0x30A1, 0x30FB)] + ['ー', '・', '、', '。', '「', '」']

TARGETS = {
    # 언어 표(src/i18n.rs의 static 이름), Ark 언어 표시, 출력 파일, 이름 꼬리
    'ja': ('JA', 'ja', 'NailCJK-ja.ttf', 'JA'),
    'zh_hans': ('ZH_HANS', 'zh_hans', 'NailCJK-zh-Hans.ttf', 'SC'),
    # 번체는 대만 모양: Ark의 zh_tw 표시가 먼저, 없으면 zh_hant, 그다음 기본. 홍콩·마카오(zh_hk)도 이 폰트 하나로(0144).
    'zh_hant': ('ZH_HANT', ('zh_tw', 'zh_hant'), 'NailCJK-zh-Hant.ttf', 'TC'),
}
# Ark에 없는 한자를 Cubic 11에서 메우는 언어.
CUBIC_FILL = {'zh_hant'}
CUBIC_OFFSET = (-1, 1)   # fusion-pixel-font assets/configs/dump.yaml cubic-11 rasterize-offset


def table_chars(static_name):
    src = open(os.path.join(ROOT, 'src', 'i18n.rs'), encoding='utf-8').read()
    m = re.search(r'pub static %s: Strings = Strings \{(.*?)\n\};' % static_name, src, re.S)
    if not m:
        sys.exit(f'src/i18n.rs에 {static_name} 표가 없다')
    return set(''.join(re.findall(r'"((?:[^"\\]|\\.)*)"', m.group(1))))


def ui_symbol_chars():
    """i18n 밖 코드가 직접 찍는 기호(▶ ◀ ▼ ■ □ ♥ ♡ …). 한글은 개발 화면용(기본 폰트)이라 뺀다."""
    out = set()
    for p in glob.glob(os.path.join(ROOT, 'src', '*.rs')):
        if p.endswith('i18n.rs'):
            continue
        code = '\n'.join(l for l in open(p, encoding='utf-8') if not l.lstrip().startswith('//'))
        for lit in re.findall(r'"((?:[^"\\]|\\.)*)"', code) + re.findall(r"'(.)'", code):
            out |= {c for c in lit if not ('가' <= c <= '힣') and not ('ㄱ' <= c <= 'ㆎ')}
    return out


def ark_index(ark):
    idx = {}
    for p in glob.glob(os.path.join(ark, 'assets/glyphs/12/cmap/*/**/*.png'), recursive=True):
        kind = p.split('/cmap/')[1].split('/')[0]
        if kind == 'monospaced':
            continue
        name = os.path.basename(p)[:-4]
        hexpart, _, fl = name.partition(' ')
        try:
            cp = int(hexpart, 16)
        except ValueError:
            continue
        idx.setdefault(cp, []).append((set(fl.split(',')) if fl else None, kind, p))
    return idx


def ark_glyph(idx, ch, flavors):
    """flavors(앞이 우선)의 언어 모양, 없으면 기본 모양."""
    cands = idx.get(ord(ch), [])
    wants = [lambda f, fl=fl: f is not None and fl in f for fl in flavors] + [lambda f: f is None]
    for want in wants:
        for kind in ('proportional', 'common'):
            for f, k, p in cands:
                if k == kind and want(f):
                    return p
    return None


def cubic_image(cubic, ch):
    """Cubic 11을 12px로 찍은 그림(fusion-pixel-font dump_service와 같게). 글자가 없으면 None."""
    font, image_font = cubic
    cmap = font.getBestCmap()
    if ord(ch) not in cmap:
        return None
    upem = font['head'].unitsPerEm
    h = math.ceil((font['hhea'].ascent - font['hhea'].descent) / upem * EM)
    if (h - EM) % 2:
        h += 1
    w = min(math.ceil(font['hmtx'].metrics[cmap[ord(ch)]][0] / upem * EM), EM)
    if w <= 0:
        return None
    im = Image.new('RGBA', (w, h), (0, 0, 0, 0))
    ImageDraw.Draw(im).text(CUBIC_OFFSET, ch, fill=(0, 0, 0, 255), font=image_font)
    # 위아래 같은 수의 빈 줄을 덜어 12줄로(잉크가 있으면 그만큼만).
    rows = [any(im.getpixel((x, y))[3] > 127 for x in range(w)) for y in range(h)]
    ink = [y for y, r in enumerate(rows) if r]
    top, bottom = (ink[0], h - 1 - ink[-1]) if ink else (h, h)
    pad = min((h - EM) // 2, top, bottom)
    return im.crop((0, pad, w, h - pad)) if pad > 0 else im


def png_glyph(path):
    """PNG(경로 또는 그림) → (TTGlyph, 전진 폭). 알파가 반 넘는 칸이 잉크. 줄마다 이어진 칸을 사각형 하나로."""
    im = (path if isinstance(path, Image.Image) else Image.open(path)).convert('RGBA')
    w, h = im.size
    px = im.load()
    diff = h - EM
    bottom = diff // 2 if diff % 2 == 0 else (diff + 1) // 2
    off_y = BASELINE - EM - bottom
    pen = TTGlyphPen(None)
    for r in range(h):
        y0 = (off_y + (h - 1 - r)) * U
        x = 0
        while x < w:
            if px[x, r][3] > 127:
                s = x
                while x < w and px[x, r][3] > 127:
                    x += 1
                x0, x1 = s * U, x * U
                pen.moveTo((x0, y0))
                pen.lineTo((x0, y0 + U))
                pen.lineTo((x1, y0 + U))
                pen.lineTo((x1, y0))
                pen.closePath()
            else:
                x += 1
    return pen.glyph(), w * U


def galmuri_glyph(gal, ch):
    cmap = gal.getBestCmap()
    if ord(ch) not in cmap:
        return None
    name = cmap[ord(ch)]
    pen = TTGlyphPen(None)
    gal.getGlyphSet()[name].draw(pen)
    return pen.glyph(), gal['hmtx'][name][0]


def build(ark, lang, cubic=None):
    static_name, flavors, out_name, suffix = TARGETS[lang]
    flavors = (flavors,) if isinstance(flavors, str) else flavors
    flavor = flavors[0]
    ark_sha = subprocess.run(['git', '-C', ark, 'rev-parse', 'HEAD'], capture_output=True, text=True).stdout.strip() or '?'
    chars = table_chars(static_name) | ui_symbol_chars() | {chr(c) for c in range(0x20, 0x7F)}
    if lang == 'ja':
        chars |= set(KANA)
    chars.discard('\n')
    idx = ark_index(ark)
    gal = TTFont(os.path.join(ROOT, 'assets', 'fonts', 'Galmuri11-Condensed.ttf'))
    patch_dir = os.path.join(ROOT, 'tools', 'fonts', 'patch', flavor)

    glyphs, metrics, cmap, order = {}, {}, {}, ['.notdef']
    g, adv = png_glyph(os.path.join(ark, 'assets/glyphs/12/notdef.png'))
    glyphs['.notdef'], metrics['.notdef'] = g, adv
    borrowed, missing, from_cubic = [], [], []
    for ch in sorted(chars):
        patch = os.path.join(patch_dir, '%04X.png' % ord(ch))
        p = patch if os.path.exists(patch) else ark_glyph(idx, ch, flavors)
        cub = None if p or lang not in CUBIC_FILL or not cubic or not is_han(ch) else cubic_image(cubic, ch)
        if p:
            g, adv = png_glyph(p)
        elif cub is not None:
            g, adv = png_glyph(cub)
            from_cubic.append(ch)
        else:
            got = galmuri_glyph(gal, ch)
            if got is None:
                missing.append(ch)
                continue
            g, adv = got
            borrowed.append(ch)
        name = 'uni%04X' % ord(ch)
        glyphs[name], metrics[name] = g, adv
        cmap[ord(ch)] = name
        order.append(name)
    if missing:
        sys.exit(f'{lang}: 글자가 없다 {"".join(missing)} — 문구를 바꾸거나 tools/fonts/patch/{flavor}/<16진수>.png를 찍는다')

    family = f'Nail CJK 12 {suffix}'
    fb = FontBuilder(UPEM, isTTF=True)
    fb.setupGlyphOrder(order)
    fb.setupCharacterMap(cmap)
    fb.setupGlyf(glyphs)
    glyf = fb.font['glyf']
    hmtx = {}
    for n in order:
        gl = glyf[n]
        gl.recalcBounds(glyf)
        hmtx[n] = (metrics[n], getattr(gl, 'xMin', 0) if gl.numberOfContours else 0)
    fb.setupHorizontalMetrics(hmtx)
    fb.setupHorizontalHeader(ascent=ASCENT, descent=DESCENT)
    fb.setupNameTable({
        'familyName': family,
        'styleName': 'Regular',
        'uniqueFontIdentifier': f'{family};{ark_sha[:12]}',
        'fullName': family,
        'psName': family.replace(' ', ''),
        'version': f'Version 1.0; ark-pixel-font {ark_sha[:12]}',
        'copyright': ('Copyright (c) 2021, TakWolf (https://takwolf.com) — Ark Pixel Font. '
                      'Symbols from Galmuri, Copyright (c) 2019-2025 Lee Minseo. '
                      + ('Some Han glyphs from Cubic 11 (俐方體11號), ACh-K, OFL 1.1. ' if from_cubic else '')
                      + 'Subset and rebuilt for Naïl: Under the Roots.'),
        'description': f'Subset of Ark Pixel Font 12px proportional ({flavor}) for one game. Not the original font.',
        'licenseDescription': 'This Font Software is licensed under the SIL Open Font License, Version 1.1.',
        'licenseInfoURL': 'https://openfontlicense.org',
    })
    fb.setupOS2(sTypoAscender=ASCENT, sTypoDescender=DESCENT, sTypoLineGap=0,
                usWinAscent=ASCENT + 200, usWinDescent=-DESCENT, fsType=0)
    fb.setupPost()
    out = os.path.join(ROOT, 'assets', 'fonts', out_name)
    fb.save(out)
    print(f'{out_name}: {len(order) - 1}자, {os.path.getsize(out) // 1024}KB, Galmuri에서 빌림 {"".join(borrowed)!r}, '
          f'Cubic 11에서 {len(from_cubic)}자, ark {ark_sha[:12]}')


def is_han(ch):
    return 0x3400 <= ord(ch) <= 0x9FFF or 0xF900 <= ord(ch) <= 0xFAFF or 0x20000 <= ord(ch) <= 0x2FFFF


if __name__ == '__main__':
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    cubic = (TTFont(sys.argv[2]), ImageFont.truetype(sys.argv[2], EM))
    for lang in TARGETS:
        build(sys.argv[1], lang, cubic)

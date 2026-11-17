#!/usr/bin/env python3
"""상점 영상(트레일러) 잇기: 장면 칸 + 게임이 낸 효과음(기록에서 다시 섞음) + 장면마다 고른 음악 → 1920×1080 MP4.

    python3 tools/store/trailer.py <찍은 폴더(tr)> <출력.mp4>

- 장면은 `tools/store/trailer_capture.js`로 찍는다(영어 화면, 칸 + audio.json + bufs/).
- 효과음: 게임이 낸 그대로(언제·어느 소리·크기·멈춤)를 칸 시각에 맞춰 다시 섞는다 — 타격감은 게임의 것.
  음악·환경음(웹은 mp3, 반복)은 기록에서 빼고, 장면마다 `SCENES`의 곡을 깐다(같은 곡이 이어지면 끊지 않고, 바뀌면 겹쳐 넘긴다).
- 그림은 게임 화면(960×540 = 320×180의 3배)을 가장 가까운 칸으로 2배(도트가 흐려지지 않게). 처음·끝 카드는 타이틀 그림 ×3 + 로고.
- 규격: Epic 권장(1920×1080 H.264, 30fps, AAC 스테레오 48kHz, ≤300MB) — docs/epic/store-text.md.
"""
import json
import subprocess
import sys
import tempfile
from pathlib import Path

import numpy as np
from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parent))
import epic_media  # noqa: E402  (로고·글자 찍기를 같이 쓴다)

ROOT = Path(__file__).resolve().parents[2]
MUSIC = ROOT / "assets" / "music"
SR = 48_000
FPS = 30

# (찍은 장면 폴더, 시작 초, 길이 초, 곡). 곡 이름은 assets/music/<이름>.ogg(src/music.rs Track::file과 같은 이름),
# "이름@초"면 그 초부터 튼다(머리가 조용한 곡).
SCENES = [
    ("hall", 0.3, 4.5, "roots_explorer@14"),
    ("kongi", 0.2, 1.6, "boss"),
    ("momo", 0.5, 4.0, "boss"),
    ("mumu", 0.0, 5.0, "side_keeper"),
    ("village", 1.0, 4.0, "village_springs"),
]
INTRO = (3.0, "title_light")   # 처음 카드: 길이, 곡
OUTRO = (4.0, "ending")        # 끝 카드
XFADE = 0.5                    # 곡이 바뀔 때 겹치는 초
# 게임이 실제로 낸 비율 그대로: 기본 설정에서 곡은 0.42까지 올린다(기록의 svol 최댓값, 효과음은 낸 크기 그대로).
# 다 섞은 뒤 loudnorm으로 전체 크기만 맞춘다(-16 LUFS).
MUSIC_GAIN = 0.42
SFX_GAIN = 1.0


def decode(src) -> np.ndarray:
    """소리(파일 경로 또는 바이트)를 48kHz 스테레오 float32 (n, 2)로."""
    args = ["ffmpeg", "-v", "error", "-i", "pipe:0" if isinstance(src, bytes) else str(src),
            "-f", "f32le", "-ac", "2", "-ar", str(SR), "pipe:1"]
    out = subprocess.run(args, input=src if isinstance(src, bytes) else None, capture_output=True, check=True).stdout
    return np.frombuffer(out, dtype=np.float32).reshape(-1, 2).copy()


def sfx_bus(scene: Path, start: float, dur: float) -> np.ndarray:
    """기록한 효과음을 [start, start+dur) 구간으로 다시 섞는다. 시각 0 = 첫 칸."""
    d = json.loads((scene / "audio.json").read_text())
    t0, fps = d["t0"], d["fps"]
    bufs, music_keys = {}, set()
    for f in (scene / "bufs").glob("*.bin"):
        b = f.read_bytes()
        if b[:4] == b"RIFF":
            bufs[int(f.stem)] = b
        else:
            music_keys.add(int(f.stem))  # mp3 = 음악·환경음
    cache = {}
    inst = {}          # 재생 번호 → {key, t, end, loop, gains:[(t, g)]}
    active = {}        # 소리 번호 → {재생 번호}
    for ev in d["events"]:
        kind, t = ev[0], (ev[1] - t0) / 1000.0 - 1.0 / fps  # 칸 n은 시계가 (n+1)/fps만큼 흐른 뒤 찍혔다
        if kind == "play":
            _, _, key, vol, loop, pb = ev
            if key in music_keys or key not in bufs:
                continue
            inst[pb] = {"key": key, "t": t, "end": None, "loop": bool(loop), "gains": [(t, vol)]}
            active.setdefault(key, set()).add(pb)
        elif kind == "svol":
            for pb in active.get(ev[2], ()):
                inst[pb]["gains"].append((t, ev[3]))
        elif kind == "pvol" and ev[2] in inst:
            inst[ev[2]]["gains"].append((t, ev[3]))
        elif kind == "stop":
            for pb in active.pop(ev[2], set()):
                if inst[pb]["end"] is None:
                    inst[pb]["end"] = t
        elif kind == "pstop" and ev[2] in inst:
            if inst[ev[2]]["end"] is None:
                inst[ev[2]]["end"] = t
    n = int(round(dur * SR))
    out = np.zeros((n, 2), np.float32)
    for p in inst.values():
        if p["key"] not in cache:
            cache[p["key"]] = decode(bufs[p["key"]])
        a = cache[p["key"]]
        if len(a) == 0:
            continue
        s0 = int(round((p["t"] - start) * SR))
        end_t = p["end"] if p["end"] is not None else (None if p["loop"] else p["t"] + len(a) / SR)
        s1 = n if end_t is None else int(round((end_t - start) * SR))
        s1 = min(s1, n)
        if s1 <= 0 or s0 >= n or s1 <= s0:
            continue
        length = s1 - s0
        sig = np.tile(a, (length // len(a) + 1, 1))[:length] if p["loop"] else a[:length]
        g = np.empty(len(sig), np.float32)
        gains = p["gains"]
        for i, (gt, gv) in enumerate(gains):
            a_i = max(0, int(round((gt - p["t"]) * SR)))
            b_i = len(sig) if i + 1 == len(gains) else max(0, int(round((gains[i + 1][0] - p["t"]) * SR)))
            g[a_i:b_i] = gv
        sig = sig * g[:, None]
        lo, hi = max(0, s0), min(n, s0 + len(sig))
        if hi > lo:
            out[lo:hi] += sig[lo - s0:hi - s0]
    return out * SFX_GAIN


def music_bus(blocks):
    """blocks = [(길이 초, 곡)] → 이어진 음악. 같은 곡이 이어지면 계속, 바뀌면 XFADE만큼 겹쳐 넘긴다."""
    total = int(round(sum(b[0] for b in blocks) * SR))
    out = np.zeros((total + int(XFADE * SR) * 2, 2), np.float32)
    tracks, pos, i = {}, 0, 0
    while i < len(blocks):
        name = blocks[i][1]
        j, length = i, 0.0
        while j < len(blocks) and blocks[j][1] == name:
            length += blocks[j][0]
            j += 1
        file, _, off = name.partition("@")  # "곡@초" = 그 초부터(조용한 머리를 건너뛴다)
        if name not in tracks:
            tracks[name] = decode(MUSIC / f"{file}.ogg")[int(float(off or 0) * SR):]
        a = tracks[name]
        n = int(round(length * SR))
        tail = int(XFADE * SR) if j < len(blocks) else 0
        seg = np.tile(a, ((n + tail) // len(a) + 1, 1))[: n + tail].copy()
        fi = int(XFADE * SR)
        if i > 0:
            seg[:fi] *= np.linspace(0, 1, fi, dtype=np.float32)[:, None]
        if tail:
            seg[-tail:] *= np.linspace(1, 0, tail, dtype=np.float32)[:, None]
        else:
            fo = int(1.2 * SR)  # 마지막 곡은 끝에서 천천히 내린다
            seg[-fo:] *= np.linspace(1, 0, fo, dtype=np.float32)[:, None]
        out[pos:pos + len(seg)] += seg
        pos += n
        i = j
    return out[:total] * MUSIC_GAIN


def card(path: Path, sub: str = ""):
    """타이틀 그림(640×360) ×3 = 1920×1080 + 로고(+ 아래 한 줄)."""
    base = Image.open(ROOT / "assets" / "art" / "illust" / "title.png").convert("RGBA")
    im = epic_media.up(base, 3)
    epic_media.place(im, epic_media.up(epic_media.wordmark(), 9), 960, 330)
    if sub:
        epic_media.place(im, epic_media.up(epic_media.text_layer(sub, 12, epic_media.CREAM, bold=False), 5), 960, 880)
    im.convert("RGB").save(path)


def run(*args):
    subprocess.run(args, check=True)


def main():
    tr, out = Path(sys.argv[1]), Path(sys.argv[2])
    tmp = Path(tempfile.mkdtemp(prefix="trailer_"))
    parts, blocks, sfx = [], [], []

    def still(img: Path, secs: float, name: str, fade_in=False, fade_out=False):
        f = []
        if fade_in:
            f.append("fade=t=in:st=0:d=0.6")
        if fade_out:
            f.append(f"fade=t=out:st={secs - 0.8}:d=0.8")
        vf = ",".join(f) if f else "null"
        p = tmp / f"{name}.mp4"
        run("ffmpeg", "-v", "error", "-y", "-loop", "1", "-framerate", str(FPS), "-i", str(img), "-t", str(secs),
            "-vf", vf, "-c:v", "libx264", "-preset", "slow", "-tune", "animation", "-crf", "16", "-pix_fmt", "yuv420p", str(p))
        parts.append(p)

    card(tmp / "intro.png")
    still(tmp / "intro.png", INTRO[0], "00_intro", fade_in=True)
    blocks.append(INTRO)
    sfx.append(np.zeros((int(round(INTRO[0] * SR)), 2), np.float32))

    for k, (name, start, dur, track) in enumerate(SCENES, 1):
        sc = tr / name
        f0, nf = int(round(start * FPS)), int(round(dur * FPS))
        p = tmp / f"{k:02d}_{name}.mp4"
        run("ffmpeg", "-v", "error", "-y", "-framerate", str(FPS), "-start_number", str(f0), "-i", str(sc / "f_%04d.png"),
            "-frames:v", str(nf), "-vf", "scale=1920:1080:flags=neighbor", "-c:v", "libx264", "-preset", "slow",
            "-tune", "animation", "-crf", "16", "-pix_fmt", "yuv420p", str(p))
        parts.append(p)
        blocks.append((nf / FPS, track))
        sfx.append(sfx_bus(sc, f0 / FPS, nf / FPS))

    card(tmp / "outro.png", "Out now")
    still(tmp / "outro.png", OUTRO[0], "99_outro", fade_out=True)
    blocks.append(OUTRO)
    sfx.append(np.zeros((int(round(OUTRO[0] * SR)), 2), np.float32))

    s = np.concatenate(sfx)
    m = music_bus(blocks)
    n = min(len(s), len(m))
    mix = s[:n] + m[:n]
    peak = float(np.abs(mix).max()) or 1.0
    mix = mix / max(1.0, peak / 0.89)  # 넘치면 낮춘다(-1dBFS)
    wav = tmp / "mix.f32"
    mix.astype(np.float32).tofile(wav)

    lst = tmp / "parts.txt"
    lst.write_text("".join(f"file '{p}'\n" for p in parts))
    run("ffmpeg", "-v", "error", "-y", "-f", "concat", "-safe", "0", "-i", str(lst),
        "-f", "f32le", "-ar", str(SR), "-ac", "2", "-i", str(wav),
        "-c:v", "copy", "-af", "loudnorm=I=-16:TP=-1.5:LRA=11", "-c:a", "aac", "-b:a", "192k", "-ar", str(SR), "-shortest", "-movflags", "+faststart", str(out))
    print(out, f"{n / SR:.1f}s", f"peak {peak:.2f}")


if __name__ == "__main__":
    main()

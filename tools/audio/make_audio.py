"""효과음·음악 파일 만들기(docs/decisions/0081). 산출물(assets/sfx, assets/music)을 커밋하고, 이 파일이 원본이다.

실행: python3 tools/audio/make_audio.py <Kenney 폴더> <rpgchip 폴더> <어쿠스틱 폴더>
  - Kenney(CC0): https://kenney.nl/assets/interface-sounds · impact-sounds · rpg-audio 의 zip을 한 폴더에 푼다.
  - rpgchip(CC0, Aureolus_Omicron): https://opengameart.org/content/15-melodic-rpg-chiptunes 의 zip을 푼다.
  - 어쿠스틱(CC0, OpenGameArt, 0154): 아래 `ACOUSTIC` 표의 페이지에서 받아 표의 파일 이름으로 한 폴더에 둔다.
  → assets/sfx/<이름>_<n>.wav, assets/music/<이름>.ogg(데스크톱·Android) + .mp3(웹)
  ffmpeg(libvorbis·libmp3lame)와 numpy가 있어야 한다.

- 효과음: 44.1kHz 모노 16비트 WAV(웹·네이티브 모두 그대로 푼다, quad-snd는 mp3를 못 푼다). 끝의 조용한 꼬리를
  자르고(봉우리 −40dB 아래) 끝 5ms를 줄인다. 봉우리를 −1dBFS로 맞춘다(Kenney ogg는 풀면 +2dB까지 넘는 것이 있어
  그대로 16비트로 쓰면 깨진다). 소리마다 크기는 코드(`sfx.rs`의 표)에서 곱한다.
- 음악: 칩튠은 사실상 모노(옆 성분 −36dB 아래)라 모노로 줄인다(크기 절반). 어쿠스틱 곡은 좌우 폭이 넓어(−1~−3dB)
  스테레오로 둔다(네이티브는 어차피 스테레오 f32로 풀어 메모리는 같다). 처음·끝 5ms를 줄여 고리 이음매의 딸깍을
  없앤다. 평균 크기를 −15dB RMS 근처로 맞추되 봉우리는 −1dBFS를 넘기지 않는다. ogg q4, mp3 V5.
- 표(SFX·MUSIC)를 고치면 다시 돌리고, sfx.rs·music.rs의 표와 assets/LICENSES.md를 함께 고친다.
"""
import glob
import os
import subprocess
import sys

import numpy as np

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SR = 44100

# (게임 이름, Kenney 파일 이름들, 최대 길이 초). 같은 이름의 여러 파일 = 무작위로 고르는 변형.
SFX = [
    ("ui_move", ["tick_004"], 0.08),
    ("ui_confirm", ["select_002"], 0.08),
    ("ui_back", ["back_002"], 0.12),
    ("buy", ["handleCoins2"], 0.40),
    ("deny", ["error_004"], 0.15),
    ("enemy_hit", ["impactPunch_medium_000", "impactPunch_medium_001", "impactPunch_medium_003"], 0.30),
    ("enemy_die", ["impactSoft_heavy_000", "impactSoft_heavy_002", "impactSoft_heavy_004"], 0.45),
    ("hurt", ["impactPunch_heavy_002", "impactPunch_heavy_004"], 0.40),
    ("chest", ["creak3"], 0.40),
]

# (게임 이름, 출처, 파일 이름). 출처 "chip" = rpgchip 폴더(모노), "acoustic" = 어쿠스틱 폴더(스테레오).
# music.rs의 TRACKS와 같은 이름. 쉬는 곳(타이틀·마을·쉼터)은 피아노·바이올린, 모험하는 곳은 칩튠(0154).
MUSIC = [
    ("title", "acoustic", "prologue_theme"),
    ("village", "acoustic", "lovely_town"),
    ("field", "chip", "rpgchip02_bittersweet_story"),
    ("camp", "acoustic", "snowfall_looped"),
    ("roots", "chip", "rpgchip06_dungeon"),
    ("deep", "chip", "rpgchip07_the_shrine_of_mysteries"),
    ("boss", "chip", "rpgchip14_battle_2"),
]

# 어쿠스틱 곡의 출처(모두 OpenGameArt 페이지 라이선스 칸 "CC0", 2026-10-27 받음). 파일 이름 ← 받은 파일.
ACOUSTIC = {
    "prologue_theme": "https://opengameart.org/content/prologue-theme (Cleyton Kauffman, zip 안 Prologue Theme.wav)",
    "lovely_town": "https://opengameart.org/content/lovely-town (fluffclipse, Lovely town_0.ogg)",
    "snowfall_looped": "https://opengameart.org/content/snowfall (Kistol, Snowfall (Looped ver.)_0.ogg)",
}

PEAK = 10 ** (-1 / 20)
ACOUSTIC_RMS = -19


def decode(path, channels=1):
    raw = subprocess.run(
        ["ffmpeg", "-v", "error", "-i", path, "-f", "f32le", "-ac", str(channels), "-ar", str(SR), "-"],
        capture_output=True, check=True,
    ).stdout
    a = np.frombuffer(raw, dtype=np.float32).astype(np.float64)
    return a.reshape(-1, channels) if channels > 1 else a


def find(folder, name):
    hits = [
        p
        for ext in ("ogg", "wav", "flac", "mp3")
        for p in glob.glob(os.path.join(folder, "**", f"{name}.{ext}"), recursive=True)
    ]
    if not hits:
        sys.exit(f"없다: {name}.ogg|wav|flac|mp3 ({folder})")
    return sorted(hits)[0]


def fade_edges(a, ms_in, ms_out):
    a = a.copy()
    n_in, n_out = int(SR * ms_in / 1000), int(SR * ms_out / 1000)
    shape = (-1, 1) if a.ndim > 1 else (-1,)
    if n_in:
        a[:n_in] *= np.linspace(0.0, 1.0, n_in).reshape(shape)
    if n_out:
        a[-n_out:] *= np.linspace(1.0, 0.0, n_out).reshape(shape)
    return a


def write_wav(path, a):
    pcm = (np.clip(a, -1.0, 1.0) * 32767).astype("<i2")
    subprocess.run(
        ["ffmpeg", "-v", "error", "-y", "-f", "s16le", "-ar", str(SR), "-ac", "1", "-i", "-",
         "-map_metadata", "-1", "-fflags", "+bitexact", "-flags:a", "+bitexact", path],
        input=pcm.tobytes(), check=True,
    )


def make_sfx(kenney):
    out = os.path.join(ROOT, "assets", "sfx")
    os.makedirs(out, exist_ok=True)
    for old in glob.glob(os.path.join(out, "*.wav")):
        os.remove(old)
    for name, files, max_s in SFX:
        for i, f in enumerate(files):
            a = decode(find(kenney, f))
            # 소리가 시작하기 전 빈 곳과 조용한 꼬리를 자른다.
            loud = np.nonzero(np.abs(a) > np.abs(a).max() * 0.01)[0]
            a = a[max(loud[0] - 22, 0):loud[-1] + 1][: int(SR * max_s)]
            a = fade_edges(a, 0, 5) * (PEAK / np.abs(a).max())
            path = os.path.join(out, f"{name}_{i}.wav")
            write_wav(path, a)
            print(f"{os.path.relpath(path, ROOT)}  {len(a) / SR * 1000:.0f}ms  ← {f}")


def limit(a, block=64, attack_db=1.0, release_db=0.03):
    """미리 보는 봉우리 리미터: 64샘플(1.5ms) 칸마다 −1dBFS를 넘지 않을 이득을 구하고, 내려갈 때는 칸마다 1dB씩
    미리(≈6ms 앞부터), 올라올 때는 칸마다 0.03dB씩(4dB를 ≈190ms에) 움직인다. 칸 가운데 사이를 직선으로 이어
    어느 샘플도 −1dBFS를 넘지 않는다(이웃 칸의 최솟값을 쓰기 때문)."""
    n = len(a)
    nb = -(-n // block)
    peak = np.abs(a).reshape(n, -1).max(axis=1)
    peak = np.pad(peak, (0, nb * block - n)).reshape(nb, block).max(axis=1)
    need = np.minimum(0.0, 20 * np.log10(PEAK / np.maximum(peak, 1e-9)))
    g = np.minimum(need, np.minimum(np.r_[0.0, need[:-1]], np.r_[need[1:], 0.0]))
    for i in range(nb - 2, -1, -1):
        g[i] = min(g[i], g[i + 1] + attack_db)
    for i in range(1, nb):
        g[i] = min(g[i], g[i - 1] + release_db)
    centers = np.arange(nb) * block + block / 2
    gain = 10 ** (np.interp(np.arange(n), centers, g) / 20)
    print(f"  리미터: 가장 깊게 {g.min():.1f}dB, 1dB 넘게 누른 시간 {(g < -1).mean() * 100:.2f}%")
    return a * (gain.reshape(-1, 1) if a.ndim > 1 else gain)


def make_music(chip, acoustic):
    out = os.path.join(ROOT, "assets", "music")
    os.makedirs(out, exist_ok=True)
    for name, src, f in MUSIC:
        ch = 2 if src == "acoustic" else 1
        a = decode(find(acoustic if src == "acoustic" else chip, f), ch)
        # 크기는 가운데(좌우 평균)로 잰다: 스테레오도 모노 곡과 같은 −15dB RMS 근처에.
        mid = a.mean(axis=1) if ch > 1 else a
        rms = np.sqrt((mid ** 2).mean())
        if src == "acoustic":
            # 어쿠스틱 곡은 봉우리가 드물게 튀어 봉우리로만 맞추면 칩튠보다 5dB까지 작다. 칩튠 크기(−19dB RMS)로
            # 올리고 넘는 봉우리만 리미터로 누른다.
            a = limit(a * (10 ** (ACOUSTIC_RMS / 20) / rms))
        else:
            a = a * min(10 ** (-15 / 20) / rms, PEAK / np.abs(a).max())
        a = fade_edges(a, 5, 5)
        pcm = (np.clip(a, -1.0, 1.0) * 32767).astype("<i2").tobytes()
        for ext, args in [("ogg", ["-c:a", "libvorbis", "-q:a", "4"]), ("mp3", ["-c:a", "libmp3lame", "-q:a", "5"])]:
            path = os.path.join(out, f"{name}.{ext}")
            subprocess.run(
                ["ffmpeg", "-v", "error", "-y", "-f", "s16le", "-ar", str(SR), "-ac", str(ch), "-i", "-",
                 "-map_metadata", "-1", *args, path],
                input=pcm, check=True,
            )
            print(f"{os.path.relpath(path, ROOT)}  {len(a) / SR:.1f}s  {os.path.getsize(path) // 1024}KB  ← {f}")


if __name__ == "__main__":
    if len(sys.argv) != 4:
        sys.exit(__doc__)
    make_sfx(sys.argv[1])
    make_music(sys.argv[2], sys.argv[3])

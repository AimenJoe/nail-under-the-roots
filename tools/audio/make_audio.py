"""효과음·음악 파일 만들기(docs/decisions/0081). 산출물(assets/sfx, assets/music)을 커밋하고, 이 파일이 원본이다.

실행: python3 tools/audio/make_audio.py <Kenney 폴더> <rpgchip 폴더>
  - Kenney(CC0): https://kenney.nl/assets/interface-sounds · impact-sounds · rpg-audio 의 zip을 한 폴더에 푼다.
  - rpgchip(CC0, Aureolus_Omicron): https://opengameart.org/content/15-melodic-rpg-chiptunes 의 zip을 푼다.
  → assets/sfx/<이름>_<n>.wav, assets/music/<이름>.ogg(데스크톱·Android) + .mp3(웹)
  ffmpeg(libvorbis·libmp3lame)와 numpy가 있어야 한다.

- 효과음: 44.1kHz 모노 16비트 WAV(웹·네이티브 모두 그대로 푼다, quad-snd는 mp3를 못 푼다). 끝의 조용한 꼬리를
  자르고(봉우리 −40dB 아래) 끝 5ms를 줄인다. 봉우리를 −1dBFS로 맞춘다(Kenney ogg는 풀면 +2dB까지 넘는 것이 있어
  그대로 16비트로 쓰면 깨진다). 소리마다 크기는 코드(`sfx.rs`의 표)에서 곱한다.
- 음악: 사실상 모노(옆 성분 −36dB 아래)라 모노로 줄인다(크기 절반). 처음·끝 5ms를 줄여 고리 이음매의 딸깍을
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

# (게임 이름, rpgchip 파일 이름). music.rs의 TRACKS와 같은 이름.
MUSIC = [
    ("title", "rpgchip01_title_screen"),
    ("village", "rpgchip03_town"),
    ("field", "rpgchip02_bittersweet_story"),
    ("camp", "rpgchip05_pavane"),
    ("roots", "rpgchip06_dungeon"),
    ("deep", "rpgchip07_the_shrine_of_mysteries"),
    ("boss", "rpgchip14_battle_2"),
]

PEAK = 10 ** (-1 / 20)


def decode(path):
    raw = subprocess.run(
        ["ffmpeg", "-v", "error", "-i", path, "-f", "f32le", "-ac", "1", "-ar", str(SR), "-"],
        capture_output=True, check=True,
    ).stdout
    return np.frombuffer(raw, dtype=np.float32).astype(np.float64)


def find(folder, name):
    hits = [p for p in glob.glob(os.path.join(folder, "**", name + ".ogg"), recursive=True)]
    if not hits:
        sys.exit(f"없다: {name}.ogg ({folder})")
    return sorted(hits)[0]


def fade_edges(a, ms_in, ms_out):
    a = a.copy()
    n_in, n_out = int(SR * ms_in / 1000), int(SR * ms_out / 1000)
    if n_in:
        a[:n_in] *= np.linspace(0.0, 1.0, n_in)
    if n_out:
        a[-n_out:] *= np.linspace(1.0, 0.0, n_out)
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


def make_music(chip):
    out = os.path.join(ROOT, "assets", "music")
    os.makedirs(out, exist_ok=True)
    for name, f in MUSIC:
        a = decode(find(chip, f))
        rms = np.sqrt((a ** 2).mean())
        gain = min(10 ** (-15 / 20) / rms, PEAK / np.abs(a).max())
        a = fade_edges(a, 5, 5) * gain
        pcm = (np.clip(a, -1.0, 1.0) * 32767).astype("<i2").tobytes()
        for ext, args in [("ogg", ["-c:a", "libvorbis", "-q:a", "4"]), ("mp3", ["-c:a", "libmp3lame", "-q:a", "5"])]:
            path = os.path.join(out, f"{name}.{ext}")
            subprocess.run(
                ["ffmpeg", "-v", "error", "-y", "-f", "s16le", "-ar", str(SR), "-ac", "1", "-i", "-",
                 "-map_metadata", "-1", *args, path],
                input=pcm, check=True,
            )
            print(f"{os.path.relpath(path, ROOT)}  {len(a) / SR:.1f}s  {os.path.getsize(path) // 1024}KB  ← {f}")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    make_sfx(sys.argv[1])
    make_music(sys.argv[2])

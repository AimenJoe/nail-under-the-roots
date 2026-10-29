//! 음악(docs/decisions/0081): 장소마다 곡 하나, 바뀌면 교차, 대사·메뉴에서는 낮춘다.
//!
//! - 곡 파일은 `assets/music/<이름>.ogg`(데스크톱·Android) / `.mp3`(웹). 웹은 브라우저가 풀고 Safari의 Ogg Vorbis
//!   지원이 버전마다 다르다. 네이티브(quad-snd·audrey)는 mp3를 못 푼다(wav·ogg만) → 둘로 나눠 싣는다.
//! - 네이티브는 곡을 통째로 44.1kHz f32 스테레오로 풀어 둔다(1분 ≈ 21MB). 고리는 짧게, 원본은 44.1kHz로
//!   (다른 샘플레이트는 가장 가까운 점 리샘플이라 거칠어진다).
//! - 곡 파일 이름은 `Track::file` 한 곳(빠뜨리면 컴파일이 안 된다). 시험이 모든 곡의 두 파일을 본다.
//! - 곡은 들어야 할 때 읽고(코루틴), 다 내려가면 놓는다. 네이티브는 읽는 그 프레임에 통째로 풀어 멈칫한다
//!   (이 PC에서 1분 50초 곡이 약 0.2초) → 장소가 바뀌는 검은 화면에서 읽히게 된다. 보스 곡은 미리(`KEEP`).
//! - 소리 크기는 설정의 "음악"(효과음과 따로).

use macroquad::audio::{
    load_sound_from_bytes, play_sound, set_sound_volume, stop_sound, PlaySoundParams, Sound,
};
use macroquad::experimental::coroutines::{start_coroutine, Coroutine};

/// 어디의 곡인가. 같은 곡이 여러 장소에 걸릴 수 있다(`App::music_track`). 장소마다 옛 곡과 사장이 고른 새 곡(0234)을
/// 둘 다 쓴다 — 어떤 때 어느 곡인지는 `App::music_track` 한 곳.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Track {
    /// 옛 곡(0154): 서곡 · 마을 · 필드 칩튠 · 눈 오는 쉼터 · 굴 칩튠 · 신비의 사당 · 싸움 2.
    Title,
    Village,
    Field,
    Camp,
    Roots,
    Deep,
    Boss,
    /// 새 곡(0234).
    TitleLight,
    VillageSprings,
    FieldOverworld,
    CampCalm,
    RootsExplorer,
    DeepPool,
    Silk,
    /// 보스 2단계(0233): 깨어나면 이 곡으로 넘어간다.
    BossRise,
    /// 가장 깊은 곳(할매와 매듭).
    Final,
    Ending,
    /// 환경음(음악 밑에 깔리는 고리, `Music::ambience`).
    AmbMeadow,
    AmbCave,
    AmbDeep,
}

impl Track {
    /// 파일 이름(`assets/music/<이름>.ogg|mp3`). 파일은 `tools/audio/make_audio.py`가 만든다(출처는 assets/LICENSES.md).
    /// `_` 없이 모두 적는다 — 곡을 더하면 컴파일러가 여기를 묻는다(0235 R19, 예전 `TRACKS` 표는 빠뜨리면 그 장소가 조용히 무음이었다).
    pub fn file(self) -> &'static str {
        match self {
            Track::Title => "title",
            Track::Village => "village",
            Track::Field => "field",
            Track::Camp => "camp",
            Track::Roots => "roots",
            Track::Deep => "deep",
            Track::Boss => "boss",
            Track::TitleLight => "title_light",
            Track::VillageSprings => "village_springs",
            Track::FieldOverworld => "field_overworld",
            Track::CampCalm => "camp_calm",
            Track::RootsExplorer => "roots_explorer",
            Track::DeepPool => "deep_pool",
            Track::Silk => "silk",
            Track::BossRise => "boss_rise",
            Track::Final => "final",
            Track::Ending => "ending",
            Track::AmbMeadow => "amb_meadow",
            Track::AmbCave => "amb_cave",
            Track::AmbDeep => "amb_deep",
        }
    }

    /// 타이틀 쪽 곡(이때는 보스 곡을 미리 읽지 않는다).
    fn is_title(self) -> bool {
        matches!(self, Track::Title | Track::TitleLight)
    }
}

/// 모든 곡(시험·패키징이 파일 목록을 만들 때). `file`의 match와 같은 수인지 시험이 본다.
#[cfg(test)]
const ALL: [Track; 20] = [
    Track::Title,
    Track::Village,
    Track::Field,
    Track::Camp,
    Track::Roots,
    Track::Deep,
    Track::Boss,
    Track::TitleLight,
    Track::VillageSprings,
    Track::FieldOverworld,
    Track::CampCalm,
    Track::RootsExplorer,
    Track::DeepPool,
    Track::Silk,
    Track::BossRise,
    Track::Final,
    Track::Ending,
    Track::AmbMeadow,
    Track::AmbCave,
    Track::AmbDeep,
];

/// 다 내려간 뒤에도 놓지 않는 곡. 보스 곡은 싸움 한가운데서 갑자기 필요하다: 그때 풀면(네이티브는 그 자리에서
/// 통째로 푼다) 화면이 멈춘다 → 처음 게임 장면에 들어설 때(화면이 검을 때) 미리 읽고 쥐고 있는다.
/// 2단계 곡도 싸움 한가운데서 넘어가니 같이 쥔다(0233·0234).
const KEEP: &[Track] = &[Track::Boss, Track::BossRise];

/// 교차 시간(초): 새 곡이 올라오고 옛 곡이 내려가는 데 걸리는 시간.
const FADE: f32 = 1.0;
/// 대사·메뉴에서 음악을 이만큼으로(약 −6dB). 귀로 맞춘다(0081 조사).
const DUCK: f32 = 0.5;
/// 음악 전체 크기(효과음 MASTER_VOLUME과 따로 맞춘다).
const MASTER: f32 = 0.6;
/// 환경음 전체 크기(설정의 "효과음"에 곱한다). 음악 밑에 깔릴 만큼 — 귀로 맞춘다.
const AMB_MASTER: f32 = 0.35;

enum Slot {
    Loading(Coroutine<Option<Sound>>),
    Ready(Sound),
    /// 읽거나 풀 수 없었다. 다시 찾지 않는다(콘솔에 한 번만 남는다).
    Failed,
}

pub struct Music {
    /// 읽는 중이거나 읽은 곡. 들을 때 읽고, 다 내려가면 놓는다(네이티브는 1분 ≈ 21MB라 다 쥐고 있지 않는다).
    slots: Vec<(Track, Slot)>,
    /// 지금 곡과 올라온 정도(0..1).
    now: Option<(Track, f32)>,
    /// 내려가는 옛 곡들.
    old: Vec<(Track, f32)>,
    /// 설정의 음악 크기(0..1).
    volume: f32,
    /// 지금 낮춘 정도(1 = 그대로, DUCK = 낮춤). 부드럽게 따라간다.
    duck: f32,
    /// 전체 크기(음악 `MASTER`, 환경음 `AMB_MASTER`).
    master: f32,
    /// 다 내려가도 쥐고 있는 곡(음악은 `KEEP`, 환경음은 없음).
    keep: &'static [Track],
}

impl Music {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            now: None,
            old: Vec::new(),
            volume: 0.7,
            duck: 1.0,
            master: MASTER,
            keep: KEEP,
        }
    }

    /// 환경음 층(0234): 같은 교차·낮추기로 음악 밑에 고리 하나. 크기는 설정의 효과음을 따른다.
    pub fn ambience() -> Self {
        Self {
            master: AMB_MASTER,
            keep: &[],
            ..Self::new()
        }
    }

    pub fn set_volume(&mut self, v: f32) {
        self.volume = v.clamp(0.0, 1.0);
    }

    /// 아직 안 읽은 곡이면 읽기 시작한다(웹은 내려받고 브라우저가 푸는 동안 게임이 돈다).
    fn request(&mut self, t: Track) {
        if self.slots.iter().any(|(x, _)| *x == t) {
            return;
        }
        let name = t.file();
        let ext = if cfg!(target_arch = "wasm32") {
            "mp3"
        } else {
            "ogg"
        };
        let path = format!("music/{name}.{ext}");
        let co = start_coroutine(async move {
            match macroquad::file::load_file(&path).await {
                Ok(bytes) => match load_sound_from_bytes(&bytes).await {
                    Ok(s) => Some(s),
                    Err(e) => {
                        eprintln!("음악 {path}: 풀 수 없다({e})");
                        None
                    }
                },
                Err(e) => {
                    eprintln!("음악 {path}: 읽을 수 없다({e})");
                    None
                }
            }
        });
        self.slots.push((t, Slot::Loading(co)));
    }

    fn sound(&self, t: Track) -> Option<&Sound> {
        self.slots.iter().find_map(|(x, s)| match s {
            Slot::Ready(s) if *x == t => Some(s),
            _ => None,
        })
    }

    /// 프레임마다: 들려야 할 곡(`None` = 조용히), 낮출지.
    pub fn update(&mut self, want: Option<Track>, ducked: bool, dt: f32) {
        if let Some(t) = want {
            self.request(t);
            if !t.is_title() {
                for k in self.keep {
                    self.request(*k);
                }
            }
        }
        for (_, slot) in self.slots.iter_mut() {
            if let Slot::Loading(co) = slot {
                if co.is_done() {
                    *slot = match co.retrieve().flatten() {
                        Some(s) => Slot::Ready(s),
                        None => Slot::Failed,
                    };
                }
            }
        }
        // 곡이 없는 장소는 음악을 내린다. 아직 읽는 중이거나 파일이 없는 곡도 "없음"으로 본다
        // (읽는 동안 옛 곡이 내려가고, 다 읽으면 올라온다).
        let want = want.filter(|t| self.sound(*t).is_some());
        if self.now.map(|(t, _)| t) != want {
            if let Some(n) = self.now.take() {
                self.old.retain(|(t, _)| *t != n.0);
                self.old.push(n);
            }
            if let Some(t) = want {
                // 내려가던 같은 곡이면 그 자리에서 다시 올린다(처음부터 다시 틀지 않는다).
                let level = match self.old.iter().position(|(o, _)| *o == t) {
                    Some(i) => self.old.remove(i).1,
                    None => {
                        if let Some(s) = self.sound(t) {
                            play_sound(
                                s,
                                PlaySoundParams {
                                    looped: true,
                                    volume: 0.0,
                                },
                            );
                        }
                        0.0
                    }
                };
                self.now = Some((t, level));
            }
        }
        let step = dt / FADE;
        let target = if ducked { DUCK } else { 1.0 };
        self.duck += (target - self.duck) * (dt * 6.0).min(1.0);
        let gain = self.master * self.volume * self.duck;
        if let Some((t, level)) = self.now.as_mut() {
            *level = (*level + step).min(1.0);
            let (t, level) = (*t, *level);
            if let Some(s) = self.sound(t) {
                set_sound_volume(s, gain * level);
            }
        }
        for (_, level) in self.old.iter_mut() {
            *level -= step;
        }
        for (t, level) in &self.old {
            if let Some(s) = self.sound(*t) {
                if *level <= 0.0 {
                    stop_sound(s);
                } else {
                    set_sound_volume(s, gain * level);
                }
            }
        }
        self.old.retain(|(_, level)| *level > 0.0);
        // 울리지 않는 곡은 놓는다(Sound를 버리면 소리 데이터도 지워진다): 다 내려간 곡, 다 읽었을 때는 이미
        // 다른 곳으로 간 곡. 지금 곡·내려가는 곡·KEEP은 남긴다.
        let now = self.now.map(|(t, _)| t);
        let fading: Vec<Track> = self.old.iter().map(|(t, _)| *t).collect();
        self.slots.retain(|(t, s)| {
            !matches!(s, Slot::Ready(_))
                || Some(*t) == now
                || fading.contains(t)
                || self.keep.contains(t)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 표의 곡마다 두 파일(웹 mp3·네이티브 ogg)이 있다 — 없으면 웹 콘솔에 404, 네이티브는 조용히 빈다.
    #[test]
    fn every_track_has_both_files() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/music");
        for t in ALL {
            let name = t.file();
            for ext in ["mp3", "ogg"] {
                let p = format!("{dir}/{name}.{ext}");
                assert!(std::path::Path::new(&p).exists(), "{t:?}: {p}");
            }
        }
    }
}

//! 음악(docs/decisions/0081): 장소마다 곡 하나, 바뀌면 교차, 대사·메뉴에서는 낮춘다.
//!
//! - 곡 파일은 `assets/music/<이름>.ogg`(데스크톱·Android) / `.mp3`(웹). 웹은 브라우저가 풀고 Safari의 Ogg Vorbis
//!   지원이 버전마다 다르다. 네이티브(quad-snd·audrey)는 mp3를 못 푼다(wav·ogg만) → 둘로 나눠 싣는다.
//! - 네이티브는 곡을 통째로 44.1kHz f32 스테레오로 풀어 둔다(1분 ≈ 21MB). 고리는 짧게, 원본은 44.1kHz로
//!   (다른 샘플레이트는 가장 가까운 점 리샘플이라 거칠어진다).
//! - `TRACKS`에 있는 것만 읽는다. 없는 파일을 찾아 웹 콘솔에 404를 남기지 않게. 곡을 넣으면 여기에 한 줄.
//! - 소리 크기는 설정의 "음악"(효과음과 따로).

use macroquad::audio::{
    load_sound_from_bytes, play_sound, set_sound_volume, stop_sound, PlaySoundParams, Sound,
};

/// 어디의 곡인가. 같은 곡이 여러 장소에 걸릴 수 있다(`App::music_track`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Track {
    Title,
    Village,
    Field,
    Camp,
    Roots,
    Deep,
    Boss,
}

/// (곡, 파일 이름). 파일이 생기면 여기에 넣는다. 지금은 비어 있다(출처를 받기 전, 0081).
const TRACKS: &[(Track, &str)] = &[];

/// 교차 시간(초): 새 곡이 올라오고 옛 곡이 내려가는 데 걸리는 시간.
const FADE: f32 = 1.0;
/// 대사·메뉴에서 음악을 이만큼으로(약 −6dB). 귀로 맞춘다(0081 조사).
const DUCK: f32 = 0.5;
/// 음악 전체 크기(효과음 MASTER_VOLUME과 따로 맞춘다).
const MASTER: f32 = 0.6;

pub struct Music {
    tracks: Vec<(Track, Sound)>,
    /// 지금 곡과 올라온 정도(0..1).
    now: Option<(Track, f32)>,
    /// 내려가는 옛 곡들.
    old: Vec<(Track, f32)>,
    /// 설정의 음악 크기(0..1).
    volume: f32,
    /// 지금 낮춘 정도(1 = 그대로, DUCK = 낮춤). 부드럽게 따라간다.
    duck: f32,
}

impl Music {
    pub async fn load() -> Self {
        let ext = if cfg!(target_arch = "wasm32") {
            "mp3"
        } else {
            "ogg"
        };
        let mut tracks = Vec::new();
        for (t, name) in TRACKS {
            let path = format!("music/{name}.{ext}");
            match macroquad::file::load_file(&path).await {
                Ok(bytes) => match load_sound_from_bytes(&bytes).await {
                    Ok(s) => tracks.push((*t, s)),
                    Err(e) => eprintln!("음악 {path}: 풀 수 없다({e})"),
                },
                Err(e) => eprintln!("음악 {path}: 읽을 수 없다({e})"),
            }
        }
        Self {
            tracks,
            now: None,
            old: Vec::new(),
            volume: 0.7,
            duck: 1.0,
        }
    }

    pub fn set_volume(&mut self, v: f32) {
        self.volume = v.clamp(0.0, 1.0);
    }

    fn sound(&self, t: Track) -> Option<&Sound> {
        self.tracks.iter().find(|(x, _)| *x == t).map(|(_, s)| s)
    }

    /// 프레임마다: 들려야 할 곡(`None` = 조용히), 낮출지.
    pub fn update(&mut self, want: Option<Track>, ducked: bool, dt: f32) {
        // 곡이 없는 장소는 음악을 내린다. 파일이 없는 곡도 "없음"으로 본다.
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
        let gain = MASTER * self.volume * self.duck;
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
    }
}

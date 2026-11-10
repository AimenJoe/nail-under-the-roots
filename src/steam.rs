//! Steam 연결(0276 S1). `--features steam`일 때만 Steamworks를 부르고, 아니면(웹·Android·iOS·직접 받은 PC 판) 모두 빈 손이다.
//! Steam이 꺼져 있거나 앱 번호를 모르면 조용히 연결 없이 켜진다 — 게임은 Steam에 기대지 않는다.
//!
//! 앱 번호: Steam으로 켜면 Steam이 알려 준다. 개발 중에는 실행 파일 옆 `steam_appid.txt`(레포에 넣지 않는다).
//! 업적은 `achievements.rs` 표 — 기록이 바뀔 때마다 `sync`가 새로 이룬 것만 연다.
//! 기록 폴더: 연결되면 Steam 계정마다 `<앱 폴더>/steam-<계정 번호>`(`profile::scope_saves`, 0276 S3) — 그래서 `start`는
//! 기록을 처음 읽기 전에 부른다(`App::new` 맨 앞). Auto-Cloud 경로의 `{Steam3AccountID}`와 같은 번호다.

use crate::profile::Profile;

pub struct Steam {
    #[cfg(feature = "steam")]
    client: Option<steamworks::Client>,
    /// 이번 실행에서 이미 연 업적(같은 것을 매번 다시 부르지 않게).
    opened: Vec<&'static str>,
    /// 켠 뒤 첫 프레임에 옛 기록으로 이룬 업적을 한꺼번에 열었나.
    caught_up: bool,
}

impl Steam {
    pub fn start() -> Self {
        Self {
            #[cfg(feature = "steam")]
            client: match steamworks::Client::init() {
                Ok(c) => {
                    let id = c.user().steam_id().account_id().raw();
                    crate::profile::scope_saves(&format!("steam-{id}"));
                    Some(c)
                }
                Err(e) => {
                    eprintln!("steam: 연결 없이 켠다 ({e})");
                    None
                }
            },
            opened: Vec::new(),
            caught_up: false,
        }
    }

    /// 매 프레임: Steam 콜백을 돌린다. 첫 프레임에는 옛 기록으로 이미 이룬 업적도 연다.
    pub fn tick(&mut self, p: &Profile) {
        #[cfg(feature = "steam")]
        if let Some(c) = &self.client {
            c.run_callbacks();
        }
        if !self.caught_up {
            self.caught_up = true;
            self.sync(p);
        }
    }

    /// 기록이 바뀐 뒤: 새로 이룬 업적을 열고 한 번에 올린다.
    pub fn sync(&mut self, p: &Profile) {
        let fresh: Vec<&'static str> = crate::achievements::fresh(p, &self.opened)
            .map(|x| x.api)
            .collect();
        if fresh.is_empty() {
            return;
        }
        #[cfg(feature = "steam")]
        if let Some(c) = &self.client {
            let stats = c.user_stats();
            for api in &fresh {
                let _ = stats.achievement(api).set();
            }
            let _ = stats.store_stats();
        }
        self.opened.extend(fresh);
    }
}

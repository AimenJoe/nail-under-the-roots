//! 장면 전환이 끝난 뒤에 할 일(0230 R15): 깨어난 뒤 한마디, 쉼터 도착 한마디, 런 첫 한마디, 길 지도 열기.
//!
//! 예전에는 `App`의 깃발 넷이었고 순서가 서로의 `if` 조건에 숨어 있었다. 여기서는 할 일을 담아 두고,
//! 문(`Gate`: 페이드·살아 있음·게임 장면·컷씬)이 열릴 때 정해진 순서(`ORDER`)로 하나씩 꺼낸다.
//! 소리·그림·월드를 모르는 순수한 상태라 단위 시험으로 돈다 — 실제로 하는 일(`Do`)은 앱이 한다.

/// 할 일의 종류. `ORDER` 순서로 본다.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Wake,
    Camp,
    RunStart,
    Route,
}

/// 한 프레임에 이 순서로 한 번씩 본다(앞의 일이 컷씬을 열면 뒤의 일은 다음 프레임 이후로 밀린다).
pub const ORDER: [Kind; 4] = [Kind::Wake, Kind::Camp, Kind::RunStart, Kind::Route];

/// 꺼낸 일을 앱이 한다.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Do {
    /// 깨어나는 어두워짐·밝아짐이 끝났다: 이웃의 한마디(0037).
    SayWake,
    /// 쉼터 도착 한마디(쉼터 번호, 0225).
    CampArrive(u32),
    /// 런 첫 한마디(마을에서 곧장 내려간 런, 0225).
    SayRunStart,
    /// 길 지도를 연다(마을에서 내려온 런의 첫 쉼터, 0226).
    OpenRoute,
}

/// 지금 할 일을 꺼내도 되는가. 앱이 매번 새로 잰다(앞의 일이 컷씬을 열 수 있다).
#[derive(Clone, Copy, Debug)]
pub struct Gate {
    /// 페이드(깨어나기·내려가는 단면)가 끝났다.
    pub settled: bool,
    pub alive: bool,
    /// 장면이 게임(메뉴·컷씬 장면이 아님).
    pub in_game: bool,
    /// 컷씬이 돌고 있다.
    pub cutscene: bool,
}

#[derive(Default, Debug)]
pub struct After {
    wake: bool,
    camp: Option<u32>,
    run_start: bool,
    route: bool,
    /// 마을에서 내려온 런(0226): 첫 쉼터에 닿으면 도착 한마디 대신 길 지도.
    village_run: bool,
}

impl After {
    /// 새 런·장면: 남은 일을 다 버린다.
    pub fn clear(&mut self) {
        *self = After::default();
    }

    /// 깨어난 뒤 한마디를 할지(기록이 남는 런만).
    pub fn set_wake(&mut self, on: bool) {
        self.wake = on;
    }

    /// 쉼터에 닿았다(도착 한마디는 내려가는 단면이 다 흐른 뒤에).
    pub fn camp(&mut self, n: u32) {
        self.camp = Some(n);
    }

    /// 마을 뿌리 길에서 첫 쉼터로 곧장 내려가는 런: 닿으면 런 첫 한마디, 그다음 길 지도.
    pub fn village_run(&mut self) {
        self.run_start = true;
        self.village_run = true;
    }

    /// `kind`가 남아 있고 문이 열렸으면 꺼낸다.
    pub fn take(&mut self, kind: Kind, g: Gate) -> Option<Do> {
        let calm = g.settled && g.alive && g.in_game;
        match kind {
            Kind::Wake if self.wake && calm => {
                self.wake = false;
                Some(Do::SayWake)
            }
            Kind::Camp if calm => {
                let n = self.camp.take()?;
                if std::mem::take(&mut self.village_run) {
                    self.route = true;
                    None
                } else {
                    Some(Do::CampArrive(n))
                }
            }
            Kind::RunStart if self.run_start && self.camp.is_none() && calm => {
                self.run_start = false;
                Some(Do::SayRunStart)
            }
            Kind::Route
                if self.route && !self.run_start && g.settled && g.in_game && !g.cutscene =>
            {
                self.route = false;
                Some(Do::OpenRoute)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OPEN: Gate = Gate {
        settled: true,
        alive: true,
        in_game: true,
        cutscene: false,
    };
    const FADING: Gate = Gate {
        settled: false,
        ..OPEN
    };

    /// 한 프레임: 앱처럼 `ORDER`대로 한 번씩. `talks`면 꺼낸 한마디가 컷씬을 연다.
    fn frame(a: &mut After, g: Gate, talks: bool) -> Vec<Do> {
        let mut g = g;
        let mut out = vec![];
        for k in ORDER {
            if let Some(d) = a.take(k, g) {
                if talks && d != Do::OpenRoute {
                    g.cutscene = true;
                }
                out.push(d);
            }
        }
        out
    }

    #[test]
    fn nothing_while_fading_or_dead() {
        let mut a = After::default();
        a.set_wake(true);
        a.camp(1);
        assert!(frame(&mut a, FADING, false).is_empty());
        let dead = Gate {
            alive: false,
            ..OPEN
        };
        assert!(frame(&mut a, dead, false).is_empty());
        assert_eq!(
            frame(&mut a, OPEN, false),
            vec![Do::SayWake, Do::CampArrive(1)]
        );
        assert!(frame(&mut a, OPEN, false).is_empty(), "한 번만");
    }

    #[test]
    fn village_run_goes_camp_then_first_line_then_route() {
        let mut a = After::default();
        a.village_run();
        // 내려가는 단면 동안 쉼터에 닿는다 — 아직 아무것도 안 한다.
        assert!(frame(&mut a, FADING, false).is_empty());
        a.camp(1);
        assert!(frame(&mut a, FADING, false).is_empty());
        // 단면이 끝나면: 도착 한마디 대신(마을에서 온 런) 런 첫 한마디, 그 대화가 끝날 때까지 길 지도는 기다린다.
        assert_eq!(frame(&mut a, OPEN, true), vec![Do::SayRunStart]);
        let talking = Gate {
            cutscene: true,
            ..OPEN
        };
        assert!(frame(&mut a, talking, false).is_empty());
        assert_eq!(frame(&mut a, OPEN, false), vec![Do::OpenRoute]);
        // 다음 쉼터는 보통 도착 한마디.
        a.camp(2);
        assert_eq!(frame(&mut a, OPEN, false), vec![Do::CampArrive(2)]);
    }

    #[test]
    fn first_line_waits_for_camp() {
        let mut a = After::default();
        a.village_run();
        a.camp(1);
        // 쉼터 일이 남아 있는 동안 런 첫 한마디는 안 나온다(같은 프레임에 쉼터가 먼저 빠진다).
        let dead = Gate {
            alive: false,
            ..OPEN
        };
        assert!(frame(&mut a, dead, false).is_empty());
        assert_eq!(a.take(Kind::RunStart, OPEN), None);
    }

    #[test]
    fn wake_can_be_cancelled_and_clear_drops_all() {
        let mut a = After::default();
        a.set_wake(true);
        a.set_wake(false);
        assert!(frame(&mut a, OPEN, false).is_empty());
        a.village_run();
        a.camp(3);
        a.clear();
        assert!(frame(&mut a, OPEN, false).is_empty());
        a.camp(1);
        assert_eq!(
            frame(&mut a, OPEN, false),
            vec![Do::CampArrive(1)],
            "마을 런 표시도 지워졌다"
        );
    }
}

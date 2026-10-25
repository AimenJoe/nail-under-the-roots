//! 장면 상태 기계와 메뉴. 인트로 → 타이틀 → (설정) → 게임 ⇄ 일시정지(메뉴·상태·가방·저널·지도).
//!
//! 메뉴는 커서 없이 동작한다(상하좌우 + 확인 + 뒤로, 탭은 Q/E·L1/R1). 탭/클릭은 보조로,
//! 직전 프레임에 그린 요소의 사각형(`hits`)에 맞춰 같은 동작을 한다. (CLAUDE.md "UI는 커서 없이")
//! 아이템·퀘스트·층 구조는 아직 데이터가 없다. 틀만 있고 내용은 다음 단계에서 들어온다(docs/decisions/0007).

use macroquad::prelude::*;

use crate::art::Art;
use crate::cutscene::{self, Cutscene};
use crate::equipment::{Character, WEAPONS};
use crate::input::{Actions, Input};
use crate::render::{self, Renderer};
use crate::scenario::{self, Scenario};
use crate::sfx::Sfx;
use crate::tuning as t;
use crate::tutorial::Tutorial;
use crate::ui::{self, Ui, FONT_PX};
use crate::world::World;

const INTRO_LINES: [&str; 3] = [
    "큰 나무가 시들어 간다.",
    "뿌리가 땅속 이웃들을 이상하게 바꿔 놓았다.",
    "아기 두더지 Naïl은 원인을 찾아 뿌리를 따라 내려간다.",
];
const INTRO_LINE_TIME: f32 = 2.6;
#[cfg(feature = "dev")]
const TITLE_ITEMS: [&str; 3] = ["시작", "설정", "개발"];
#[cfg(not(feature = "dev"))]
const TITLE_ITEMS: [&str; 2] = ["시작", "설정"];
const PAUSE_TABS: [&str; 5] = ["메뉴", "상태", "가방", "저널", "지도"];
const PAUSE_MENU: [&str; 4] = ["계속하기", "다시 시작", "설정", "타이틀로"];
const SETTING_ROWS: [&str; 4] = ["소리", "화면 흔들림", "디버그 정보", "돌아가기"];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Scene {
    Intro,
    Title,
    Settings {
        back: Back,
    },
    /// 개발용 시나리오 목록.
    Dev,
    /// 컷씬. 끝나면 게임.
    Cutscene,
    Game,
    Pause,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Back {
    Title,
    Pause,
}

/// 설정. 아직 저장하지 않는다(저장은 3단계). 값은 0..=10 단계.
pub struct Settings {
    pub volume: u8,
    pub shake: u8,
    pub debug_overlay: bool,
}

impl Settings {
    fn volume_f(&self) -> f32 {
        self.volume as f32 / 10.0
    }
    fn shake_f(&self) -> f32 {
        self.shake as f32 / 10.0
    }
}

pub struct Quest {
    pub title: &'static str,
    pub desc: &'static str,
    pub goal: u32,
    pub progress: u32,
}

impl Quest {
    pub fn done(&self) -> bool {
        self.progress >= self.goal
    }
}

/// 저널. 지금은 프로토타입 통계에 묶인 튜토리얼 퀘스트뿐.
pub struct Journal {
    pub quests: Vec<Quest>,
}

impl Journal {
    fn new() -> Self {
        Self {
            quests: vec![
                Quest {
                    title: "뿌리를 따라",
                    desc: "시들어 가는 나무의 뿌리를 따라 더 깊이 내려간다. (층 구조는 3단계에서)",
                    goal: 1,
                    progress: 0,
                },
                Quest {
                    title: "발톱 길들이기",
                    desc: "발톱 3연타로 적을 세 마리 처치한다. 3타째가 묵직한지 느껴 볼 것.",
                    goal: 3,
                    progress: 0,
                },
                Quest {
                    title: "기습",
                    desc: "굴파기로 땅속에 들어갔다가 솟구쳐서 적을 처치한다.",
                    goal: 1,
                    progress: 0,
                },
                Quest {
                    title: "버티기",
                    desc: "웨이브 3까지 살아남는다.",
                    goal: 3,
                    progress: 1,
                },
            ],
        }
    }

    fn update(&mut self, w: &World) {
        self.quests[1].progress = w.kills.min(self.quests[1].goal);
        self.quests[2].progress = w.emerge_kills.min(self.quests[2].goal);
        self.quests[3].progress = w.wave.min(self.quests[3].goal);
    }
}

pub enum Item {
    /// equipment::WEAPONS 번호.
    Weapon(usize),
}

/// 가방. 지금은 무기만 들어간다. 그 밖의 아이템은 2단계(방 클리어 보상)에서.
pub struct Inventory {
    pub slots: Vec<Option<Item>>,
    pub cols: usize,
    /// 장착 중인 무기(WEAPONS 번호). 맨손이면 None.
    pub equipped: Option<usize>,
}

impl Inventory {
    /// `weapons`를 넣고 첫 번째를 장착한다. 비어 있으면 맨손.
    fn new(weapons: &[usize]) -> Self {
        let mut inv = Self {
            slots: (0..15).map(|_| None).collect(),
            cols: 5,
            equipped: None,
        };
        for w in weapons {
            inv.add(*w);
        }
        inv.equipped = weapons.first().copied();
        inv
    }

    fn has(&self, w: usize) -> bool {
        self.slots
            .iter()
            .any(|s| matches!(s, Some(Item::Weapon(x)) if *x == w))
    }

    /// 빈 칸에 넣는다. 이미 있으면 그대로.
    fn add(&mut self, w: usize) {
        if self.has(w) {
            return;
        }
        if let Some(slot) = self.slots.iter_mut().find(|s| s.is_none()) {
            *slot = Some(Item::Weapon(w));
        }
    }
}

/// 무기 획득 창(JRPG식). 떠 있는 동안 월드는 멈춘다.
struct Gift {
    weapon: usize,
    t: f32,
}

/// 가방 칸의 발톱 아이콘: 크림/무기 색 발톱 세 개.
fn draw_claw_icon(ui: &Ui, r: Rect, color: Color) {
    let dark = Color::new(color.r * 0.6, color.g * 0.6, color.b * 0.6, 1.0);
    for k in 0..3 {
        let x = r.x + 6.0 + k as f32 * 4.0;
        ui.rect(Rect::new(x, r.y + 6.0 + (k % 2) as f32, 2.0, 8.0), color);
        ui.rect(Rect::new(x, r.y + 13.0 + (k % 2) as f32, 2.0, 2.0), dark);
    }
    ui.rect(Rect::new(r.x + 5.0, r.y + 15.0, 12.0, 2.0), dark);
}

/// 직전 프레임에 그린, 탭/클릭에 반응하는 요소.
#[derive(Clone, Copy)]
enum Hit {
    TitleItem(usize),
    PauseTab(usize),
    PauseItem(usize),
    SettingRow(usize),
    SettingLeft(usize),
    SettingRight(usize),
    InvSlot(usize),
    JournalItem(usize),
    DevItem(usize),
}

pub struct App {
    scene: Scene,
    intro_t: f32,
    title_cursor: usize,
    pause_tab: usize,
    pause_cursor: usize,
    settings_cursor: usize,
    inv_cursor: usize,
    journal_cursor: usize,
    dev_cursor: usize,
    /// 지금 플레이 중인 시나리오. "다시 시작"은 이걸 다시 만든다.
    current: Scenario,
    cutscene: Option<Cutscene>,
    tutorial: Option<Tutorial>,
    gift: Option<Gift>,
    /// 아직 창을 못 띄운 획득(여러 개가 한꺼번에 들어올 때).
    gift_queue: Vec<usize>,
    pub settings: Settings,
    world: World,
    journal: Journal,
    inventory: Inventory,
    sfx: Sfx,
    input: Input,
    renderer: Renderer,
    acc: f32,
    run_time: f32,
    /// 쓰러진 뒤 흐른 시간. 사망 연출과 입력 잠금.
    death_t: f32,
    /// 컷씬 레터박스(0 = 없음, 1 = 다 내려옴). 조작이 돌아오면 걷힌다.
    letterbox: f32,
    hits: Vec<(Rect, Hit)>,
    /// 직전 프레임의 (scale, ox, oy). 탭 좌표를 논리 좌표로 바꿀 때.
    last_ui: (f32, f32, f32),
}

impl App {
    pub fn new(sfx: Sfx, font: Option<Font>, art: Option<Art>) -> Self {
        let settings = Settings {
            volume: 7,
            shake: 10,
            debug_overlay: false,
        };
        sfx.set_volume(settings.volume_f());
        let mut world = World::new();
        world.debug.overlay = settings.debug_overlay;
        Self {
            scene: Scene::Intro,
            intro_t: 0.0,
            title_cursor: 0,
            pause_tab: 0,
            pause_cursor: 0,
            settings_cursor: 0,
            inv_cursor: 0,
            journal_cursor: 0,
            dev_cursor: 0,
            current: Scenario::Normal,
            cutscene: None,
            tutorial: None,
            gift_queue: Vec::new(),
            settings,
            world,
            journal: Journal::new(),
            inventory: Inventory::new(&[]),
            gift: None,
            sfx,
            input: Input::new(),
            renderer: Renderer::new(font, art),
            acc: 0.0,
            run_time: 0.0,
            death_t: 0.0,
            letterbox: 0.0,
            hits: Vec::new(),
            last_ui: (1.0, 0.0, 0.0),
        }
    }

    /// 프레임마다 한 번: 입력 → 장면 갱신 → 그리기.
    pub fn frame(&mut self) {
        self.input.set_menu_mode(self.scene != Scene::Game);
        self.input.poll();
        let dt = get_frame_time().min(t::MAX_FRAME_DT);
        let target = if self.scene == Scene::Cutscene {
            1.0
        } else {
            0.0
        };
        let step = dt / t::LETTERBOX_TIME;
        self.letterbox = if self.letterbox < target {
            (self.letterbox + step).min(target)
        } else {
            (self.letterbox - step).max(target)
        };
        match self.scene {
            Scene::Game => self.update_game(dt),
            Scene::Cutscene => {
                let a = self.input.take();
                if let Some(c) = self.cutscene.as_mut() {
                    c.update(&a, dt, &mut self.world);
                    // 컷씬에서 받은 무기는 컷씬이 끝난 뒤 창으로 보여 준다.
                    self.gift_queue.append(&mut self.world.gifts);
                    if c.is_done() {
                        self.cutscene = None;
                        self.scene = Scene::Game;
                    }
                } else {
                    self.scene = Scene::Game;
                }
            }
            _ => {
                let a = self.input.take();
                self.update_menu(&a, dt);
            }
        }
        self.draw();
    }

    // ---- 갱신 ---------------------------------------------------------------

    fn update_game(&mut self, dt: f32) {
        self.acc += dt;
        if self.gift.is_some() {
            self.update_gift(dt);
            return;
        }
        let mut open_pause = false;
        let mut restart = false;
        while self.acc >= t::TICK_DT {
            let a = self.input.take();
            if a.toggle_debug {
                self.settings.debug_overlay = !self.settings.debug_overlay;
                self.world.debug.overlay = self.settings.debug_overlay;
            }
            if a.toggle_hitboxes {
                self.world.debug.hitboxes = !self.world.debug.hitboxes;
            }
            if a.menu {
                open_pause = true;
            }
            let dead = !self.world.player.is_alive();
            // 쓰러진 직후의 연타는 버린다. 잠금이 풀린 뒤 새로 누른 확인만 재시작.
            let unlocked = dead && self.death_t >= t::DEATH_INPUT_LOCK;
            if a.restart || (unlocked && (a.confirm || a.attack)) {
                restart = true;
            }
            self.world.tick(&a, &self.sfx);
            self.acc -= t::TICK_DT;
        }
        if self.world.player.is_alive() {
            self.run_time += dt;
        } else {
            self.death_t += dt;
        }
        self.take_gifts();
        if let Some(tu) = self.tutorial.as_mut() {
            tu.update(&mut self.world, dt);
            if tu.is_done() {
                self.tutorial = None;
            }
        }
        self.journal.update(&self.world);
        if restart {
            self.start_game();
        } else if open_pause {
            self.open_pause();
        }
    }

    fn open_pause(&mut self) {
        self.world.freeze();
        self.pause_tab = 0;
        self.pause_cursor = 0;
        self.scene = Scene::Pause;
    }

    /// 시나리오로 새 런을 시작한다. Normal과 CutsceneOnly는 오프닝 컷씬을 먼저 재생한다.
    fn start(&mut self, s: Scenario) {
        let hitboxes = self.world.debug.hitboxes;
        self.current = s;
        self.world = scenario::build(s);
        self.world.debug.overlay = self.settings.debug_overlay;
        self.world.debug.hitboxes = hitboxes;
        self.journal = Journal::new();
        // 무기는 스토리를 따라 하나씩(docs/decisions/0012). 첫 시작·튜토리얼은 맨손에서 출발해
        // 튜토리얼에서 두더지 발톱을 줍는다. 개발 시나리오는 테스트를 위해 기본 발톱(또는 전부)을 들고 시작.
        let start = Character::Mole.starting_weapon();
        self.inventory = match s {
            Scenario::Normal | Scenario::Tutorial => Inventory::new(&[]),
            Scenario::AllClaws => Inventory::new(&[start, 0, 2, 3]),
            _ => Inventory::new(&[start]),
        };
        self.gift = None;
        self.gift_queue.clear();
        self.apply_equipped();
        self.run_time = 0.0;
        self.acc = 0.0;
        self.death_t = 0.0;
        self.tutorial = None;
        // 첫 시작은 컷씬 → 플레이 튜토리얼 → 웨이브 1. 튜토리얼이 끝나야 자동 웨이브가 켜진다.
        if matches!(s, Scenario::Normal | Scenario::Tutorial) {
            self.tutorial = Some(Tutorial::new(&mut self.world));
        }
        if matches!(s, Scenario::Normal | Scenario::CutsceneOnly) {
            self.world.enemies.clear();
            self.world.wave = 0;
            self.world.player.pos = vec2(40.0, 90.0);
            self.cutscene = Some(cutscene::opening(&self.world));
            self.scene = Scene::Cutscene;
        } else {
            self.scene = Scene::Game;
        }
    }

    fn start_game(&mut self) {
        self.start(self.current);
    }

    fn hit_at(&self, screen: Vec2) -> Option<Hit> {
        let (scale, ox, oy) = self.last_ui;
        let p = vec2((screen.x - ox) / scale, (screen.y - oy) / scale);
        self.hits
            .iter()
            .find(|(r, _)| r.contains(p))
            .map(|(_, h)| *h)
    }

    fn update_menu(&mut self, a: &Actions, dt: f32) {
        let tap_hit = a.tap.and_then(|p| self.hit_at(p));
        match self.scene {
            Scene::Intro => {
                self.intro_t += dt;
                let skip = a.confirm || a.attack || a.menu || a.tap.is_some();
                if skip || self.intro_t > INTRO_LINE_TIME * INTRO_LINES.len() as f32 + 0.5 {
                    self.scene = Scene::Title;
                }
            }
            Scene::Title => {
                if a.up {
                    self.title_cursor =
                        (self.title_cursor + TITLE_ITEMS.len() - 1) % TITLE_ITEMS.len();
                }
                if a.down {
                    self.title_cursor = (self.title_cursor + 1) % TITLE_ITEMS.len();
                }
                let chosen = if a.confirm {
                    Some(self.title_cursor)
                } else if let Some(Hit::TitleItem(i)) = tap_hit {
                    Some(i)
                } else {
                    None
                };
                match chosen {
                    Some(0) => self.start(Scenario::Normal),
                    Some(1) => {
                        self.settings_cursor = 0;
                        self.scene = Scene::Settings { back: Back::Title };
                    }
                    Some(2) => {
                        self.dev_cursor = 0;
                        self.scene = Scene::Dev;
                    }
                    _ => {}
                }
            }
            Scene::Dev => {
                let n = scenario::LIST.len();
                if a.up {
                    self.dev_cursor = (self.dev_cursor + n - 1) % n;
                }
                if a.down {
                    self.dev_cursor = (self.dev_cursor + 1) % n;
                }
                if a.cancel {
                    self.scene = Scene::Title;
                    return;
                }
                let chosen = if a.confirm {
                    Some(self.dev_cursor)
                } else if let Some(Hit::DevItem(i)) = tap_hit {
                    Some(i)
                } else {
                    None
                };
                if let Some(i) = chosen {
                    self.start(scenario::LIST[i].scenario);
                }
            }
            Scene::Settings { back } => {
                let n = SETTING_ROWS.len();
                if a.up {
                    self.settings_cursor = (self.settings_cursor + n - 1) % n;
                }
                if a.down {
                    self.settings_cursor = (self.settings_cursor + 1) % n;
                }
                let mut adjust = 0i8;
                if a.left {
                    adjust = -1;
                }
                if a.right {
                    adjust = 1;
                }
                let mut row = self.settings_cursor;
                match tap_hit {
                    Some(Hit::SettingLeft(i)) => {
                        row = i;
                        adjust = -1;
                    }
                    Some(Hit::SettingRight(i)) => {
                        row = i;
                        adjust = 1;
                    }
                    Some(Hit::SettingRow(i)) => {
                        row = i;
                        if i == 3 {
                            adjust = 0;
                        } else {
                            adjust = 1;
                        }
                    }
                    _ => {}
                }
                if adjust != 0 {
                    self.settings_cursor = row;
                    self.adjust_setting(row, adjust);
                }
                let back_chosen = a.cancel
                    || (a.confirm && self.settings_cursor == 3)
                    || matches!(tap_hit, Some(Hit::SettingRow(3)));
                if back_chosen {
                    self.scene = match back {
                        Back::Title => Scene::Title,
                        Back::Pause => Scene::Pause,
                    };
                }
            }
            Scene::Pause => self.update_pause(a, tap_hit),
            Scene::Game | Scene::Cutscene => {}
        }
    }

    fn adjust_setting(&mut self, row: usize, d: i8) {
        match row {
            0 => {
                self.settings.volume = (self.settings.volume as i8 + d).clamp(0, 10) as u8;
                self.sfx.set_volume(self.settings.volume_f());
                self.sfx.play(crate::sfx::Id::EnemyHit);
            }
            1 => self.settings.shake = (self.settings.shake as i8 + d).clamp(0, 10) as u8,
            2 => {
                self.settings.debug_overlay = !self.settings.debug_overlay;
                self.world.debug.overlay = self.settings.debug_overlay;
            }
            _ => {}
        }
    }

    fn update_pause(&mut self, a: &Actions, tap_hit: Option<Hit>) {
        let n_tabs = PAUSE_TABS.len();
        if a.cancel || a.menu {
            self.scene = Scene::Game;
            return;
        }
        if a.tab_prev {
            self.pause_tab = (self.pause_tab + n_tabs - 1) % n_tabs;
        }
        if a.tab_next {
            self.pause_tab = (self.pause_tab + 1) % n_tabs;
        }
        if let Some(Hit::PauseTab(i)) = tap_hit {
            self.pause_tab = i;
        }
        // 가로 탐색이 없는 탭에서는 좌우로도 탭을 넘긴다.
        let horizontal_tab = self.pause_tab != 2;
        if horizontal_tab && a.left {
            self.pause_tab = (self.pause_tab + n_tabs - 1) % n_tabs;
        }
        if horizontal_tab && a.right {
            self.pause_tab = (self.pause_tab + 1) % n_tabs;
        }
        match self.pause_tab {
            0 => {
                let n = PAUSE_MENU.len();
                if a.up {
                    self.pause_cursor = (self.pause_cursor + n - 1) % n;
                }
                if a.down {
                    self.pause_cursor = (self.pause_cursor + 1) % n;
                }
                let chosen = if a.confirm {
                    Some(self.pause_cursor)
                } else if let Some(Hit::PauseItem(i)) = tap_hit {
                    Some(i)
                } else {
                    None
                };
                match chosen {
                    Some(0) => self.scene = Scene::Game,
                    Some(1) => self.start_game(),
                    Some(2) => {
                        self.settings_cursor = 0;
                        self.scene = Scene::Settings { back: Back::Pause };
                    }
                    Some(3) => {
                        self.title_cursor = 0;
                        self.scene = Scene::Title;
                    }
                    _ => {}
                }
            }
            2 => {
                let cols = self.inventory.cols;
                let n = self.inventory.slots.len();
                if a.left && !self.inv_cursor.is_multiple_of(cols) {
                    self.inv_cursor -= 1;
                }
                if a.right && self.inv_cursor % cols < cols - 1 && self.inv_cursor + 1 < n {
                    self.inv_cursor += 1;
                }
                if a.up && self.inv_cursor >= cols {
                    self.inv_cursor -= cols;
                }
                if a.down && self.inv_cursor + cols < n {
                    self.inv_cursor += cols;
                }
                let mut equip = a.confirm;
                if let Some(Hit::InvSlot(i)) = tap_hit {
                    equip = self.inv_cursor == i;
                    self.inv_cursor = i;
                }
                if equip {
                    if let Some(Item::Weapon(w)) = self.inventory.slots[self.inv_cursor] {
                        // 캐릭터가 쓸 수 있는 종류만.
                        if WEAPONS[w].kind == Character::Mole.weapon_kind() {
                            self.inventory.equipped = Some(w);
                            self.apply_equipped();
                            self.sfx.play(crate::sfx::Id::Claw(2));
                        }
                    }
                }
            }
            3 => {
                let n = self.journal.quests.len();
                if a.up {
                    self.journal_cursor = (self.journal_cursor + n - 1) % n;
                }
                if a.down {
                    self.journal_cursor = (self.journal_cursor + 1) % n;
                }
                if let Some(Hit::JournalItem(i)) = tap_hit {
                    self.journal_cursor = i;
                }
            }
            _ => {}
        }
    }

    // ---- 그리기 ---------------------------------------------------------------

    fn draw(&mut self) {
        let in_world = matches!(
            self.scene,
            Scene::Game | Scene::Pause | Scene::Cutscene | Scene::Settings { back: Back::Pause }
        );
        let alpha = if self.scene == Scene::Game {
            self.acc / t::TICK_DT
        } else {
            1.0
        };
        self.renderer.begin_world(
            if in_world { Some(&self.world) } else { None },
            alpha,
            self.settings.shake_f(),
        );
        if let Some(c) = &self.cutscene {
            for actor in c.actors.iter().skip(1).filter(|a| a.visible) {
                render::draw_actor(actor.pos, actor.facing, actor.color);
            }
        }
        let ui = self.renderer.present();
        self.hits.clear();
        self.last_ui = (ui.scale, ui.ox, ui.oy);

        match self.scene {
            Scene::Intro => self.draw_intro(&ui),
            Scene::Title => self.draw_title(&ui),
            Scene::Dev => self.draw_dev(&ui),
            Scene::Cutscene => {
                self.draw_letterbox(&ui);
                self.draw_cutscene(&ui);
            }
            Scene::Settings { back } => {
                if back == Back::Pause {
                    self.renderer
                        .draw_hud(&ui, &self.world, self.input.gamepad_count);
                    ui.dim();
                }
                self.draw_settings(&ui);
            }
            Scene::Game => {
                self.draw_letterbox(&ui);
                self.renderer
                    .draw_hud(&ui, &self.world, self.input.gamepad_count);
                if let Some(g) = &self.gift {
                    self.draw_gift(&ui, g);
                } else if let Some(h) = self.tutorial.as_ref().and_then(|t| t.hint()) {
                    self.draw_hint(&ui, h);
                }
                if !self.world.player.is_alive() {
                    self.draw_dead(&ui);
                }
            }
            Scene::Pause => {
                self.renderer
                    .draw_hud(&ui, &self.world, self.input.gamepad_count);
                ui.dim();
                self.draw_pause(&ui);
            }
        }
        if let Some(o) = self.input.overlay() {
            render::draw_touch_overlay(&o);
        }
    }

    /// 컷씬 레터박스: 화면 가장자리 비네트 + 위아래 검은 띠 + 안쪽으로 번지는 그림자.
    /// 대화창·건너뛰기 안내는 그 위에. 조작이 돌아오면 같은 속도로 걷힌다.
    fn draw_letterbox(&self, ui: &Ui) {
        if self.letterbox <= 0.0 {
            return;
        }
        let k = self.letterbox;
        let e = k * k * (3.0 - 2.0 * k);
        self.renderer.draw_vignette(ui, t::CUTSCENE_VIGNETTE * e);
        let bar = (t::LETTERBOX_BAR * e).round();
        let soft = 8.0;
        ui.rect(Rect::new(0.0, 0.0, 320.0, bar), BLACK);
        ui.rect(Rect::new(0.0, 180.0 - bar, 320.0, bar), BLACK);
        for i in 0..soft as i32 {
            let a = 0.55 * e * (1.0 - i as f32 / soft);
            let c = Color::new(0.0, 0.0, 0.0, a);
            ui.rect(Rect::new(0.0, bar + i as f32, 320.0, 1.0), c);
            ui.rect(Rect::new(0.0, 180.0 - bar - 1.0 - i as f32, 320.0, 1.0), c);
        }
    }

    fn draw_intro(&self, ui: &Ui) {
        let idx = (self.intro_t / INTRO_LINE_TIME) as usize;
        if let Some(line) = INTRO_LINES.get(idx) {
            let p = (self.intro_t - idx as f32 * INTRO_LINE_TIME) / INTRO_LINE_TIME;
            let a = (p * 4.0).min(1.0) * ((1.0 - p) * 4.0).min(1.0);
            ui.text_center(line, 160.0, 84.0, FONT_PX, Color::new(0.92, 0.86, 0.76, a));
        }
        ui.text_center("확인: 건너뛰기", 160.0, 160.0, FONT_PX, ui::TEXT_DIM);
    }

    fn draw_title(&mut self, ui: &Ui) {
        ui.text_center("Naïl", 160.0, 34.0, FONT_PX * 2.0, ui::ACCENT);
        ui.text_center("Under the Roots", 160.0, 60.0, FONT_PX, ui::TEXT);
        let rects = ui.menu_list(&TITLE_ITEMS, self.title_cursor, 124.0, 92.0, 72.0);
        for (i, r) in rects.into_iter().enumerate() {
            self.hits.push((r, Hit::TitleItem(i)));
        }
        ui.text_center(
            "상하 이동 · 확인(Enter/A) · 뒤로(Esc/B)",
            160.0,
            150.0,
            FONT_PX,
            ui::TEXT_DIM,
        );
        ui.text_right("프로토타입 v0.1", 316.0, 166.0, FONT_PX, ui::TEXT_DIM);
    }

    fn draw_dev(&mut self, ui: &Ui) {
        let panel = Rect::new(24.0, 16.0, 272.0, 148.0);
        ui.panel(panel, Some("개발 시나리오"));
        let names: Vec<&str> = scenario::LIST.iter().map(|e| e.name).collect();
        let rects = ui.menu_list(
            &names,
            self.dev_cursor,
            panel.x + 6.0,
            panel.y + 24.0,
            110.0,
        );
        for (i, r) in rects.into_iter().enumerate() {
            self.hits.push((r, Hit::DevItem(i)));
        }
        let e = &scenario::LIST[self.dev_cursor];
        let dx = panel.x + 126.0;
        ui.text(e.name, dx, panel.y + 26.0, FONT_PX, ui::ACCENT);
        for (i, l) in ui::wrap(ui, e.desc, panel.right() - dx - 6.0, FONT_PX)
            .iter()
            .enumerate()
        {
            ui.text(l, dx, panel.y + 42.0 + i as f32 * 14.0, FONT_PX, ui::TEXT);
        }
        for (i, l) in ["확인: 시작 · 뒤로: 타이틀", "적 구성은 src/scenario.rs"]
            .iter()
            .enumerate()
        {
            ui.text(
                l,
                dx,
                panel.bottom() - 30.0 + i as f32 * 13.0,
                FONT_PX,
                ui::TEXT_DIM,
            );
        }
    }

    /// JRPG식 대화창: 아래 전체 폭, 이름표, 초상 자리, 타자 효과, ▼.
    fn draw_cutscene(&self, ui: &Ui) {
        let Some(c) = &self.cutscene else { return };
        if let Some(d) = c.dialogue() {
            let b = Rect::new(8.0, 124.0, 304.0, 50.0);
            ui.panel(b, None);
            // 이름표
            let name_w = ui.text_w(d.name, FONT_PX) + 10.0;
            let tag = Rect::new(b.x + 6.0, b.y - 8.0, name_w, FONT_PX + 4.0);
            ui.rect(tag, ui::PANEL_LIGHT);
            ui.rect_lines(tag, ui::BORDER);
            ui.text(d.name, tag.x + 5.0, tag.y + 2.0, FONT_PX, ui::ACCENT);
            // 초상 자리(그림이 없으니 색 네모)
            let portrait = Rect::new(b.x + 8.0, b.y + 10.0, 28.0, 28.0);
            ui.rect(portrait, d.color);
            ui.rect_lines(portrait, ui::BORDER);
            // 본문(타자 효과)
            let shown: String = d.text.chars().take(d.shown_chars).collect();
            let tx = portrait.right() + 8.0;
            let tw = b.right() - tx - 10.0;
            for (i, l) in ui::wrap(ui, &shown, tw, FONT_PX).iter().take(3).enumerate() {
                ui.text(l, tx, b.y + 9.0 + i as f32 * 13.0, FONT_PX, ui::TEXT);
            }
            if d.complete && (get_time() * 2.0) as i64 % 2 == 0 {
                ui.text(
                    "▼",
                    b.right() - 16.0,
                    b.bottom() - 15.0,
                    FONT_PX,
                    ui::ACCENT,
                );
            }
        }
        ui.text_center("뒤로(Esc/B): 건너뛰기", 160.0, 1.0, FONT_PX, ui::TEXT_DIM);
        if c.fade > 0.0 {
            draw_rectangle(
                0.0,
                0.0,
                screen_width(),
                screen_height(),
                Color::new(0.0, 0.0, 0.0, c.fade),
            );
        }
    }

    fn draw_settings(&mut self, ui: &Ui) {
        let panel = Rect::new(56.0, 36.0, 208.0, 104.0);
        ui.panel(panel, Some("설정"));
        let values = [
            format!("{}%", self.settings.volume * 10),
            format!("{}%", self.settings.shake * 10),
            if self.settings.debug_overlay {
                "켬"
            } else {
                "끔"
            }
            .to_string(),
            String::new(),
        ];
        let row_h = FONT_PX + 6.0;
        for (i, label) in SETTING_ROWS.iter().enumerate() {
            let y = panel.y + 22.0 + i as f32 * row_h;
            let selected = self.settings_cursor == i;
            if i == 3 {
                let r = Rect::new(panel.x + 6.0, y, panel.w - 12.0, FONT_PX + 4.0);
                if selected {
                    ui.rect(r, ui::PANEL_LIGHT);
                    ui.text("▶", r.x + 2.0, r.y + 2.0, FONT_PX, ui::ACCENT);
                }
                ui.text(
                    label,
                    r.x + 14.0,
                    r.y + 2.0,
                    FONT_PX,
                    if selected { ui::TEXT } else { ui::TEXT_DIM },
                );
                self.hits.push((r, Hit::SettingRow(3)));
            } else {
                let (row, l, r) = ui.setting_row(
                    label,
                    &values[i],
                    panel.x + 6.0,
                    y,
                    panel.w - 12.0,
                    selected,
                );
                self.hits.push((l, Hit::SettingLeft(i)));
                self.hits.push((r, Hit::SettingRight(i)));
                self.hits.push((row, Hit::SettingRow(i)));
            }
        }
        ui.text_center(
            "좌우로 값 바꾸기 · 설정은 아직 저장되지 않음",
            160.0,
            panel.bottom() + 6.0,
            FONT_PX,
            ui::TEXT_DIM,
        );
    }

    /// 장착 무기를 플레이어에 반영한다. 맨손이면 공격이 안 나간다.
    fn apply_equipped(&mut self) {
        match self.inventory.equipped {
            Some(w) => {
                self.world.player.moveset = WEAPONS[w].moveset;
                self.world.player.armed = true;
            }
            None => self.world.player.armed = false,
        }
    }

    /// 월드가 받은 무기(획득물·컷씬)를 가방에 넣고 획득 창을 띄운다.
    fn take_gifts(&mut self) {
        self.gift_queue.append(&mut self.world.gifts);
        if self.gift.is_some() || self.gift_queue.is_empty() {
            return;
        }
        let w = self.gift_queue.remove(0);
        self.inventory.add(w);
        if self.inventory.equipped.is_none() {
            self.inventory.equipped = Some(w);
            self.apply_equipped();
        }
        self.world.freeze();
        self.gift = Some(Gift { weapon: w, t: 0.0 });
    }

    /// 획득 창: 최소 시간이 지나면 확인으로 닫고, 그대로 두어도 닫힌다. 그동안 월드는 멈춘다.
    fn update_gift(&mut self, dt: f32) {
        let a = self.input.take();
        let Some(g) = self.gift.as_mut() else { return };
        g.t += dt;
        let close = (g.t >= t::GIFT_MIN && (a.confirm || a.attack || a.tap.is_some()))
            || g.t >= t::GIFT_AUTO_CLOSE;
        if close {
            self.gift = None;
            self.acc = 0.0;
            self.take_gifts();
        }
    }

    fn draw_gift(&self, ui: &Ui, g: &Gift) {
        let wp = &WEAPONS[g.weapon];
        let k = (g.t / 0.2).min(1.0);
        let r = Rect::new(56.0, 48.0 + (1.0 - k) * 8.0, 208.0, 74.0);
        ui.panel(r, None);
        ui.text_center("새 발톱을 얻었다!", 160.0, r.y + 5.0, FONT_PX, ui::ACCENT);
        let icon = Rect::new(r.x + 10.0, r.y + 22.0, 22.0, 22.0);
        ui.rect(icon, ui::PANEL_LIGHT);
        ui.rect_lines(icon, ui::BORDER);
        draw_claw_icon(ui, icon, wp.color);
        ui.text(wp.name, icon.right() + 8.0, r.y + 22.0, FONT_PX, ui::TEXT);
        let lines = ui::wrap(ui, wp.desc, r.right() - icon.right() - 16.0, FONT_PX);
        for (i, l) in lines.iter().take(2).enumerate() {
            ui.text(
                l,
                icon.right() + 8.0,
                r.y + 36.0 + i as f32 * 13.0,
                FONT_PX,
                ui::TEXT_DIM,
            );
        }
        if g.t >= t::GIFT_MIN && (get_time() * 2.0) as i64 % 2 == 0 {
            ui.text(
                "▼",
                r.right() - 14.0,
                r.bottom() - 14.0,
                FONT_PX,
                ui::ACCENT,
            );
        }
    }

    /// 튜토리얼 힌트: 아래 가운데 한 줄.
    fn draw_hint(&self, ui: &Ui, text: &str) {
        let w = ui.text_w(text, FONT_PX) + 16.0;
        let r = Rect::new(160.0 - w / 2.0, 158.0, w, FONT_PX + 8.0);
        ui.rect(r, ui::PANEL);
        ui.rect_lines(r, ui::BORDER);
        ui.text_center(text, 160.0, r.y + 4.0, FONT_PX, ui::TEXT);
    }

    /// 사망 연출: 서서히 어두워지고 글자가 떠오른 뒤, 입력 잠금이 풀리면 안내가 깜빡인다.
    fn draw_dead(&self, ui: &Ui) {
        let k = (self.death_t / t::DEATH_FADE).min(1.0);
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 0.55 * k),
        );
        ui.text_center(
            "쓰러졌다",
            160.0,
            64.0,
            FONT_PX * 2.0,
            Color::new(0.9, 0.3, 0.3, k),
        );
        if self.death_t >= t::DEATH_INPUT_LOCK && (get_time() * 2.0) as i64 % 2 == 0 {
            ui.text_center(
                "확인: 다시 시작   메뉴: 일시정지",
                160.0,
                96.0,
                FONT_PX,
                ui::TEXT,
            );
        }
    }

    fn draw_pause(&mut self, ui: &Ui) {
        let panel = Rect::new(12.0, 10.0, 296.0, 160.0);
        ui.panel(panel, None);
        let tabs = ui.tabs(&PAUSE_TABS, self.pause_tab, panel.x + 6.0, panel.y + 5.0);
        for (i, r) in tabs.into_iter().enumerate() {
            self.hits.push((r, Hit::PauseTab(i)));
        }
        ui.line(
            vec2(panel.x + 4.0, panel.y + 24.0),
            vec2(panel.right() - 4.0, panel.y + 24.0),
            ui::BORDER,
        );
        ui.text_right(
            "Q/E · L1/R1: 탭",
            panel.right() - 6.0,
            panel.y + 7.0,
            FONT_PX,
            ui::TEXT_DIM,
        );
        let content = Rect::new(
            panel.x + 8.0,
            panel.y + 30.0,
            panel.w - 16.0,
            panel.h - 38.0,
        );
        match self.pause_tab {
            0 => {
                let rects = ui.menu_list(
                    &PAUSE_MENU,
                    self.pause_cursor,
                    content.x + 90.0,
                    content.y + 12.0,
                    100.0,
                );
                for (i, r) in rects.into_iter().enumerate() {
                    self.hits.push((r, Hit::PauseItem(i)));
                }
            }
            1 => self.draw_status(ui, content),
            2 => self.draw_inventory(ui, content),
            3 => self.draw_journal(ui, content),
            _ => self.draw_map(ui, content),
        }
    }

    fn draw_status(&self, ui: &Ui, c: Rect) {
        let p = &self.world.player;
        let hearts: String = (0..t::PLAYER_HP)
            .map(|i| if i < p.hp { '♥' } else { '♡' })
            .collect();
        let secs = self.run_time as u32;
        let lines = [
            (
                Character::Mole.name().to_string(),
                "아기 두더지. 발톱으로 싸우고 발톱으로 판다.".to_string(),
            ),
            ("체력".to_string(), hearts),
            (
                "이번 런".to_string(),
                format!(
                    "웨이브 {}  처치 {}  시간 {:02}:{:02}",
                    self.world.wave,
                    self.world.kills,
                    secs / 60,
                    secs % 60
                ),
            ),
            if p.armed {
                (
                    p.moveset.name.to_string(),
                    format!(
                        "{}연타. 마지막 타 {:.2}s, 넉백 {}",
                        p.moveset.hits.len(),
                        p.moveset.hits[p.moveset.last()].swing,
                        p.moveset.hits[p.moveset.last()].knockback as i32
                    ),
                )
            } else {
                (
                    "맨손".to_string(),
                    "아직 발톱이 없다. 공격할 수 없다.".to_string(),
                )
            },
            (
                "굴파기".to_string(),
                format!(
                    "땅속 {:.2}s 무적, 솟구침 반경 {}px, 쿨다운 {:.1}s",
                    t::BURROW_MAX_TIME,
                    t::EMERGE_RADIUS as i32,
                    t::BURROW_COOLDOWN
                ),
            ),
            ("특기".to_string(), "(미정 — 2단계)".to_string()),
        ];
        let row_h = FONT_PX + 5.0;
        for (i, (k, v)) in lines.iter().enumerate() {
            let y = c.y + i as f32 * row_h;
            ui.text(k, c.x, y, FONT_PX, ui::ACCENT);
            ui.text(v, c.x + 52.0, y, FONT_PX, ui::TEXT);
        }
    }

    fn draw_inventory(&mut self, ui: &Ui, c: Rect) {
        let cols = self.inventory.cols;
        let slot = 22.0;
        let gap = 4.0;
        let x0 = c.x + 8.0;
        let y0 = c.y + 2.0;
        for (i, item) in self.inventory.slots.iter().enumerate() {
            let (col, row) = (i % cols, i / cols);
            let r = Rect::new(
                x0 + col as f32 * (slot + gap),
                y0 + row as f32 * (slot + gap),
                slot,
                slot,
            );
            ui.rect(r, ui::PANEL_LIGHT);
            ui.rect_lines(
                r,
                if i == self.inv_cursor {
                    ui::ACCENT
                } else {
                    ui::BORDER
                },
            );
            if let Some(Item::Weapon(w)) = item {
                draw_claw_icon(ui, r, WEAPONS[*w].color);
                if self.inventory.equipped == Some(*w) {
                    ui.text("E", r.x + 2.0, r.y + 1.0, FONT_PX, ui::ACCENT);
                }
            }
            self.hits.push((r, Hit::InvSlot(i)));
        }
        let desc_x = x0 + cols as f32 * (slot + gap) + 8.0;
        let desc_w = c.right() - desc_x;
        match &self.inventory.slots[self.inv_cursor] {
            Some(Item::Weapon(w)) => {
                let wp = &WEAPONS[*w];
                ui.text(wp.name, desc_x, y0, FONT_PX, ui::ACCENT);
                let mut y = y0 + 16.0;
                for l in ui::wrap(ui, wp.desc, desc_w, FONT_PX).iter() {
                    ui.text(l, desc_x, y, FONT_PX, ui::TEXT);
                    y += 14.0;
                }
                let ms = wp.moveset;
                let last = &ms.hits[ms.last()];
                ui.text(
                    &format!("{}연타 · 마지막 타 피해 {}", ms.hits.len(), last.damage),
                    desc_x,
                    y + 4.0,
                    FONT_PX,
                    ui::TEXT_DIM,
                );
                let hint = if self.inventory.equipped == Some(*w) {
                    "장착 중"
                } else {
                    "확인: 장착"
                };
                ui.text(hint, desc_x, y + 18.0, FONT_PX, ui::ACCENT);
            }
            None => {
                ui.text("비어 있음", desc_x, y0, FONT_PX, ui::TEXT_DIM);
                for (i, l) in ui::wrap(
                    ui,
                    "아이템은 2단계(방 클리어 보상 선택)에서 들어온다. 지금은 틀만 있다.",
                    desc_w,
                    FONT_PX,
                )
                .iter()
                .enumerate()
                {
                    ui.text(
                        l,
                        desc_x,
                        y0 + 16.0 + i as f32 * 14.0,
                        FONT_PX,
                        ui::TEXT_DIM,
                    );
                }
            }
        }
        ui.text(
            &format!(
                "{} / {} 칸",
                self.inventory.slots.iter().filter(|s| s.is_some()).count(),
                self.inventory.slots.len()
            ),
            x0,
            y0 + 3.0 * (slot + gap) + 2.0,
            FONT_PX,
            ui::TEXT_DIM,
        );
    }

    fn draw_journal(&mut self, ui: &Ui, c: Rect) {
        let row_h = FONT_PX + 4.0;
        for (i, q) in self.journal.quests.iter().enumerate() {
            let r = Rect::new(c.x, c.y + i as f32 * row_h, 150.0, row_h);
            let selected = i == self.journal_cursor;
            if selected {
                ui.rect(r, ui::PANEL_LIGHT);
            }
            let mark = if q.done() { "■" } else { "□" };
            let col = if q.done() {
                ui::TEXT_DIM
            } else if selected {
                ui::TEXT
            } else {
                ui::TEXT_DIM
            };
            ui.text(
                mark,
                r.x + 2.0,
                r.y + 2.0,
                FONT_PX,
                if q.done() { ui::ACCENT } else { col },
            );
            ui.text(q.title, r.x + 16.0, r.y + 2.0, FONT_PX, col);
            ui.text_right(
                &format!("{}/{}", q.progress, q.goal),
                r.right() - 4.0,
                r.y + 2.0,
                FONT_PX,
                col,
            );
            self.hits.push((r, Hit::JournalItem(i)));
        }
        let q = &self.journal.quests[self.journal_cursor];
        let dx = c.x + 158.0;
        let dw = c.right() - dx;
        ui.text(q.title, dx, c.y, FONT_PX, ui::ACCENT);
        for (i, l) in ui::wrap(ui, q.desc, dw, FONT_PX).iter().enumerate() {
            ui.text(l, dx, c.y + 16.0 + i as f32 * 14.0, FONT_PX, ui::TEXT);
        }
    }

    fn draw_map(&self, ui: &Ui, c: Rect) {
        ui.text(
            &format!("1층 · {}", self.world.biome.name()),
            c.x,
            c.y,
            FONT_PX,
            ui::ACCENT,
        );
        ui.text_right("프로토타입: 방 하나", c.right(), c.y, FONT_PX, ui::TEXT_DIM);
        // 방 전체(320×180)를 0.5배로.
        let k = 0.5;
        let origin = vec2(c.x + (c.w - t::VIEW_W * k) / 2.0, c.y + 18.0);
        let m = |p: Vec2| origin + p * k;
        let room = &self.world.room;
        ui.rect(
            Rect::new(origin.x, origin.y, t::VIEW_W * k, t::VIEW_H * k),
            Color::new(0.30, 0.22, 0.20, 1.0),
        );
        let inner = room.inner;
        ui.rect(
            Rect::new(
                origin.x + inner.x * k,
                origin.y + inner.y * k,
                inner.w * k,
                inner.h * k,
            ),
            Color::new(0.12, 0.09, 0.14, 1.0),
        );
        for p in &room.pillars {
            ui.rect(
                Rect::new(origin.x + p.x * k, origin.y + p.y * k, p.w * k, p.h * k),
                Color::new(0.30, 0.22, 0.20, 1.0),
            );
        }
        for e in &self.world.enemies {
            let p = m(e.pos);
            ui.rect(
                Rect::new(p.x - 1.5, p.y - 1.5, 3.0, 3.0),
                crate::world::enemy_color(e.kind),
            );
        }
        let p = m(self.world.player.pos);
        ui.rect(Rect::new(p.x - 2.0, p.y - 2.0, 4.0, 4.0), ui::ACCENT);
        ui.text(
            "층 구조·방 연결은 3단계(런 구조)에서. 지금은 현재 방만.",
            c.x,
            origin.y + t::VIEW_H * k + 6.0,
            FONT_PX,
            ui::TEXT_DIM,
        );
    }
}

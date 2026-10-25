//! 장면 상태 기계와 메뉴. 인트로 → 타이틀 → (설정) → 게임 ⇄ 일시정지(메뉴·상태·가방·저널·지도).
//!
//! 메뉴는 커서 없이 동작한다(상하좌우 + 확인 + 뒤로, 탭은 Q/E·L1/R1). 탭/클릭은 보조로,
//! 직전 프레임에 그린 요소의 사각형(`hits`)에 맞춰 같은 동작을 한다. (CLAUDE.md "UI는 커서 없이")
//! 아이템·퀘스트·층 구조는 아직 데이터가 없다. 틀만 있고 내용은 다음 단계에서 들어온다(docs/decisions/0007).

use macroquad::prelude::*;

use crate::art::Art;
use crate::cutscene::{self, Cutscene};
use crate::equipment::{Character, CHARMS, CHARM_SLOTS, WEAPONS};
use crate::i18n::{self, fill, tr, Tr};
use crate::input::{Actions, Input};
use crate::render::{self, Renderer};
use crate::scenario::{self, Scenario};
use crate::sfx::Sfx;
use crate::tuning as t;
use crate::tutorial::Tutorial;
use crate::ui::{self, Ui, FONT_PX};
use crate::world::World;

const INTRO_LINES: [Tr; 3] = [|s| s.intro_1, |s| s.intro_2, |s| s.intro_3];
const INTRO_LINE_TIME: f32 = 2.6;
#[cfg(feature = "dev")]
const TITLE_ITEMS: [Tr; 3] = [|s| s.title_start, |s| s.title_settings, |s| s.title_dev];
#[cfg(not(feature = "dev"))]
const TITLE_ITEMS: [Tr; 2] = [|s| s.title_start, |s| s.title_settings];
const PAUSE_TABS: [Tr; 5] = [
    |s| s.tab_menu,
    |s| s.tab_status,
    |s| s.tab_bag,
    |s| s.tab_journal,
    |s| s.tab_map,
];
const PAUSE_MENU: [Tr; 4] = [
    |s| s.pm_resume,
    |s| s.pm_restart,
    |s| s.pm_settings,
    |s| s.pm_title,
];
const SETTING_ROWS: [Tr; 5] = [
    |s| s.set_sound,
    |s| s.set_shake,
    |s| s.set_debug,
    |s| s.set_language,
    |s| s.set_back,
];
/// 설정의 "돌아가기" 줄.
const SETTING_BACK: usize = 4;

/// 표 → 지금 언어의 글 목록(메뉴 그리기용).
fn trs(items: &[Tr]) -> Vec<&'static str> {
    items.iter().map(|f| tr(*f)).collect()
}

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
    pub title: Tr,
    pub desc: Tr,
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
                    title: |s| s.q_roots,
                    desc: |s| s.q_roots_desc,
                    goal: 1,
                    progress: 0,
                },
                Quest {
                    title: |s| s.q_claws,
                    desc: |s| s.q_claws_desc,
                    goal: 3,
                    progress: 0,
                },
                Quest {
                    title: |s| s.q_ambush,
                    desc: |s| s.q_ambush_desc,
                    goal: 1,
                    progress: 0,
                },
                Quest {
                    title: |s| s.q_survive,
                    desc: |s| s.q_survive_desc,
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
    /// equipment::CHARMS 번호. 끼고 있는지는 world.charms.
    Charm(usize),
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

    /// 장신구를 빈 칸에 넣는다. 이미 있으면 그대로.
    fn add_charm(&mut self, c: usize) {
        if self
            .slots
            .iter()
            .any(|s| matches!(s, Some(Item::Charm(x)) if *x == c))
        {
            return;
        }
        if let Some(slot) = self.slots.iter_mut().find(|s| s.is_none()) {
            *slot = Some(Item::Charm(c));
        }
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

/// 가방 칸의 장신구 아이콘: 끈에 매단 둥근 돌.
fn draw_charm_icon(ui: &Ui, r: Rect, color: Color) {
    let dark = Color::new(color.r * 0.6, color.g * 0.6, color.b * 0.6, 1.0);
    ui.rect(Rect::new(r.x + 6.0, r.y + 4.0, 10.0, 1.0), ui::BORDER);
    ui.rect(Rect::new(r.x + 10.0, r.y + 5.0, 2.0, 3.0), ui::BORDER);
    ui.rect(Rect::new(r.x + 7.0, r.y + 8.0, 8.0, 9.0), dark);
    ui.rect(Rect::new(r.x + 8.0, r.y + 8.0, 6.0, 7.0), color);
    ui.rect(
        Rect::new(r.x + 8.0, r.y + 9.0, 2.0, 2.0),
        Color::new(1.0, 1.0, 1.0, 0.8),
    );
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
    pub fn new(sfx: Sfx, fonts: render::Fonts, art: Option<Art>) -> Self {
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
            renderer: Renderer::new(fonts, art),
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
            Scenario::Gear => {
                let mut inv = Inventory::new(&[start]);
                for c in 0..CHARMS.len() {
                    inv.add_charm(c);
                }
                inv
            }
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
                        if i == SETTING_BACK {
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
                    || (a.confirm && self.settings_cursor == SETTING_BACK)
                    || matches!(tap_hit, Some(Hit::SettingRow(SETTING_BACK)));
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
            3 => i18n::set_lang(i18n::lang().cycle(d)),
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
                    match self.inventory.slots[self.inv_cursor] {
                        Some(Item::Weapon(w)) => {
                            // 캐릭터가 쓸 수 있는 종류만.
                            if WEAPONS[w].kind == Character::Mole.weapon_kind() {
                                self.inventory.equipped = Some(w);
                                self.apply_equipped();
                                self.sfx.play(crate::sfx::Id::Claw(2));
                            }
                        }
                        Some(Item::Charm(c)) => {
                            // 끼고 있으면 빼고, 아니면 낀다. 칸이 가득이면 먼저 낀 것과 바꾼다.
                            let charms = &mut self.world.charms;
                            if let Some(i) = charms.iter().position(|x| *x == c) {
                                charms.remove(i);
                            } else {
                                if charms.len() >= CHARM_SLOTS {
                                    charms.remove(0);
                                }
                                charms.push(c);
                            }
                            self.world.refresh_stats();
                            self.sfx.play(crate::sfx::Id::Claw(1));
                        }
                        None => {}
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
                    self.draw_hint(&ui, tr(h));
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
        if let Some(line) = INTRO_LINES.get(idx).map(|f| tr(*f)) {
            let p = (self.intro_t - idx as f32 * INTRO_LINE_TIME) / INTRO_LINE_TIME;
            let a = (p * 4.0).min(1.0) * ((1.0 - p) * 4.0).min(1.0);
            ui.text_center(line, 160.0, 84.0, FONT_PX, Color::new(0.92, 0.86, 0.76, a));
        }
        ui.text_center(i18n::s().skip_hint, 160.0, 160.0, FONT_PX, ui::TEXT_DIM);
    }

    fn draw_title(&mut self, ui: &Ui) {
        ui.text_center("Naïl", 160.0, 34.0, FONT_PX * 2.0, ui::ACCENT);
        ui.text_center("Under the Roots", 160.0, 60.0, FONT_PX, ui::TEXT);
        let s = i18n::s();
        let rects = ui.menu_list(&trs(&TITLE_ITEMS), self.title_cursor, 124.0, 92.0, 72.0);
        for (i, r) in rects.into_iter().enumerate() {
            self.hits.push((r, Hit::TitleItem(i)));
        }
        ui.text_center(s.title_controls, 160.0, 150.0, FONT_PX, ui::TEXT_DIM);
        ui.text_right(s.version, 316.0, 166.0, FONT_PX, ui::TEXT_DIM);
    }

    fn draw_dev(&mut self, ui: &Ui) {
        // 개발 화면은 번역하지 않는다(한국어). 언어와 상관없이 한글 폰트로.
        let base = ui.with_base_font();
        let ui = &base;
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
            // 초상화(2배 밀도, docs/decisions/0015). 그림이 없으면 색 네모.
            let portrait = Rect::new(b.x + 4.0, b.y + 3.0, 44.0, 44.0);
            ui.rect(portrait, ui::PANEL_LIGHT);
            let drawn = d
                .portrait
                .is_some_and(|f| self.renderer.draw_portrait(ui, portrait, f));
            if !drawn {
                ui.rect(Rect::new(b.x + 12.0, b.y + 11.0, 28.0, 28.0), d.color);
            }
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
        ui.text_center(i18n::s().cut_skip, 160.0, 1.0, FONT_PX, ui::TEXT_DIM);
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
        let panel = Rect::new(56.0, 28.0, 208.0, 116.0);
        let s = i18n::s();
        ui.panel(panel, Some(s.settings));
        let values = [
            format!("{}%", self.settings.volume * 10),
            format!("{}%", self.settings.shake * 10),
            if self.settings.debug_overlay {
                s.on
            } else {
                s.off
            }
            .to_string(),
            s.lang_name.to_string(),
            String::new(),
        ];
        let row_h = FONT_PX + 6.0;
        for (i, label) in trs(&SETTING_ROWS).into_iter().enumerate() {
            let y = panel.y + 22.0 + i as f32 * row_h;
            let selected = self.settings_cursor == i;
            if i == SETTING_BACK {
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
                self.hits.push((r, Hit::SettingRow(SETTING_BACK)));
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
        for (i, l) in ui::wrap(ui, s.settings_hint, 300.0, FONT_PX)
            .iter()
            .take(2)
            .enumerate()
        {
            ui.text_center(
                l,
                160.0,
                panel.bottom() + 4.0 + i as f32 * 13.0,
                FONT_PX,
                ui::TEXT_DIM,
            );
        }
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
        let r = Rect::new(56.0, 42.0 + (1.0 - k) * 8.0, 208.0, 86.0);
        ui.panel(r, None);
        ui.text_center(i18n::s().gift_title, 160.0, r.y + 5.0, FONT_PX, ui::ACCENT);
        let icon = Rect::new(r.x + 10.0, r.y + 22.0, 22.0, 22.0);
        ui.rect(icon, ui::PANEL_LIGHT);
        ui.rect_lines(icon, ui::BORDER);
        draw_claw_icon(ui, icon, wp.color);
        ui.text(
            tr(wp.name),
            icon.right() + 8.0,
            r.y + 22.0,
            FONT_PX,
            ui::TEXT,
        );
        let lines = ui::wrap(ui, tr(wp.desc), r.right() - icon.right() - 16.0, FONT_PX);
        for (i, l) in lines.iter().take(3).enumerate() {
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
        // 한 줄이 기본. 언어에 따라 길면 두 줄로 위로 자란다.
        let lines = ui::wrap(ui, text, 288.0, FONT_PX);
        let w = lines
            .iter()
            .map(|l| ui.text_w(l, FONT_PX))
            .fold(0.0, f32::max)
            + 16.0;
        let h = lines.len() as f32 * 13.0 + 8.0;
        let r = Rect::new(160.0 - w / 2.0, 178.0 - h - 7.0, w, h);
        ui.rect(r, ui::PANEL);
        ui.rect_lines(r, ui::BORDER);
        for (i, l) in lines.iter().enumerate() {
            ui.text_center(l, 160.0, r.y + 4.0 + i as f32 * 13.0, FONT_PX, ui::TEXT);
        }
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
        let s = i18n::s();
        ui.text_center(
            s.dead,
            160.0,
            64.0,
            FONT_PX * 2.0,
            Color::new(0.9, 0.3, 0.3, k),
        );
        if self.death_t >= t::DEATH_INPUT_LOCK && (get_time() * 2.0) as i64 % 2 == 0 {
            ui.text_center(s.dead_hint, 160.0, 96.0, FONT_PX, ui::TEXT);
        }
    }

    fn draw_pause(&mut self, ui: &Ui) {
        let panel = Rect::new(12.0, 8.0, 296.0, 156.0);
        ui.panel(panel, None);
        let tabs = ui.tabs(
            &trs(&PAUSE_TABS),
            self.pause_tab,
            panel.x + 6.0,
            panel.y + 5.0,
            panel.w - 12.0,
        );
        for (i, r) in tabs.into_iter().enumerate() {
            self.hits.push((r, Hit::PauseTab(i)));
        }
        ui.line(
            vec2(panel.x + 4.0, panel.y + 24.0),
            vec2(panel.right() - 4.0, panel.y + 24.0),
            ui::BORDER,
        );
        // 탭 안내는 패널 밖 오른쪽 아래. 탭 줄에 두면 탭 이름이 긴 언어에서 겹치고, 패널 안에 두면 내용과 겹친다.
        ui.text_right(
            i18n::s().tab_hint,
            panel.right() - 2.0,
            panel.bottom() + 2.0,
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
                    &trs(&PAUSE_MENU),
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
        let hearts: String = (0..p.stats.max_hp)
            .map(|i| if i < p.hp { '♥' } else { '♡' })
            .collect();
        let secs = self.run_time as u32;
        let s = i18n::s();
        let last = &p.moveset.hits[p.moveset.last()];
        let charms = if self.world.charms.is_empty() {
            s.charms_none.to_string()
        } else {
            self.world
                .charms
                .iter()
                .map(|c| tr(CHARMS[*c].name))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let st = p.stats;
        let attack = if st.damage != 0 || (st.attack_speed - 1.0).abs() > 1e-3 {
            format!(
                " · {}",
                fill(
                    s.stat_attack,
                    &[&st.damage, &((st.attack_speed * 100.0).round() as i32)]
                )
            )
        } else {
            String::new()
        };
        let lines = [
            (s.hp, hearts),
            (
                s.this_run,
                fill(
                    s.run_stats,
                    &[
                        &self.world.wave,
                        &self.world.kills,
                        &format!("{:02}:{:02}", secs / 60, secs % 60),
                    ],
                ),
            ),
            match self.inventory.equipped.filter(|_| p.armed) {
                Some(w) => (
                    tr(WEAPONS[w].name),
                    fill(
                        s.combo_stats,
                        &[
                            &p.moveset.hits.len(),
                            &format!("{:.2}", last.swing),
                            &(last.knockback as i32),
                        ],
                    ) + &attack,
                ),
                None => (s.bare_hands, s.bare_hands_desc.to_string()),
            },
            (
                s.burrow,
                fill(
                    s.burrow_stats,
                    &[
                        &format!("{:.2}", t::BURROW_MAX_TIME),
                        &(t::EMERGE_RADIUS as i32),
                        &format!("{:.1}", t::BURROW_COOLDOWN * st.burrow_cd),
                    ],
                ),
            ),
            (s.charms_label, charms),
            (s.glowbugs, self.world.glowbugs.to_string()),
            (s.special, format!("{}: {}", s.special_name, s.special_desc)),
        ];
        let row_h = FONT_PX + 3.0;
        // 이름 칸 폭은 언어마다 다르다(무기 이름이 가장 길다). 가장 긴 이름에 맞춘다.
        let key_w = lines
            .iter()
            .map(|(k, _)| ui.text_w(k, FONT_PX))
            .fold(0.0, f32::max)
            + 8.0;
        // 값이 길면(언어에 따라) 두 줄까지 바꾸고 아래 줄을 민다. 그보다 길면 자른다.
        let mut y = c.y;
        for (k, v) in lines.iter() {
            // 칸을 넘으면 그리지 않는다(아래는 탭 안내 자리).
            if y + FONT_PX > c.bottom() {
                break;
            }
            ui.text(&ui.fit(k, 72.0, FONT_PX), c.x, y, FONT_PX, ui::ACCENT);
            let vx = c.x + key_w.min(80.0);
            let vw = c.right() - vx;
            let wrapped = ui::wrap(ui, v, vw, FONT_PX);
            for (j, l) in wrapped.iter().take(2).enumerate() {
                if j > 0 {
                    y += 12.0;
                }
                let l = if j == 1 && wrapped.len() > 2 {
                    ui.fit(&format!("{l}…"), vw, FONT_PX)
                } else {
                    l.clone()
                };
                ui.text(&l, vx, y, FONT_PX, ui::TEXT);
            }
            y += row_h;
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
            match item {
                Some(Item::Weapon(w)) => {
                    draw_claw_icon(ui, r, WEAPONS[*w].color);
                    if self.inventory.equipped == Some(*w) {
                        ui.text("E", r.x + 2.0, r.y + 1.0, FONT_PX, ui::ACCENT);
                    }
                }
                Some(Item::Charm(c)) => {
                    draw_charm_icon(ui, r, CHARMS[*c].color);
                    if self.world.charms.contains(c) {
                        ui.text("E", r.x + 2.0, r.y + 1.0, FONT_PX, ui::ACCENT);
                    }
                }
                None => {}
            }
            self.hits.push((r, Hit::InvSlot(i)));
        }
        let s = i18n::s();
        let desc_x = x0 + cols as f32 * (slot + gap) + 8.0;
        let desc_w = c.right() - desc_x;
        match &self.inventory.slots[self.inv_cursor] {
            Some(Item::Weapon(w)) => {
                let wp = &WEAPONS[*w];
                ui.text(tr(wp.name), desc_x, y0, FONT_PX, ui::ACCENT);
                let mut y = y0 + 16.0;
                for l in ui::wrap(ui, tr(wp.desc), desc_w, FONT_PX).iter() {
                    ui.text(l, desc_x, y, FONT_PX, ui::TEXT);
                    y += 14.0;
                }
                let ms = wp.moveset;
                let last = &ms.hits[ms.last()];
                ui.text(
                    &fill(s.bag_combo, &[&ms.hits.len(), &last.damage]),
                    desc_x,
                    y + 4.0,
                    FONT_PX,
                    ui::TEXT_DIM,
                );
                let hint = if self.inventory.equipped == Some(*w) {
                    s.equipped
                } else {
                    s.equip_hint
                };
                ui.text(hint, desc_x, y + 18.0, FONT_PX, ui::ACCENT);
            }
            Some(Item::Charm(c)) => {
                let ch = &CHARMS[*c];
                ui.text(tr(ch.name), desc_x, y0, FONT_PX, ui::ACCENT);
                let mut y = y0 + 16.0;
                for l in ui::wrap(ui, tr(ch.desc), desc_w, FONT_PX).iter() {
                    ui.text(l, desc_x, y, FONT_PX, ui::TEXT);
                    y += 14.0;
                }
                ui.text(
                    &fill(s.charm_slots, &[&self.world.charms.len(), &CHARM_SLOTS]),
                    desc_x,
                    y + 4.0,
                    FONT_PX,
                    ui::TEXT_DIM,
                );
                let hint = if self.world.charms.contains(c) {
                    s.charm_unequip
                } else {
                    s.charm_equip
                };
                ui.text(hint, desc_x, y + 18.0, FONT_PX, ui::ACCENT);
            }
            None => {
                ui.text(s.empty_slot, desc_x, y0, FONT_PX, ui::TEXT_DIM);
                for (i, l) in ui::wrap(ui, s.empty_slot_desc, desc_w, FONT_PX)
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
            &fill(
                s.slots,
                &[
                    &self.inventory.slots.iter().filter(|x| x.is_some()).count(),
                    &self.inventory.slots.len(),
                ],
            ),
            x0,
            y0 + 3.0 * (slot + gap) + 2.0,
            FONT_PX,
            ui::TEXT_DIM,
        );
        // 퀵 아이템(가방 칸과 따로, HUD 왼쪽 아래와 같은 것)
        let bag = &self.world.bag;
        ui.text(
            &ui.fit(
                &fill(
                    s.bag_items,
                    &[&bag.berries, &t::BERRY_MAX, &bag.worms, &t::WORM_MAX],
                ),
                desc_x - x0 - 4.0,
                FONT_PX,
            ),
            x0,
            y0 + 3.0 * (slot + gap) + 16.0,
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
            let title = ui.fit(tr(q.title), r.w - 46.0, FONT_PX);
            ui.text(&title, r.x + 16.0, r.y + 2.0, FONT_PX, col);
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
        ui.text(tr(q.title), dx, c.y, FONT_PX, ui::ACCENT);
        for (i, l) in ui::wrap(ui, tr(q.desc), dw, FONT_PX).iter().enumerate() {
            ui.text(l, dx, c.y + 16.0 + i as f32 * 14.0, FONT_PX, ui::TEXT);
        }
    }

    fn draw_map(&self, ui: &Ui, c: Rect) {
        let s = i18n::s();
        let proto_w = ui.text_w(s.map_proto, FONT_PX);
        ui.text(
            &ui.fit(
                &fill(s.map_floor, &[&tr(self.world.biome.name())]),
                c.w - proto_w - 8.0,
                FONT_PX,
            ),
            c.x,
            c.y,
            FONT_PX,
            ui::ACCENT,
        );
        ui.text_right(s.map_proto, c.right(), c.y, FONT_PX, ui::TEXT_DIM);
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
        // 한 줄만(아래는 탭 안내 자리). 길면 자른다.
        ui.text(
            &ui.fit(s.map_note, c.w, FONT_PX),
            c.x,
            origin.y + t::VIEW_H * k + 4.0,
            FONT_PX,
            ui::TEXT_DIM,
        );
    }
}

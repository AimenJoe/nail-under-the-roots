//! 장면 상태 기계와 메뉴. 인트로 → 타이틀 → (설정) → 게임 ⇄ 일시정지(메뉴·상태·가방·저널·지도).
//!
//! 메뉴는 커서 없이 동작한다(상하좌우 + 확인 + 뒤로, 탭은 Q/E·L1/R1). 탭/클릭은 보조로,
//! 직전 프레임에 그린 요소의 사각형(`hits`)에 맞춰 같은 동작을 한다. (CLAUDE.md "UI는 커서 없이")
//! 아이템·퀘스트·층 구조는 아직 데이터가 없다. 틀만 있고 내용은 다음 단계에서 들어온다(docs/decisions/0007).

use macroquad::prelude::*;

mod dream;
mod pick;
mod stage;

use crate::art::Art;
use crate::cutscene::{self, Cutscene};
use crate::equipment::{Character, CHARMS, CHARM_SLOTS, WEAPONS};
use crate::i18n::{self, fill, tr, Tr};
use crate::input::{self, Actions, Device, Input, Verb};
use crate::render::{self, Renderer};
use crate::scenario::{self, Scenario};
use crate::sfx::Sfx;
use crate::tuning as t;
use crate::tutorial::Tutorial;
use crate::ui::{self, Ui, FONT_PX};
use crate::world::World;

const INTRO_LINES: [Tr; 3] = [|s| s.intro_1, |s| s.intro_2, |s| s.intro_3];
const INTRO_LINE_TIME: f32 = 2.6;
/// 타이틀 메뉴 줄. 오늘의 굴은 튜토리얼을 마친 뒤에만 보인다.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TitleItem {
    Start,
    Daily,
    Album,
    Settings,
    #[cfg(feature = "dev")]
    Dev,
}
/// 앨범 칸 배치(가로 칸 수).
const ALBUM_COLS: usize = 5;
const PAUSE_TABS: [Tr; 6] = [
    |s| s.tab_menu,
    |s| s.tab_status,
    |s| s.tab_bag,
    |s| s.tab_journal,
    |s| s.tab_bestiary,
    |s| s.tab_map,
];
/// 지도 탭 번호(지도 버튼으로 바로 연다).
const TAB_MAP: usize = 5;
const PAUSE_MENU: [Tr; 4] = [
    |s| s.pm_resume,
    |s| s.pm_restart,
    |s| s.pm_settings,
    |s| s.pm_title,
];
const SETTING_ROWS: [Tr; 8] = [
    |s| s.set_sound,
    |s| s.set_shake,
    |s| s.set_debug,
    |s| s.set_language,
    |s| s.set_assist,
    |s| s.set_controls,
    |s| s.set_erase,
    |s| s.set_back,
];
/// 설정의 "포포의 응원"·"조작 설정"·"기록 지우기"·"돌아가기" 줄.
const SETTING_ASSIST: usize = 4;
const SETTING_CONTROLS: usize = 5;
const SETTING_ERASE: usize = 6;
const SETTING_BACK: usize = 7;
/// 값이 아니라 누르는 줄(좌우로 바뀌지 않는다).
fn setting_is_button(i: usize) -> bool {
    matches!(i, SETTING_CONTROLS | SETTING_ERASE | SETTING_BACK)
}
/// 조작 설정의 줄: 동작들(input::ACTS) + 기본값 + 돌아가기.
const CONTROL_NAMES: [Tr; 7] = [
    |s| s.act_attack,
    |s| s.act_dodge,
    |s| s.act_special,
    |s| s.act_use_item,
    |s| s.act_item_prev,
    |s| s.act_item_next,
    |s| s.act_map,
];
const CONTROL_RESET: usize = 7;
const CONTROL_BACK: usize = 8;
/// 다시 붙이기: 이만큼 아무것도 안 누르면 그만둔다.
const REBIND_TIMEOUT: f32 = 5.0;

/// 안내 글의 동사 자리(`{0}`, `{1}` …)를 지금 장치(키보드·패드·터치)의 이름으로 채운다.
fn vfill(t: &str, verbs: &[Verb]) -> String {
    let names: Vec<&str> = verbs.iter().map(|v| input::verb(*v)).collect();
    let args: Vec<&dyn std::fmt::Display> =
        names.iter().map(|n| n as &dyn std::fmt::Display).collect();
    fill(t, &args)
}

/// 터치 전용 글이 있으면 터치일 때 그것, 아니면 동사를 채운 글.
fn device_hint(t: &str, touch: &str, verbs: &[Verb]) -> String {
    if input::device() == Device::Touch {
        touch.to_string()
    } else {
        vfill(t, verbs)
    }
}

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
    /// 조작 설정(설정 안). 돌아가면 설정.
    Controls {
        back: Back,
    },
    /// 개발용 시나리오 목록.
    Dev,
    /// 컷씬. 끝나면 게임.
    Cutscene,
    Game,
    Pause,
    /// 길 지도(쉼터의 뿌리 길, 굴의 출구, docs/decisions/0031).
    Route,
    /// 포포의 가게(쉼터, docs/decisions/0030).
    Shop,
    /// 앨범: 본 컷씬 일러스트를 다시 본다(타이틀, docs/decisions/0041).
    Album,
    /// 실뜨기 단계 고르기(마을의 누룩 할매, docs/decisions/0043·0045).
    KnotPick,
    /// 마을 꾸미기 가게(마을의 포포, docs/decisions/0045).
    Decor,
    /// 큰 나무의 재주: 반딧불로 깨우고 고른다(마을, docs/decisions/0047).
    Tree,
    /// 실의 꿈 고르기(마을의 실 고리, docs/decisions/0055).
    DreamPick,
    /// 누구와 내려갈까(마을의 뿌리 길, 모모가 돌아온 뒤, docs/decisions/0056).
    CharPick,
}

/// 컷씬이 끝난 뒤 갈 곳.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AfterCut {
    Game,
    Title,
    Village,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Back {
    Title,
    Pause,
}

/// 설정. 값은 0..=10 단계. 웹에서는 브라우저에 저장한다(prefs.rs). 디버그 정보는 저장하지 않는다.
pub struct Settings {
    pub volume: u8,
    pub shake: u8,
    pub debug_overlay: bool,
    /// 사람이 고른 언어(None = 기기 언어).
    pub lang: Option<i18n::Lang>,
}

impl Settings {
    fn volume_f(&self) -> f32 {
        self.volume as f32 / 10.0
    }
    fn shake_f(&self) -> f32 {
        self.shake as f32 / 10.0
    }
}

/// 런 통계를 기록에 쌓을 때의 기준(저널, docs/decisions/0048). 월드가 새로 만들어지면 0부터.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct Tally([u32; 9]);

impl Tally {
    const KEYS: [&'static str; 9] = [
        crate::journal::stat::KILLS,
        crate::journal::stat::EMERGE_KILLS,
        crate::journal::stat::PARRIES,
        crate::journal::stat::DRILLS,
        crate::journal::stat::SLAMS,
        crate::journal::stat::FINISHERS,
        crate::journal::stat::EATEN,
        crate::journal::stat::CHESTS,
        crate::journal::stat::SPRINGS,
    ];

    fn of(w: &World) -> Self {
        Self([
            w.kills,
            w.emerge_kills,
            w.parries,
            w.drills,
            w.slams,
            w.finisher_hits,
            w.items_used,
            w.chests,
            w.springs,
        ])
    }
}

pub enum Item {
    /// equipment::WEAPONS 번호.
    Weapon(usize),
    /// equipment::CHARMS 번호. 끼고 있는지는 world.charms.
    Charm(usize),
}

/// 독 발톱(equipment::WEAPONS 번호, 큰 나무에서 깨운다, 0052).
pub(crate) const VENOM_CLAW: usize = 3;
/// 새싹 발톱(수수가 만들어 준다, 0053).
pub(crate) const SPROUT_CLAW: usize = 0;
/// 바위 발톱(콩이가 준다, 0054).
pub(crate) const STONE_CLAW: usize = 2;

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
    item: crate::world::Gift,
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
#[derive(Clone, Copy, PartialEq, Eq)]
enum Hit {
    TitleItem(usize),
    PauseTab(usize),
    PauseItem(usize),
    SettingRow(usize),
    SettingLeft(usize),
    SettingRight(usize),
    InvSlot(usize),
    /// 조작 설정 칸(줄, 열).
    ControlCell(usize, usize),
    JournalItem(usize),
    BestiaryItem(usize),
    /// 도감 설명(누르면 다음 쪽).
    BestiaryPage,
    DevItem(usize),
    /// 길 지도: 고를 수 있는 칸(choices 번호).
    RouteNode(usize),
    ShopItem(usize),
    /// 사기 전에 묻는 창의 단추.
    ShopYes,
    ShopNo,
    /// 닫기 단추·패널 바깥(가게·길 지도).
    Close,
    /// 컷씬 건너뛰기 안내.
    Skip,
    /// 앨범 칸.
    AlbumItem(usize),
    /// 실뜨기: 단계 내리기·올리기·시작.
    KnotLeft,
    KnotRight,
    KnotGo,
    /// 큰 나무의 재주 칸(자리, 갈래).
    TreeCell(usize, usize),
    /// 실의 꿈 줄.
    DreamRow(usize),
    /// 캐릭터 고르기 칸.
    CharCard(usize),
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
    bestiary_cursor: usize,
    /// 도감 설명 쪽(0 이야기 · 1 공격 · 2 요령). 칸마다 두 줄로 자르던 것을 쪽으로 나눴다(0051).
    bestiary_page: usize,
    /// 조작 설정: 줄, 열(0 키보드 · 1 패드), 다시 붙이는 중이면 남은 시간.
    controls_cursor: usize,
    controls_col: usize,
    rebind: Option<f32>,
    dev_cursor: usize,
    /// 지금 플레이 중인 시나리오. "다시 시작"은 이걸 다시 만든다.
    current: Scenario,
    cutscene: Option<Cutscene>,
    tutorial: Option<Tutorial>,
    gift: Option<Gift>,
    /// 아직 창을 못 띄운 획득(여러 개가 한꺼번에 들어올 때).
    gift_queue: Vec<crate::world::Gift>,
    pub settings: Settings,
    world: World,
    /// 저널: 런 통계 기준, 받을 수 있는 수(새로 생기면 알림), 목록 스크롤, 다시 세기까지(0048).
    tally: Tally,
    journal_ready: usize,
    journal_scroll: usize,
    journal_check: f32,
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
    /// 길 지도: 고른 칸, 뒤로 닫을 수 있나(쉼터에서 열었을 때만).
    route_cursor: usize,
    route_cancel: bool,
    /// 가게: 고른 줄, 방금 한 일(글, 남은 시간).
    shop_cursor: usize,
    shop_msg: Option<(Tr, f32)>,
    /// 사기 전에 묻는 중: (물건 줄, "사기"에 커서가 있나).
    shop_ask: Option<(usize, bool)>,
    /// 보리에게 말 건 횟수(말이 돌아간다).
    talk_n: u32,
    /// 런을 넘어 남는 기록(docs/decisions/0041).
    profile: crate::profile::Profile,
    /// 이번 쓰러짐을 이미 셌나.
    death_noted: bool,
    /// 깨어나는 장면(어두워졌다 밝아짐)이 끝나면 한마디.
    pending_wake: bool,
    /// 설정 "기록 지우기"를 한 번 눌렀다(한 번 더 누르면 지운다).
    erase_armed: bool,
    /// 이 컷씬이 끝나면 갈 곳(엔딩 = 런의 끝).
    after_cutscene: AfterCut,
    /// 마을 꾸미기 가게의 커서.
    decor_cursor: usize,
    /// 재주 창 커서(자리, 갈래).
    tree_cursor: (usize, usize),
    /// 실의 꿈(0055): 고르기 창의 줄, 꾸고 있는 꿈.
    dream_cursor: usize,
    dream: Option<dream::DreamRun>,
    /// 꿈에서 깬 직후 고르기 창이 입력을 받지 않는 남은 시간(싸우던 연타가 다음 꿈을 열지 않게).
    dream_lock: f32,
    /// 캐릭터 고르기 창의 칸(0 = Naïl, 1 = 모모).
    char_cursor: usize,
    /// 이번 런의 실뜨기 단계(고르는 화면의 커서이기도 하다).
    knot_level: u8,
    /// 타이틀 아래 잠깐 띄우는 글(오늘의 굴 완주 등)과 남은 시간.
    title_note: Option<(Tr, f32)>,
    /// 앨범: 고른 칸, 크게 보는 중이면 흐른 시간.
    album_cursor: usize,
    album_view: Option<f32>,
}

impl App {
    pub fn new(sfx: Sfx, fonts: render::Fonts, art: Option<Art>) -> Self {
        let saved = crate::prefs::load();
        let settings = Settings {
            volume: saved.map(|p| p.volume).unwrap_or(7),
            shake: saved.map(|p| p.shake).unwrap_or(10),
            debug_overlay: false,
            lang: saved.and_then(|p| p.lang),
        };
        if let Some(p) = saved {
            crate::input::set_bindings(p.bindings);
        }
        if let Some(l) = settings.lang {
            i18n::set_lang(l);
        }
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
            bestiary_cursor: 0,
            bestiary_page: 0,
            controls_cursor: 0,
            controls_col: 0,
            rebind: None,
            dev_cursor: 0,
            current: Scenario::Normal,
            cutscene: None,
            tutorial: None,
            gift_queue: Vec::new(),
            settings,
            world,
            tally: Tally::default(),
            journal_ready: 0,
            journal_scroll: 0,
            journal_check: 0.0,
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
            route_cursor: 0,
            route_cancel: false,
            shop_cursor: 0,
            shop_msg: None,
            shop_ask: None,
            talk_n: 0,
            profile: crate::profile::load(),
            death_noted: false,
            pending_wake: false,
            erase_armed: false,
            album_cursor: 0,
            album_view: None,
            after_cutscene: AfterCut::Game,
            decor_cursor: 0,
            tree_cursor: (0, 0),
            dream_cursor: 0,
            dream: None,
            dream_lock: 0.0,
            char_cursor: 0,
            knot_level: 0,
            title_note: None,
        }
    }

    /// 기록을 저장한다. 이번 런에서 만난 이웃도 함께 남긴다.
    fn save_profile(&mut self) {
        for k in &self.world.seen {
            self.profile.set(&crate::profile::key::mob(k));
        }
        crate::profile::save(&self.profile);
    }

    /// 첫 시작(Normal)만 이야기 기록을 남긴다. 개발 시나리오는 기록을 건드리지 않는다.
    fn recording(&self) -> bool {
        self.current == Scenario::Normal
    }

    /// 도움 모드 "포포의 응원": 켜져 있으면 20%에서 시작해 쓰러질 때마다 2%씩, 최대 80%(Hades God Mode를 참고, 0041).
    fn assist_resist(&self) -> f32 {
        use crate::profile::key;
        if self.profile.flag(key::ASSIST) {
            (0.2 + 0.02 * self.profile.count(key::ASSIST_DEATHS) as f32).min(0.8)
        } else {
            0.0
        }
    }

    fn apply_assist(&mut self) {
        self.world.player.resist = self.assist_resist();
    }

    /// 짧은 한마디(런 사이). 맞는 줄이 있으면 대화 컷씬.
    /// 표에서 지금 할 한마디를 골라 튼다. 말했으면 true.
    fn say_story(&mut self, when: crate::story::When) -> bool {
        let Some(l) = crate::story::take(&mut self.profile, when) else {
            return false;
        };
        crate::profile::save(&self.profile);
        let c = cutscene::talk(&self.world, &[(l.who, l.text, cutscene::Face::Happy)]);
        if self.cutscene.is_none() {
            self.cutscene = Some(c);
            self.scene = Scene::Cutscene;
        }
        true
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
                let mut a = self.input.take();
                // 건너뛰기 안내를 누르면 뒤로와 같다(터치).
                if a.tap.is_some_and(|p| self.hit_at(p) == Some(Hit::Skip)) {
                    a.cancel = true;
                    a.tap = None;
                }
                if let Some(c) = self.cutscene.as_mut() {
                    c.update(&a, dt, &mut self.world);
                    // 본 일러스트는 앨범에 남는다(개발 시나리오에서 봐도).
                    if let Some(il) = c.illust {
                        if self
                            .profile
                            .set(&crate::profile::key::illust(il.spec().file))
                        {
                            crate::profile::save(&self.profile);
                        }
                    }
                    // 컷씬에서 받은 무기는 컷씬이 끝난 뒤 창으로 보여 준다.
                    self.gift_queue.append(&mut self.world.gifts);
                    if c.is_done() {
                        self.cutscene = None;
                        match std::mem::replace(&mut self.after_cutscene, AfterCut::Game) {
                            AfterCut::Game => self.scene = Scene::Game,
                            AfterCut::Title => {
                                self.title_cursor = 0;
                                self.scene = Scene::Title;
                            }
                            AfterCut::Village => self.start(Scenario::Village),
                        }
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
        let mut open_map = false;
        let mut restart = false;
        let mut interact = None;
        while self.acc >= t::TICK_DT {
            let mut a = self.input.take();
            // 쉼터: 말 걸 것 곁에서 공격 버튼 = 말 걸기(휘두르지 않는다).
            if a.attack && interact.is_none() {
                if let Some(i) = self.world.interactable() {
                    a.attack = false;
                    interact = Some(i);
                }
            }
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
            if a.map {
                open_map = true;
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
            self.death_noted = false;
        } else {
            self.death_t += dt;
            if !self.death_noted {
                self.death_noted = true;
                self.on_death();
            }
        }
        // 깨어나는 어두워짐·밝아짐이 끝나면 이웃의 한마디(갈색 뿌리가 데려왔다, 0037).
        if self.pending_wake
            && self.world.adv.fade.is_none()
            && self.world.player.is_alive()
            && self.scene == Scene::Game
        {
            self.pending_wake = false;
            self.say_story(crate::story::When::Wake);
        }
        self.take_gifts();
        if let Some(tu) = self.tutorial.as_mut() {
            tu.update(&mut self.world, dt);
            // 포포가 할 말이 생기면 대화 컷씬(월드는 멈춘다).
            let talk = tu.take_talk();
            if !talk.is_empty() && self.cutscene.is_none() && self.world.player.is_alive() {
                self.cutscene = Some(cutscene::popo_talk(&self.world, &talk));
                self.scene = Scene::Cutscene;
            }
            if tu.is_done() {
                self.tutorial = None;
                if self.recording() && self.profile.set(crate::profile::key::TUTORIAL) {
                    self.save_profile();
                }
            }
        }
        self.sync_journal(t::TICK_DT);
        self.take_adventure_events();
        self.tick_dream(dt);
        if let Some(i) = interact {
            if self.scene == Scene::Game {
                self.interact(i);
            }
        }
        if self.scene != Scene::Game {
            return;
        }
        // 꿈에서 쓰러지면 잠깐 뒤 마을로 깬다(tick_dream). 재시작 입력은 받지 않는다.
        if restart && self.dream.is_none() {
            // 모험(필드·던전)에서 쓰러지면 저장 지점으로. 아레나·개발 시나리오는 처음부터.
            if self.world.adv.place != crate::adventure::Place::Arena
                && !self.world.player.is_alive()
            {
                self.world.begin_respawn();
                self.death_t = 0.0;
                self.pending_wake = self.recording();
            } else {
                self.start_game();
            }
        } else if open_pause {
            self.open_pause();
        } else if open_map {
            self.open_pause();
            self.pause_tab = TAB_MAP;
        }
    }

    /// 쓰러졌다: 센다(이야기 조건·도움 모드), 기록을 저장한다.
    fn on_death(&mut self) {
        if !self.recording() {
            return;
        }
        use crate::profile::key;
        self.profile.add(key::DEATHS, 1);
        if self.profile.flag(key::ASSIST) {
            self.profile.add(key::ASSIST_DEATHS, 1);
        }
        self.save_profile();
        self.apply_assist();
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
        if s != Scenario::Dream {
            self.dream = None;
        }
        self.current = s;
        self.world = scenario::build(s);
        if s == Scenario::Village {
            self.world = World::village(self.village_state());
        }
        self.world.debug.overlay = self.settings.debug_overlay;
        self.world.debug.hitboxes = hitboxes;
        self.journal_cursor = 0;
        self.journal_scroll = 0;
        // 캐릭터(0056): 런(첫 시작·오늘의 굴·실의 꿈)은 뿌리 길에서 고른 캐릭터, 그 밖(마을·개발)은 Naïl.
        // 고른 캐릭터(0056): 보통 런·오늘의 굴, 개발의 가장 깊은 곳(모모 끝 장면 보기).
        // 실의 꿈은 Naïl만: 꿈 목록에 모모가 있다.
        let ch = if matches!(s, Scenario::Normal | Scenario::Daily | Scenario::Deepest) {
            self.picked()
        } else {
            Character::Mole
        };
        self.world.character = ch;
        self.world.player.kit = ch;
        // 무기는 스토리를 따라 하나씩(docs/decisions/0012). 첫 시작·튜토리얼은 맨손에서 출발해
        // 튜토리얼에서 두더지 발톱을 줍는다. 개발 시나리오는 테스트를 위해 기본 발톱(또는 전부)을 들고 시작.
        let start = ch.starting_weapon();
        // 튜토리얼을 마친 기록이 있으면 오프닝·튜토리얼을 건너뛰고 발톱을 든 채 시작한다(0041).
        let veteran = s == Scenario::Normal && self.profile.flag(crate::profile::key::TUTORIAL);
        if s == Scenario::Normal {
            self.profile.add(crate::profile::key::RUNS, 1);
            crate::profile::save(&self.profile);
        }
        if s == Scenario::Daily {
            // 한 달 넘은 오늘의 굴 기록은 지운다(기록이 끝없이 늘지 않게).
            let today = crate::daily::today();
            self.profile
                .drop_counts(|k| crate::daily::is_stale_key(k, today));
            // 첫 쉼터에서 시작하니 시작만 해도 쉼터 1.
            self.profile.raise(&crate::daily::key_best(today), 1);
            crate::profile::save(&self.profile);
        }
        // 도감은 런을 넘어 남는다.
        for k in crate::species::BESTIARY {
            if self.profile.flag(&crate::profile::key::mob(k)) && !self.world.seen.contains(&k) {
                self.world.seen.push(k);
            }
        }
        self.death_noted = false;
        self.pending_wake = false;
        self.apply_assist();
        // 실뜨기는 첫 시작에만(개발 시나리오는 0단계).
        let knots = crate::knots::Knots::new(if s == Scenario::Normal {
            self.knot_level.min(self.knots_open())
        } else {
            0
        });
        self.world.knots = knots;
        self.world.player.eat_cd_mul = knots.eat_cd_mul();
        self.world.player.parry_mul = knots.parry_mul();
        self.world.player.burrow_knot = knots.burrow_cd_mul();
        // 저널 보상(0048): 상자에서 나올 장신구, 털빛. 통계 기준은 새 월드에서 0부터.
        self.world.charm_pool = crate::journal::charm_pool(&self.profile);
        self.world.fur = crate::journal::fur(&self.profile);
        self.tally = Tally::of(&self.world);
        self.journal_ready = crate::journal::ready_count(&self.profile);
        // 큰 나무의 재주(0047): 튜토리얼만 빼고 모든 시작에(개발 시나리오도 고른 재주로 시험한다).
        if s != Scenario::Tutorial {
            self.world.player.talents = self.talents();
        }
        // 모모(0046): 이긴 수는 기록에서(빈 자리의 꽃잎). 첫 길 끝에서 싸우는 건 첫 시작에서만, 돌아오기 전까지.
        // 개발 시나리오 "모모"는 시나리오가 정한 값을 그대로 쓴다.
        if matches!(s, Scenario::Normal | Scenario::Village) {
            use crate::profile::key;
            self.world.momo_wins = self.profile.count(key::MOMO_WINS).min(t::MOMO_FIGHTS) as u8;
            // 모모로 내려가면 모모 보스는 없다(0056).
            self.world.momo_fight = s == Scenario::Normal
                && !self.profile.flag(key::MOMO_BACK)
                && ch != Character::Momo;
            // 도토(0052)도 첫 시작에서만, 기억해 내기 전까지.
            self.world.doto_fight = s == Scenario::Normal && !self.profile.flag(key::DOTO_MEMORY);
            // 콩이(0054)도 첫 시작에서만, 이기기 전까지.
            self.world.kongi_fight = s == Scenario::Normal && !self.profile.flag(key::KONGI_BACK);
        }
        self.inventory = match s {
            Scenario::Normal if veteran => Inventory::new(&[start]),
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
        // 수수의 새싹 발톱(0053)·큰 나무에서 깨운 독 발톱(0052)도 런마다 가방에(끼는 건 가방에서).
        // 맨손 시작(튜토리얼 전)은 빼고.
        for (k, w) in [
            (crate::profile::key::SPROUT, SPROUT_CLAW),
            (crate::profile::key::STONE, STONE_CLAW),
            (crate::profile::key::VENOM, VENOM_CLAW),
        ] {
            if self.profile.flag(k)
                && self.inventory.equipped.is_some()
                && WEAPONS[w].kind == ch.weapon_kind()
            {
                self.inventory.add(w);
            }
        }
        // 저널로 받은 장신구는 런마다 가방에 들고 시작한다(0050). 상자에서는 다시 안 나온다.
        if s != Scenario::Tutorial {
            for c in crate::journal::owned_charms(&self.profile) {
                self.inventory.add_charm(c);
                if !self.world.charms_found.contains(&c) {
                    self.world.charms_found.push(c);
                }
            }
        }
        self.gift = None;
        self.gift_queue.clear();
        self.apply_equipped();
        self.run_time = 0.0;
        self.acc = 0.0;
        self.death_t = 0.0;
        self.tutorial = None;
        // 첫 시작은 컷씬 → 플레이 튜토리얼 → 웨이브 1. 튜토리얼이 끝나야 자동 웨이브가 켜진다.
        if s == Scenario::Village {
            self.scene = Scene::Game;
            self.say_story(crate::story::When::Village);
            return;
        }
        if veteran {
            self.world.open_glade();
            if self.profile.flag(crate::profile::key::BORI_BACK) {
                self.world.skip_glade_boss();
            }
            self.scene = Scene::Game;
            self.say_story(crate::story::When::RunStart);
            return;
        }
        if matches!(s, Scenario::Normal | Scenario::Tutorial) {
            self.tutorial = Some(Tutorial::new(&mut self.world));
        }
        if matches!(s, Scenario::Normal | Scenario::CutsceneOnly) {
            // 아레나(컷씬만)는 비우고, 필드(첫 시작)는 멀리 자는 적을 둔다. 컷씬 좌표는 왼쪽 위 화면 기준.
            if self.world.adv.place == crate::adventure::Place::Arena {
                self.world.enemies.clear();
            }
            self.world.wave = 0;
            self.world.player.pos = vec2(40.0, 90.0);
            self.world.snap_cam();
            self.cutscene = Some(cutscene::opening(&self.world));
            self.scene = Scene::Cutscene;
        } else if s == Scenario::Deepest {
            self.reach_deepest();
        } else if s == Scenario::Dream {
            self.begin_dream();
            self.scene = Scene::Game;
        } else if s == Scenario::StoryPreview {
            self.world.wave = 0;
            self.cutscene = Some(cutscene::story_preview(&self.world));
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
                let items = self.title_items();
                let n = items.len();
                self.title_cursor = self.title_cursor.min(n - 1);
                if let Some((_, t)) = self.title_note.as_mut() {
                    *t -= dt;
                    if *t <= 0.0 {
                        self.title_note = None;
                    }
                }
                if a.up {
                    self.title_cursor = (self.title_cursor + n - 1) % n;
                }
                if a.down {
                    self.title_cursor = (self.title_cursor + 1) % n;
                }
                let chosen = if a.confirm {
                    Some(self.title_cursor)
                } else if let Some(Hit::TitleItem(i)) = tap_hit {
                    Some(i)
                } else {
                    None
                };
                match chosen.and_then(|i| items.get(i).copied()) {
                    Some(TitleItem::Start) => self.begin_run(),
                    Some(TitleItem::Daily) => {
                        self.knot_level = 0;
                        self.start(Scenario::Daily);
                    }
                    Some(TitleItem::Album) => {
                        self.album_cursor = 0;
                        self.album_view = None;
                        self.scene = Scene::Album;
                    }
                    Some(TitleItem::Settings) => {
                        self.settings_cursor = 0;
                        self.scene = Scene::Settings { back: Back::Title };
                    }
                    #[cfg(feature = "dev")]
                    Some(TitleItem::Dev) => {
                        self.dev_cursor = 0;
                        self.scene = Scene::Dev;
                    }
                    None => {}
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
                    self.erase_armed = false;
                }
                if a.down {
                    self.settings_cursor = (self.settings_cursor + 1) % n;
                    self.erase_armed = false;
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
                        adjust = if setting_is_button(i) { 0 } else { 1 };
                    }
                    _ => {}
                }
                if adjust != 0 && !setting_is_button(row) {
                    self.settings_cursor = row;
                    self.adjust_setting(row, adjust);
                }
                let open_controls = (a.confirm && self.settings_cursor == SETTING_CONTROLS)
                    || matches!(tap_hit, Some(Hit::SettingRow(SETTING_CONTROLS)));
                if open_controls {
                    self.settings_cursor = SETTING_CONTROLS;
                    self.controls_cursor = 0;
                    self.rebind = None;
                    self.scene = Scene::Controls { back };
                    return;
                }
                // 기록 지우기: 한 번 누르면 묻고, 한 번 더 누르면 지운다.
                let erase = (a.confirm && self.settings_cursor == SETTING_ERASE)
                    || matches!(tap_hit, Some(Hit::SettingRow(SETTING_ERASE)));
                if erase {
                    self.settings_cursor = SETTING_ERASE;
                    if self.erase_armed {
                        self.erase_armed = false;
                        self.profile = crate::profile::Profile::default();
                        crate::profile::save(&self.profile);
                        self.apply_assist();
                        self.sfx.play(crate::sfx::Id::Hurt);
                    } else {
                        self.erase_armed = true;
                    }
                }
                let back_chosen = a.cancel
                    || (a.confirm && self.settings_cursor == SETTING_BACK)
                    || matches!(tap_hit, Some(Hit::SettingRow(SETTING_BACK)));
                if back_chosen {
                    self.erase_armed = false;
                    self.save_prefs();
                    self.scene = match back {
                        Back::Title => Scene::Title,
                        Back::Pause => Scene::Pause,
                    };
                }
            }
            Scene::Controls { back } => self.update_controls(a, tap_hit, back, dt),
            Scene::Pause => self.update_pause(a, tap_hit),
            Scene::Route => self.update_route(a, tap_hit),
            Scene::Shop => self.update_shop(a, tap_hit, dt),
            Scene::Album => self.update_album(a, tap_hit, dt),
            Scene::KnotPick => self.update_knot_pick(a, tap_hit),
            Scene::Decor => self.update_decor(a, tap_hit, dt),
            Scene::Tree => self.update_tree(a, tap_hit, dt),
            Scene::DreamPick => self.update_dream_pick(a, tap_hit, dt),
            Scene::CharPick => self.update_char_pick(a, tap_hit),
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
            3 => {
                let l = i18n::lang().cycle(d);
                i18n::set_lang(l);
                self.settings.lang = Some(l);
            }
            SETTING_ASSIST => {
                use crate::profile::key;
                if !self.profile.unset(key::ASSIST) {
                    self.profile.set(key::ASSIST);
                }
                crate::profile::save(&self.profile);
                self.apply_assist();
            }
            _ => {}
        }
    }

    fn save_prefs(&self) {
        crate::prefs::save(&crate::prefs::Prefs {
            volume: self.settings.volume,
            shake: self.settings.shake,
            lang: self.settings.lang,
            bindings: crate::input::bindings(),
        });
    }

    /// 조작 설정: 위아래로 동작, 좌우로 키보드/패드 칸, 확인으로 다시 붙이기(다음에 누르는 키·버튼).
    fn update_controls(&mut self, a: &Actions, tap_hit: Option<Hit>, back: Back, dt: f32) {
        use crate::input::{self, Captured, ACTS};
        if let Some(left) = self.rebind {
            // 기다리는 중: 다른 입력은 무시. Esc로 그만두기, 시간이 다 돼도 그만.
            let left = left - dt;
            if a.menu {
                self.rebind = None;
                return;
            }
            if let Some(c) = self.input.capture() {
                let mut b = input::bindings();
                let act = ACTS[self.controls_cursor];
                let ok = match c {
                    Captured::Key(k) if self.controls_col == 0 => b.bind_key(act, k),
                    Captured::Pad(p) if self.controls_col == 1 => b.bind_pad(act, p),
                    _ => false,
                };
                if ok {
                    input::set_bindings(b);
                    self.sfx.play(crate::sfx::Id::Claw(1));
                    self.rebind = None;
                    return;
                }
            }
            self.rebind = if left <= 0.0 { None } else { Some(left) };
            return;
        }
        let n = CONTROL_BACK + 1;
        if a.up {
            self.controls_cursor = (self.controls_cursor + n - 1) % n;
        }
        if a.down {
            self.controls_cursor = (self.controls_cursor + 1) % n;
        }
        if a.left || a.right {
            self.controls_col = 1 - self.controls_col;
        }
        let mut chosen = a.confirm;
        if let Some(Hit::ControlCell(row, col)) = tap_hit {
            chosen = true;
            self.controls_cursor = row;
            self.controls_col = col;
        }
        if a.cancel || (chosen && self.controls_cursor == CONTROL_BACK) {
            self.save_prefs();
            self.scene = Scene::Settings { back };
            return;
        }
        if chosen && self.controls_cursor == CONTROL_RESET {
            input::set_bindings(input::Bindings::DEFAULT);
            self.sfx.play(crate::sfx::Id::Claw(0));
        } else if chosen {
            // 확인을 누른 그 프레임의 키(Enter·Space·J)가 바로 잡히지 않게 다음 프레임부터.
            self.rebind = Some(REBIND_TIMEOUT + dt);
            self.input.take();
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
                    Some(1) => {
                        // 다시 하기·타이틀로: 런이 끝난다. 들고 있던 반딧불은 마을로(0047).
                        self.bank_glowbugs();
                        self.start_game();
                    }
                    Some(2) => {
                        self.settings_cursor = 0;
                        self.scene = Scene::Settings { back: Back::Pause };
                    }
                    Some(3) => {
                        self.bank_glowbugs();
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
                            if WEAPONS[w].kind == self.world.character.weapon_kind() {
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
                let n = crate::journal::visible(&self.profile).len().max(1);
                if a.up {
                    self.journal_cursor = (self.journal_cursor + n - 1) % n;
                }
                if a.down {
                    self.journal_cursor = (self.journal_cursor + 1) % n;
                }
                let mut take = a.confirm;
                if let Some(Hit::JournalItem(i)) = tap_hit {
                    // 고른 칸을 한 번 더 누르면 받는다.
                    take = i == self.journal_cursor;
                    self.journal_cursor = i;
                }
                if take {
                    self.claim_journal();
                }
            }
            4 => {
                let n = crate::species::BESTIARY.len();
                if a.up {
                    self.bestiary_cursor = (self.bestiary_cursor + n - 1) % n;
                }
                if a.down {
                    self.bestiary_cursor = (self.bestiary_cursor + 1) % n;
                }
                if let Some(Hit::BestiaryItem(i)) = tap_hit {
                    self.bestiary_cursor = i;
                }
                // 확인(또는 설명을 누르기) = 설명 쪽 넘기기. 좌우는 일시정지에서 탭을 바꾼다.
                if a.confirm || tap_hit == Some(Hit::BestiaryPage) {
                    self.bestiary_page = (self.bestiary_page + 1) % 3;
                }
            }
            _ => {}
        }
    }

    // ---- 그리기 ---------------------------------------------------------------

    fn draw(&mut self) {
        let in_world = matches!(
            self.scene,
            Scene::Game
                | Scene::Pause
                | Scene::Cutscene
                | Scene::Route
                | Scene::Shop
                | Scene::KnotPick
                | Scene::Decor
                | Scene::Tree
                | Scene::DreamPick
                | Scene::CharPick
                | Scene::Settings { back: Back::Pause }
                | Scene::Controls { back: Back::Pause }
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
            for (i, actor) in c.actors.iter().enumerate().skip(1) {
                if !actor.visible {
                    continue;
                }
                let pos = self.renderer.to_view(actor.pos);
                match actor.sprite {
                    Some(cutscene::Sprite::Critter(row)) => self.renderer.draw_actor_sprite(
                        row,
                        pos,
                        actor.facing,
                        c.moving(i),
                        actor.color,
                    ),
                    Some(cutscene::Sprite::Bori) => {
                        self.renderer
                            .draw_bori_actor(pos, actor.facing, c.moving(i))
                    }
                    None => render::draw_actor(pos, actor.facing, actor.color),
                }
            }
        }
        let ui = self.renderer.present();
        self.hits.clear();
        self.last_ui = (ui.scale, ui.ox, ui.oy);

        match self.scene {
            Scene::Intro => self.draw_intro(&ui),
            Scene::Title => self.draw_title(&ui),
            Scene::Album => self.draw_album(&ui),
            Scene::KnotPick => {
                ui.dim();
                self.draw_knot_pick(&ui);
            }
            Scene::Decor => {
                ui.dim();
                self.draw_decor(&ui);
            }
            Scene::Tree => {
                ui.dim();
                self.draw_tree(&ui);
            }
            Scene::DreamPick => {
                ui.dim();
                self.draw_dream_pick(&ui);
            }
            Scene::CharPick => {
                ui.dim();
                self.draw_char_pick(&ui);
            }
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
            Scene::Controls { back } => {
                if back == Back::Pause {
                    ui.dim();
                }
                self.draw_controls(&ui);
            }
            Scene::Game => {
                self.draw_letterbox(&ui);
                self.renderer
                    .draw_hud(&ui, &self.world, self.input.gamepad_count);
                // 보스와 싸우는 동안은 힌트 대신 보스 막대(같은 자리).
                let boss = self.world.enemies.iter().any(|e| e.is_boss() && !e.asleep);
                if let Some(g) = &self.gift {
                    self.draw_gift(&ui, g);
                } else if let Some(h) = self
                    .tutorial
                    .as_ref()
                    .and_then(|t| t.hint())
                    .filter(|_| !boss)
                {
                    self.draw_hint(&ui, &h);
                }
                // 획득 창이 떠 있으면 말풍선은 숨긴다(창 위에 겹쳤다).
                self.draw_dream_hud(&ui);
                if self.gift.is_none() {
                    self.draw_interact_bubble(&ui);
                }
                // 마을: 마을 씨앗·맡긴 반딧불(오른쪽 위 막대 아래).
                if self.world.adv.place == crate::adventure::Place::Village {
                    let n = self.profile.count(crate::village::KEY_SEEDS);
                    let t = fill(i18n::s().village_seeds, &[&n]);
                    ui.text_right(&t, 316.0, 14.0, FONT_PX, ui::TEXT);
                    let l = self.profile.count(crate::talents::KEY_LIGHT);
                    let t = fill(i18n::s().tree_light, &[&l]);
                    ui.text_right(&t, 316.0, 27.0, FONT_PX, ui::TEXT);
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
            Scene::Route => {
                ui.dim();
                self.draw_route(&ui);
            }
            Scene::Shop => {
                self.renderer
                    .draw_hud(&ui, &self.world, self.input.gamepad_count);
                ui.dim();
                self.draw_shop(&ui);
            }
        }
        if let Some(mut o) = self.input.overlay() {
            let p = &self.world.player;
            let burrow = crate::tuning::BURROW_COOLDOWN * p.stats.burrow_cd;
            o.buttons[crate::input::TOUCH_DIG].ready = 1.0 - p.burrow_cd / burrow;
            o.buttons[crate::input::TOUCH_SP].ready = p.sp / crate::tuning::SP_MAX;
            o.buttons[crate::input::TOUCH_ITEM].ready =
                1.0 - p.item_cd / (crate::tuning::ITEM_COOLDOWN * p.eat_cd_mul);
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
        let hint = vfill(i18n::s().skip_hint, &[Verb::Confirm]);
        ui.text_center(&hint, 160.0, 160.0, FONT_PX, ui::TEXT_DIM);
    }

    fn title_items(&self) -> Vec<TitleItem> {
        let mut v = vec![TitleItem::Start];
        if self.profile.flag(crate::profile::key::TUTORIAL) {
            v.push(TitleItem::Daily);
        }
        v.push(TitleItem::Album);
        v.push(TitleItem::Settings);
        #[cfg(feature = "dev")]
        v.push(TitleItem::Dev);
        v
    }

    fn title_label(&self, it: TitleItem) -> String {
        let s = i18n::s();
        match it {
            TitleItem::Start => s.title_start.to_string(),
            TitleItem::Daily => {
                let day = crate::daily::today();
                if self.profile.count(&crate::daily::key_clear(day)) > 0 {
                    s.daily_clear.to_string()
                } else {
                    match self.profile.count(&crate::daily::key_best(day)) {
                        0 => s.title_daily.to_string(),
                        n => fill(s.daily_best, &[&n]),
                    }
                }
            }
            TitleItem::Album => s.title_album.to_string(),
            TitleItem::Settings => s.title_settings.to_string(),
            #[cfg(feature = "dev")]
            TitleItem::Dev => s.title_dev.to_string(),
        }
    }

    /// 뿌리 길에서 고른 캐릭터(0056). 모모는 돌아온 뒤에만.
    fn picked(&self) -> Character {
        use crate::profile::key;
        if self.profile.count(key::CHAR_PICK) == 1 && self.profile.flag(key::MOMO_BACK) {
            Character::Momo
        } else {
            Character::Mole
        }
    }

    /// 실뜨기에서 고를 수 있는 가장 높은 단계(고른 캐릭터의 것, 0040).
    fn knots_open(&self) -> u8 {
        let k = crate::profile::key::knots_open(self.picked());
        (self.profile.count(&k) as u8).min(crate::knots::MAX_LEVEL)
    }

    /// 타이틀 "시작": 튜토리얼을 마쳤으면 마을로(내려가는 건 마을의 뿌리 길, 0045), 아니면 오프닝부터.
    fn begin_run(&mut self) {
        let last = self.profile.count(crate::profile::key::KNOT_PICK) as u8;
        self.knot_level = last.min(self.knots_open());
        if self.profile.flag(crate::profile::key::TUTORIAL) {
            self.start(Scenario::Village);
        } else {
            self.start(Scenario::Normal);
        }
    }

    /// 기록 → 마을 모습.
    fn village_state(&self) -> crate::village::State {
        use crate::profile::key;
        let mut decor = [false; crate::village::DECOR_ALL];
        for (i, d) in decor.iter_mut().enumerate() {
            *d = self.profile.flag(&crate::village::key_decor(i));
        }
        crate::village::State {
            bori: self.profile.flag(key::BORI_BACK),
            halmae: self.profile.flag(key::HALF_ENDING) || self.profile.flag(key::TRUE_ENDING),
            momo: self.profile.flag(key::MOMO_BACK),
            doto: self.profile.flag(key::DOTO_MEMORY),
            kongi: self.profile.flag(key::KONGI_BACK),
            decor,
            growth: crate::village::growth(
                self.profile.count(&key::clears(Character::Mole))
                    + self.profile.count(&key::clears(Character::Momo)),
            ),
        }
    }

    fn update_knot_pick(&mut self, a: &Actions, tap_hit: Option<Hit>) {
        let open = self.knots_open();
        if a.left || tap_hit == Some(Hit::KnotLeft) {
            self.knot_level = self.knot_level.saturating_sub(1);
        }
        if a.right || tap_hit == Some(Hit::KnotRight) {
            self.knot_level = (self.knot_level + 1).min(open);
        }
        if a.confirm || tap_hit == Some(Hit::KnotGo) {
            use crate::profile::key;
            self.profile
                .set_count(key::KNOT_PICK, self.knot_level as u32);
            self.profile.unset(key::KNOT_NEW);
            crate::profile::save(&self.profile);
            // 마을로 돌아간다. 고른 단계는 뿌리 길로 내려갈 때 걸린다.
            self.scene = Scene::Game;
            return;
        }
        if a.cancel || tap_hit == Some(Hit::Close) {
            self.scene = Scene::Game;
        }
    }

    fn draw_knot_pick(&mut self, ui: &Ui) {
        use crate::knots::{MAX_LEVEL, NAMES};
        let s = i18n::s();
        let open = self.knots_open();
        // 안내 두 줄이 창 밖으로 넘쳐 조작 안내와 겹쳤다: 창을 조금 키우고 매듭 줄 간격을 12로(0053).
        let panel = Rect::new(36.0, 6.0, 248.0, 160.0);
        ui.panel(panel, Some(s.knot_title));
        let mut y = panel.y + 20.0;
        for l in ui::wrap(ui, s.knot_intro, panel.w - 16.0, FONT_PX)
            .iter()
            .take(2)
        {
            ui.text(l, panel.x + 8.0, y, FONT_PX, ui::TEXT_DIM);
            y += 13.0;
        }
        // ◀ 단계 ▶
        y += 4.0;
        let label = if self.knot_level == 0 {
            s.knot_none.to_string()
        } else {
            fill(s.knot_level, &[&self.knot_level])
        };
        let (row, l, r) =
            ui.setting_row(s.knot_title, &label, panel.x + 8.0, y, panel.w - 16.0, true);
        self.hits.push((l, Hit::KnotLeft));
        self.hits.push((r, Hit::KnotRight));
        self.hits.push((row, Hit::KnotGo));
        y += FONT_PX + 8.0;
        // 매듭 목록: 걸림 ● / 열렸지만 안 걸림 ○ / 아직 안 열림 ?
        // 열 줄은 창에 안 들어가 다섯 줄씩 두 쪽: 고른 단계가 6 이상이면 6~10(1~5는 다 걸려 있다, 0053).
        let page: u8 = if self.knot_level > 5 { 5 } else { 0 };
        for i in page..(page + 5).min(MAX_LEVEL) {
            let n = i + 1;
            let (mark, text, c) = if n > open {
                ("?", "…".to_string(), ui::TEXT_DIM)
            } else if n <= self.knot_level {
                ("●", tr(NAMES[i as usize]).to_string(), ui::TEXT)
            } else {
                ("○", tr(NAMES[i as usize]).to_string(), ui::TEXT_DIM)
            };
            ui.text(mark, panel.x + 10.0, y, FONT_PX, c);
            let t = ui.fit(&text, panel.w - 34.0, FONT_PX);
            ui.text(&t, panel.x + 22.0, y, FONT_PX, c);
            y += 12.0;
        }
        let note = if self.profile.flag(crate::profile::key::KNOT_NEW) {
            s.toast_knot_open
        } else {
            s.knot_locked
        };
        for (i, l) in ui::wrap(ui, note, panel.w - 16.0, FONT_PX)
            .iter()
            .take(2)
            .enumerate()
        {
            ui.text(
                l,
                panel.x + 8.0,
                y + 2.0 + i as f32 * 12.0,
                FONT_PX,
                ui::ACCENT,
            );
        }
        let hint = vfill(s.knot_hint, &[Verb::Confirm, Verb::Change, Verb::Back]);
        let hint = ui.fit(&hint, 300.0, FONT_PX);
        ui.text_center(&hint, 160.0, panel.bottom() + 4.0, FONT_PX, ui::TEXT_DIM);
        self.close_button(ui, panel);
    }

    fn album_seen(&self, il: crate::illust::Illust) -> bool {
        self.profile
            .flag(&crate::profile::key::illust(il.spec().file))
    }

    /// 앨범: 5×2 칸에서 고르고, 본 그림은 크게(패닝·입자 그대로). 확인·뒤로·탭으로 닫는다.
    fn update_album(&mut self, a: &Actions, tap_hit: Option<Hit>, dt: f32) {
        use crate::illust::Illust;
        if let Some(t) = self.album_view.as_mut() {
            *t += dt;
            if a.confirm || a.cancel || a.attack || a.tap.is_some() {
                self.album_view = None;
            }
            return;
        }
        let n = Illust::ALL.len();
        let c = self.album_cursor;
        if a.left {
            self.album_cursor = (c + n - 1) % n;
        }
        if a.right {
            self.album_cursor = (c + 1) % n;
        }
        if a.up || a.down {
            self.album_cursor = (c + ALBUM_COLS) % n;
        }
        let chosen = if a.confirm {
            Some(self.album_cursor)
        } else if let Some(Hit::AlbumItem(i)) = tap_hit {
            self.album_cursor = i;
            Some(i)
        } else {
            None
        };
        if let Some(i) = chosen {
            if self.album_seen(Illust::ALL[i]) {
                self.album_view = Some(0.0);
            }
        }
        if a.cancel || matches!(tap_hit, Some(Hit::Close)) {
            self.scene = Scene::Title;
        }
    }

    fn draw_album(&mut self, ui: &Ui) {
        use crate::illust::Illust;
        let s = i18n::s();
        if let Some(t) = self.album_view {
            let il = Illust::ALL[self.album_cursor];
            self.renderer.draw_illust(ui, il, t);
            let hint = vfill(s.album_close, &[Verb::Back]);
            ui.text_center(&hint, 160.0, 166.0, FONT_PX, ui::TEXT_DIM);
            return;
        }
        let panel = Rect::new(16.0, 8.0, 288.0, 150.0);
        ui.panel(panel, Some(s.title_album));
        // 5×2칸(그림 9장, 0053). 칸은 16:9.
        let (cw, ch, gap) = (52.0, 30.0, 5.0);
        let x0 = panel.x + (panel.w - ALBUM_COLS as f32 * cw - (ALBUM_COLS - 1) as f32 * gap) / 2.0;
        let y0 = panel.y + 38.0;
        let mut seen = 0;
        for (i, il) in Illust::ALL.iter().enumerate() {
            let r = Rect::new(
                x0 + (i % ALBUM_COLS) as f32 * (cw + gap),
                y0 + (i / ALBUM_COLS) as f32 * (ch + gap + 10.0),
                cw,
                ch,
            );
            if self.album_seen(*il) {
                seen += 1;
                self.renderer.draw_illust_thumb(ui, *il, r);
            } else {
                ui.rect(r, ui::PANEL_LIGHT);
                ui.text_center("?", r.center().x, r.y + 12.0, FONT_PX, ui::TEXT_DIM);
            }
            let selected = i == self.album_cursor;
            ui.rect_lines(r, if selected { ui::ACCENT } else { ui::BORDER });
            if selected {
                ui.rect_lines(
                    Rect::new(r.x - 1.0, r.y - 1.0, r.w + 2.0, r.h + 2.0),
                    ui::ACCENT,
                );
            }
            self.hits.push((r, Hit::AlbumItem(i)));
        }
        let count = fill(s.album_count, &[&seen, &Illust::ALL.len()]);
        ui.text(&count, panel.x + 8.0, panel.y + 21.0, FONT_PX, ui::TEXT_DIM);
        let hint = device_hint(s.album_hint, s.album_hint, &[Verb::Confirm, Verb::Pick]);
        let hint = ui.fit(&hint, panel.w - 8.0, FONT_PX);
        ui.text_center(&hint, 160.0, panel.bottom() + 4.0, FONT_PX, ui::TEXT_DIM);
        self.close_button(ui, panel);
    }

    fn draw_title(&mut self, ui: &Ui) {
        // 뒤의 뿌리 선이 가는 글자 획과 겹치면 끊겨(깨져) 보인다(사람 피드백). 타이틀 글에는 어두운 테두리.
        let ui = &ui.with_outline();
        ui.text_center("Naïl", 160.0, 26.0, FONT_PX * 2.0, ui::ACCENT);
        ui.text_center("Under the Roots", 160.0, 50.0, FONT_PX, ui::TEXT);
        let s = i18n::s();
        let labels: Vec<String> = self
            .title_items()
            .into_iter()
            .map(|it| self.title_label(it))
            .collect();
        let labels: Vec<&str> = labels.iter().map(|l| l.as_str()).collect();
        let rects = ui.menu_list(&labels, self.title_cursor, 96.0, 68.0, 128.0);
        // 오늘의 굴 줄 오른쪽에 오늘 날짜(UTC): 날마다 다른 굴이라는 표시.
        if let Some(i) = self
            .title_items()
            .iter()
            .position(|t| *t == TitleItem::Daily)
        {
            let (_, m, d) = crate::daily::civil(crate::daily::today());
            let r = rects[i];
            ui.text(
                &format!("{m}/{d}"),
                r.right() + 4.0,
                r.y + 2.0,
                FONT_PX,
                ui::TEXT_DIM,
            );
        }
        if let Some((note, _)) = self.title_note {
            ui.text_center(tr(note), 160.0, 138.0, FONT_PX, ui::ACCENT);
        }
        for (i, r) in rects.into_iter().enumerate() {
            self.hits.push((r, Hit::TitleItem(i)));
        }
        let hint = device_hint(
            s.title_controls,
            s.title_controls_touch,
            &[Verb::Pick, Verb::Confirm, Verb::Back],
        );
        ui.text_center(&hint, 160.0, 150.0, FONT_PX, ui::TEXT_DIM);
        ui.text_right(s.version, 316.0, 166.0, FONT_PX, ui::TEXT_DIM);
        ui.text(s.copyright, 4.0, 166.0, FONT_PX, ui::TEXT_DIM);
    }

    fn draw_dev(&mut self, ui: &Ui) {
        // 개발 화면은 번역하지 않는다(한국어). 언어와 상관없이 한글 폰트로.
        let base = ui.with_base_font();
        let ui = &base;
        let panel = Rect::new(24.0, 16.0, 272.0, 148.0);
        ui.panel(panel, Some("개발 시나리오"));
        // 목록이 패널보다 길다: 고른 줄이 보이게 VISIBLE줄씩 굴린다.
        const VISIBLE: usize = 7;
        let n = scenario::LIST.len();
        let first = self
            .dev_cursor
            .saturating_sub(VISIBLE - 1)
            .min(n.saturating_sub(VISIBLE));
        let names: Vec<&str> = scenario::LIST
            .iter()
            .skip(first)
            .take(VISIBLE)
            .map(|e| e.name)
            .collect();
        let rects = ui.menu_list(
            &names,
            self.dev_cursor - first,
            panel.x + 6.0,
            panel.y + 24.0,
            110.0,
        );
        for (i, r) in rects.into_iter().enumerate() {
            self.hits.push((r, Hit::DevItem(first + i)));
        }
        if first > 0 {
            ui.text("▲", panel.x + 112.0, panel.y + 24.0, FONT_PX, ui::TEXT_DIM);
        }
        if first + VISIBLE < n {
            ui.text(
                "▼",
                panel.x + 112.0,
                panel.y + 24.0 + 6.0 * (FONT_PX + 4.0),
                FONT_PX,
                ui::TEXT_DIM,
            );
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
    fn draw_cutscene(&mut self, ui: &Ui) {
        let Some(c) = &self.cutscene else { return };
        // 일러스트는 월드·레터박스 위, 대화창 아래(docs/decisions/0039).
        if let Some(il) = c.illust {
            self.renderer.draw_illust(ui, il, c.illust_t);
        }
        if let Some(d) = c.dialogue().filter(|d| d.narration) {
            // 내레이션: 이름·초상화 없이 아래쪽 어두운 띠에 가운데 맞춤.
            let b = Rect::new(8.0, 132.0, 304.0, 42.0);
            ui.rect(b, Color::new(0.04, 0.03, 0.06, 0.72));
            // 줄은 다 찍힌 글로 나누고(찍히는 동안 줄이 바뀌지 않게) 앞에서부터 찍힌 만큼만 보인다.
            let full = ui::wrap(ui, &d.text, b.w - 24.0, FONT_PX);
            let top = b.y + (b.h - full.len().min(3) as f32 * 13.0) / 2.0;
            let mut left = d.shown_chars;
            for (i, l) in full.iter().take(3).enumerate() {
                let n = l.chars().count().min(left);
                left -= n;
                let part: String = l.chars().take(n).collect();
                ui.text(
                    &part,
                    160.0 - ui.text_w(l, FONT_PX) / 2.0,
                    top + i as f32 * 13.0,
                    FONT_PX,
                    Color::new(0.94, 0.90, 0.82, 1.0),
                );
            }
            if d.complete && (get_time() * 2.0) as i64 % 2 == 0 {
                ui.text(
                    "▼",
                    b.right() - 14.0,
                    b.bottom() - 13.0,
                    FONT_PX,
                    ui::ACCENT,
                );
            }
        } else if let Some(d) = c.dialogue() {
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
        // 건너뛰기 안내는 누를 수도 있다(터치에는 뒤로 버튼이 없다).
        let skip = vfill(i18n::s().cut_skip, &[Verb::Skip]);
        ui.text_center(&skip, 160.0, 1.0, FONT_PX, ui::TEXT_DIM);
        let sw = ui.text_w(&skip, FONT_PX) + 16.0;
        self.hits
            .push((Rect::new(160.0 - sw / 2.0, 0.0, sw, 18.0), Hit::Skip));
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
        let panel = Rect::new(56.0, 6.0, 208.0, 156.0);
        let s = i18n::s();
        ui.panel(panel, Some(s.settings));
        let assist = self.assist_resist();
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
            // 포포의 응원: 켜져 있으면 지금 줄여 주는 비율.
            if assist > 0.0 {
                format!("{} {}%", s.on, (assist * 100.0).round())
            } else {
                s.off.to_string()
            },
            String::new(),
            String::new(),
            String::new(),
        ];
        let row_h = FONT_PX + 4.0;
        for (i, label) in trs(&SETTING_ROWS).into_iter().enumerate() {
            let y = panel.y + 22.0 + i as f32 * row_h;
            let selected = self.settings_cursor == i;
            if setting_is_button(i) {
                let r = Rect::new(panel.x + 6.0, y, panel.w - 12.0, FONT_PX + 4.0);
                if selected {
                    ui.rect(r, ui::PANEL_LIGHT);
                    ui.text("▶", r.x + 2.0, r.y + 2.0, FONT_PX, ui::ACCENT);
                }
                let label = if i == SETTING_CONTROLS {
                    format!("{label}  →")
                } else if i == SETTING_ERASE && self.erase_armed {
                    s.set_erase_confirm.to_string()
                } else {
                    label.to_string()
                };
                ui.text(
                    &label,
                    r.x + 14.0,
                    r.y + 2.0,
                    FONT_PX,
                    if selected { ui::TEXT } else { ui::TEXT_DIM },
                );
                self.hits.push((r, Hit::SettingRow(i)));
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
        let settings_hint = vfill(s.settings_hint, &[Verb::Change]);
        for (i, l) in ui::wrap(ui, &settings_hint, 300.0, FONT_PX)
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

    /// 조작 설정 화면: 동작 | 키보드 | 패드 표. 고른 칸 강조, 기다리는 중이면 "누르세요… 5".
    fn draw_controls(&mut self, ui: &Ui) {
        use crate::input::{self, ACTS};
        let s = i18n::s();
        let panel = Rect::new(28.0, 2.0, 264.0, 156.0);
        ui.panel(panel, Some(s.set_controls));
        let b = input::bindings();
        let row_h = FONT_PX + 0.5;
        let name_w = 110.0;
        let col_w = (panel.w - 12.0 - name_w) / 2.0;
        let x0 = panel.x + 6.0;
        let y0 = panel.y + 19.0;
        ui.text(
            &ui.fit(s.ctl_key, col_w - 6.0, FONT_PX),
            x0 + name_w + 4.0,
            y0,
            FONT_PX,
            ui::TEXT_DIM,
        );
        ui.text(
            &ui.fit(s.ctl_pad, col_w - 6.0, FONT_PX),
            x0 + name_w + col_w + 4.0,
            y0,
            FONT_PX,
            ui::TEXT_DIM,
        );
        for row in 0..=CONTROL_BACK {
            let y = y0 + (row as f32 + 1.0) * row_h;
            let on = row == self.controls_cursor;
            if row < CONTROL_RESET {
                let act = ACTS[row];
                ui.text(
                    &ui.fit(tr(CONTROL_NAMES[row]), name_w - 4.0, FONT_PX),
                    x0 + 2.0,
                    y,
                    FONT_PX,
                    if on { ui::TEXT } else { ui::TEXT_DIM },
                );
                for col in 0..2 {
                    let r = Rect::new(
                        x0 + name_w + col as f32 * col_w,
                        y - 1.0,
                        col_w - 4.0,
                        row_h,
                    );
                    let sel = on && col == self.controls_col;
                    let waiting = sel && self.rebind.is_some();
                    if sel {
                        ui.rect(r, ui::PANEL_LIGHT);
                        ui.rect_lines(r, ui::ACCENT);
                    }
                    let label = if waiting {
                        fill(s.ctl_wait, &[&(self.rebind.unwrap_or(0.0).ceil() as i32)])
                    } else if col == 0 {
                        b.key_label(act).to_string()
                    } else {
                        b.pad(act).label().to_string()
                    };
                    ui.text(
                        &ui.fit(&label, r.w - 6.0, FONT_PX),
                        r.x + 4.0,
                        y,
                        FONT_PX,
                        if waiting { ui::ACCENT } else { ui::TEXT },
                    );
                    self.hits.push((r, Hit::ControlCell(row, col)));
                }
            } else {
                let label = if row == CONTROL_RESET {
                    s.ctl_reset
                } else {
                    s.set_back
                };
                let r = Rect::new(x0, y - 1.0, panel.w - 12.0, row_h);
                if on {
                    ui.rect(r, ui::PANEL_LIGHT);
                    ui.text("▶", r.x + 2.0, y, FONT_PX, ui::ACCENT);
                }
                ui.text(
                    label,
                    r.x + 14.0,
                    y,
                    FONT_PX,
                    if on { ui::TEXT } else { ui::TEXT_DIM },
                );
                self.hits.push((r, Hit::ControlCell(row, 0)));
            }
        }
        // 안내: 바꾸는 법은 패널 안 아래, 고정된 키는 패널 밖(두 줄까지).
        let hint = ui.fit(s.ctl_hint, panel.w - 12.0, FONT_PX);
        ui.text_center(&hint, 160.0, panel.bottom() - 12.0, FONT_PX, ui::TEXT_DIM);
        let mut y = panel.bottom() + 1.0;
        for l in ui::wrap(ui, s.ctl_fixed, 312.0, FONT_PX).iter().take(2) {
            ui.text_center(l, 160.0, y, FONT_PX, ui::TEXT_DIM);
            y += 10.5;
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
        let item = self.gift_queue.remove(0);
        match item {
            crate::world::Gift::Weapon(w) => {
                self.inventory.add(w);
                if self.inventory.equipped.is_none() {
                    self.inventory.equipped = Some(w);
                    self.apply_equipped();
                }
            }
            crate::world::Gift::Charm(c) => self.inventory.add_charm(c),
        }
        self.world.freeze();
        self.gift = Some(Gift { item, t: 0.0 });
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
        let s = i18n::s();
        let (title, name, desc) = match g.item {
            crate::world::Gift::Weapon(w) => (s.gift_title, WEAPONS[w].name, WEAPONS[w].desc),
            crate::world::Gift::Charm(c) => (s.gift_charm, CHARMS[c].name, CHARMS[c].desc),
        };
        let k = (g.t / 0.2).min(1.0);
        let r = Rect::new(56.0, 42.0 + (1.0 - k) * 8.0, 208.0, 86.0);
        ui.panel(r, None);
        ui.text_center(title, 160.0, r.y + 5.0, FONT_PX, ui::ACCENT);
        let icon = Rect::new(r.x + 10.0, r.y + 22.0, 22.0, 22.0);
        ui.rect(icon, ui::PANEL_LIGHT);
        ui.rect_lines(icon, ui::BORDER);
        match g.item {
            crate::world::Gift::Weapon(w) => draw_claw_icon(ui, icon, WEAPONS[w].color),
            crate::world::Gift::Charm(c) => draw_charm_icon(ui, icon, CHARMS[c].color),
        }
        ui.text(tr(name), icon.right() + 8.0, r.y + 22.0, FONT_PX, ui::TEXT);
        let lines = ui::wrap(ui, tr(desc), r.right() - icon.right() - 16.0, FONT_PX);
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
            let hint = vfill(s.dead_hint, &[Verb::Restart, Verb::Menu]);
            ui.text_center(&hint, 160.0, 96.0, FONT_PX, ui::TEXT);
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
        let s = i18n::s();
        ui.text_right(
            &device_hint(s.tab_hint, s.tab_hint_touch, &[Verb::Tabs]),
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
            4 => self.draw_bestiary(ui, content),
            _ => self.draw_map(ui, content),
        }
    }

    /// 상태 탭: 캐릭터 능력치는 숫자로(두 칸 표), 그 아래 필살기 설명과 이번 판 기록.
    fn draw_status(&self, ui: &Ui, c: Rect) {
        let w = &self.world;
        let p = &w.player;
        let s = i18n::s();
        let st = p.stats;
        let gauge = w.character.gauge();
        let base_dmg = p.moveset.hits.first().map(|h| h.damage).unwrap_or(1);
        let weapon = match self.inventory.equipped.filter(|_| p.armed) {
            Some(i) => tr(WEAPONS[i].name).to_string(),
            None => s.bare_hands.to_string(),
        };
        let charms = if w.charms.is_empty() {
            s.charms_none.to_string()
        } else {
            w.charms
                .iter()
                .map(|c| tr(CHARMS[*c].name))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let pct = |x: f32| fill(s.fmt_pct, &[&((x * 100.0).round() as i32)]);
        let left: [(&str, String); 5] = [
            (s.hp, fill(s.fmt_hp, &[&p.hp, &st.max_hp])),
            (s.stat_power, (base_dmg + st.damage).to_string()),
            (s.stat_aspd, pct(st.attack_speed)),
            (
                if w.character == Character::Momo {
                    s.stat_hop_cd
                } else {
                    s.stat_dig_cd
                },
                fill(s.fmt_secs, &[&format!("{:.1}", p.burrow_cd_full())]),
            ),
            (&fill(s.stat_gauge, &[&tr(gauge.name())]), pct(st.sp_gain)),
        ];
        let right: [(&str, String); 4] = [
            (s.stat_weapon, weapon),
            (s.charms_label, charms),
            (s.glowbugs, w.glowbugs.to_string()),
            (s.seeds, w.seeds.to_string()),
        ];
        let row_h = FONT_PX + 1.0;
        let col_w = c.w / 2.0 - 4.0;
        for (col, rows) in [(0usize, &left[..]), (1, &right[..])] {
            let x = c.x + col as f32 * (col_w + 8.0);
            let key_w = rows
                .iter()
                .map(|(k, _)| ui.text_w(k, FONT_PX))
                .fold(0.0, f32::max)
                .min(col_w * 0.55)
                + 6.0;
            for (i, (k, v)) in rows.iter().enumerate() {
                let y = c.y + i as f32 * row_h;
                ui.text(&ui.fit(k, key_w - 6.0, FONT_PX), x, y, FONT_PX, ui::ACCENT);
                ui.text(
                    &ui.fit(v, col_w - key_w, FONT_PX),
                    x + key_w,
                    y,
                    FONT_PX,
                    ui::TEXT,
                );
            }
        }
        // 체력 하트(숫자 옆에 그림으로도). 숫자 바로 뒤에 둔다: 고정 자리면 일본어처럼 이름 칸이 넓은 언어에서 숫자를 덮었다.
        let hy = c.y + 2.0;
        let key_w = left
            .iter()
            .map(|(k, _)| ui.text_w(k, FONT_PX))
            .fold(0.0, f32::max)
            .min(col_w * 0.55)
            + 6.0;
        let hx0 = c.x + key_w + ui.text_w(&left[0].1, FONT_PX) + 5.0;
        for i in 0..st.max_hp.min(10) {
            let col = if i < p.hp {
                Color::new(0.93, 0.33, 0.40, 1.0)
            } else {
                ui::TEXT_DIM
            };
            ui.rect(Rect::new(hx0 + i as f32 * 5.0, hy + 2.0, 4.0, 3.0), col);
            ui.rect(
                Rect::new(hx0 + i as f32 * 5.0 + 1.0, hy + 5.0, 2.0, 1.0),
                col,
            );
        }
        // 필살기
        let y0 = c.y + 5.0 * row_h + 4.0;
        ui.line(vec2(c.x, y0 - 2.0), vec2(c.right(), y0 - 2.0), ui::BORDER);
        ui.text(s.special_name, c.x, y0, FONT_PX, gauge.color());
        let desc = fill(
            s.special_desc,
            &[
                &tr(gauge.name()),
                &crate::input::prompt(crate::input::Act::Special),
            ],
        );
        let dx = c.x + ui.text_w(s.special_name, FONT_PX) + 6.0;
        let lines = ui::wrap(ui, &desc, c.right() - dx, FONT_PX);
        let n = lines.len().min(3);
        for (j, l) in lines.iter().take(n).enumerate() {
            ui.text(l, dx, y0 + j as f32 * 12.0, FONT_PX, ui::TEXT);
        }
        // 낀 재주(0047): 자리 순서로. 없으면 어디서 깨우는지.
        let mut y = y0 + n as f32 * 12.0 + 2.0;
        let picked: Vec<&str> = crate::talents::TABLE
            .iter()
            .zip(p.talents.pick)
            .filter(|(_, side)| *side > 0)
            .map(|((_, sides), side)| tr(sides[side as usize - 1].name))
            .collect();
        let talents = if picked.is_empty() {
            s.tal_none.to_string()
        } else {
            picked.join(" · ")
        };
        for l in ui::wrap(ui, &fill(s.stat_talents, &[&talents]), c.w, FONT_PX)
            .iter()
            .take(2)
        {
            ui.text(l, c.x, y, FONT_PX, ui::ACCENT);
            y += 12.0;
        }
        // 화폐 설명(칸이 남으면).
        y += 2.0;
        for l in ui::wrap(ui, s.wallet_note, c.w, FONT_PX) {
            if y + FONT_PX > c.bottom() + 4.0 {
                break;
            }
            ui.text(&l, c.x, y, FONT_PX, ui::TEXT_DIM);
            y += 12.0;
        }
    }

    /// 도감 탭: 왼쪽 목록(만난 이웃만 이름, 아니면 ???), 오른쪽 그림 + 말로 된 능력 + 이야기·공격·요령.
    fn draw_bestiary(&mut self, ui: &Ui, c: Rect) {
        use crate::species::{self, BESTIARY};
        let s = i18n::s();
        let seen = &self.world.seen;
        let list_w = 74.0;
        // 목록이 아래 "만난 이웃 n/m" 줄에 닿지 않게. 줄 간격은 13 아래로 줄이지 않는다(9종에서 일본어가 겹쳤다, 0054).
        // 다 안 들어가면 고른 줄을 따라 넘긴다(▲▼).
        let avail = c.h - FONT_PX - 6.0;
        let row_h = (avail / BESTIARY.len() as f32)
            .floor()
            .clamp(13.0, FONT_PX + 4.0);
        let fits = ((avail / row_h).floor() as usize).max(1);
        let first = self
            .bestiary_cursor
            .saturating_sub(fits - 1)
            .min(BESTIARY.len().saturating_sub(fits));
        if first > 0 {
            ui.text("▲", c.x + list_w - 8.0, c.y - 9.0, FONT_PX, ui::TEXT_DIM);
        }
        if first + fits < BESTIARY.len() {
            ui.text(
                "▼",
                c.x + list_w - 8.0,
                c.y + fits as f32 * row_h - 3.0,
                FONT_PX,
                ui::TEXT_DIM,
            );
        }
        for (i, k) in BESTIARY.iter().enumerate().skip(first).take(fits) {
            let r = Rect::new(c.x, c.y + (i - first) as f32 * row_h, list_w, row_h - 1.0);
            let known = seen.contains(k);
            let name = if known {
                tr(k.species().name)
            } else {
                s.best_unknown
            };
            let on = i == self.bestiary_cursor;
            if on {
                ui.rect(r, Color::new(0.25, 0.18, 0.22, 0.9));
                ui.rect_lines(r, ui::ACCENT);
            }
            ui.text(
                &ui.fit(name, list_w - 6.0, FONT_PX),
                r.x + 3.0,
                r.y + 1.0,
                FONT_PX,
                if on {
                    ui::ACCENT
                } else if known {
                    ui::TEXT
                } else {
                    ui::TEXT_DIM
                },
            );
            self.hits.push((r, Hit::BestiaryItem(i)));
        }
        let n_seen = BESTIARY.iter().filter(|k| seen.contains(k)).count();
        ui.text(
            &fill(s.best_count, &[&n_seen, &BESTIARY.len()]),
            c.x,
            c.bottom() - FONT_PX,
            FONT_PX,
            ui::TEXT_DIM,
        );
        let k = BESTIARY[self.bestiary_cursor.min(BESTIARY.len() - 1)];
        let x = c.x + list_w + 8.0;
        let w = c.right() - x;
        if !seen.contains(&k) {
            ui.text(s.best_unseen, x, c.y + 20.0, FONT_PX, ui::TEXT_DIM);
            return;
        }
        let sp = k.species();
        // 그림: 2배 정수 배율
        let pic = Rect::new(x, c.y, 34.0, 34.0);
        ui.rect(pic, Color::new(0.10, 0.08, 0.12, 0.9));
        ui.rect_lines(pic, ui::BORDER);
        let inner = Rect::new(pic.x + 1.0, pic.y + 1.0, 32.0, 32.0);
        let frame = ((get_time() * 2.0) as u32) % 2;
        if k == crate::enemy::Kind::Chief {
            self.renderer.draw_chief_icon(ui, inner, frame);
        } else {
            self.renderer.draw_critter(ui, inner, sp.sheet_row, frame);
        }
        let tx = pic.right() + 6.0;
        ui.text(tr(sp.name), tx, c.y, FONT_PX, ui::ACCENT);
        let (hp, pow, spd) = species::words(sp);
        let words = [(s.best_hp, hp), (s.best_pow, pow), (s.best_spd, spd)];
        for (i, (label, v)) in words.iter().enumerate() {
            let y = c.y + 13.0 + i as f32 * 12.0;
            let lw = words
                .iter()
                .map(|(l, _)| ui.text_w(l, FONT_PX))
                .fold(0.0, f32::max)
                + 6.0;
            ui.text(
                &ui.fit(label, lw - 2.0, FONT_PX),
                tx,
                y,
                FONT_PX,
                ui::TEXT_DIM,
            );
            ui.text(
                &ui.fit(tr(*v), c.right() - tx - lw, FONT_PX),
                tx + lw,
                y,
                FONT_PX,
                ui::TEXT,
            );
        }
        // 설명은 쪽 하나에 한 갈래(이야기·공격·요령)를 칸 끝까지. 그래도 넘치면 마지막 줄에 "…".
        let y0 = c.y + 49.0;
        let page = self.bestiary_page.min(2);
        let (label, text) = [
            (s.best_lore, sp.lore),
            (s.best_atk, sp.attack),
            (s.best_tip, sp.tip),
        ][page];
        ui.text(label, x, y0, FONT_PX, ui::ACCENT);
        ui.text_right(
            &format!("{}/3 · {}", page + 1, crate::input::verb(Verb::Confirm)),
            c.right(),
            y0,
            FONT_PX,
            ui::TEXT_DIM,
        );
        let top = y0 + 13.0;
        let rows = (((c.bottom() + 2.0 - top) / 12.0).floor() as usize).max(1);
        let lines = ui::wrap(ui, tr(text), w, FONT_PX);
        for (i, l) in lines.iter().take(rows).enumerate() {
            let l = if i + 1 == rows && lines.len() > rows {
                ui.fit(&format!("{l}…"), w, FONT_PX)
            } else {
                l.clone()
            };
            ui.text(&l, x, top + i as f32 * 12.0, FONT_PX, ui::TEXT);
        }
        self.hits
            .push((Rect::new(x, y0, w, c.bottom() - y0), Hit::BestiaryPage));
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
                    s.equipped.to_string()
                } else {
                    vfill(s.equip_hint, &[Verb::Confirm])
                };
                ui.text(&hint, desc_x, y + 18.0, FONT_PX, ui::ACCENT);
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
                let hint = vfill(hint, &[Verb::Confirm]);
                ui.text(&hint, desc_x, y + 18.0, FONT_PX, ui::ACCENT);
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

    /// 저널 탭(0048): 왼쪽 목록(◆ 받을 수 있음, ■ 받았음, □ 하는 중), 오른쪽 갈래·글·보상·상태.
    fn draw_journal(&mut self, ui: &Ui, c: Rect) {
        use crate::journal::{visible, Reward};
        let s = i18n::s();
        let goals = visible(&self.profile);
        if goals.is_empty() {
            return;
        }
        self.journal_cursor = self.journal_cursor.min(goals.len() - 1);
        let row_h = FONT_PX + 4.0;
        let rows = ((c.h / row_h).floor() as usize).max(1);
        if self.journal_cursor < self.journal_scroll {
            self.journal_scroll = self.journal_cursor;
        }
        if self.journal_cursor >= self.journal_scroll + rows {
            self.journal_scroll = self.journal_cursor + 1 - rows;
        }
        let list_w = 150.0;
        for (k, g) in goals
            .iter()
            .enumerate()
            .skip(self.journal_scroll)
            .take(rows)
        {
            let r = Rect::new(
                c.x,
                c.y + (k - self.journal_scroll) as f32 * row_h,
                list_w,
                row_h,
            );
            let selected = k == self.journal_cursor;
            if selected {
                ui.rect(r, ui::PANEL_LIGHT);
            }
            let (mark, mc) = if g.claimed(&self.profile) {
                ("■", ui::TEXT_DIM)
            } else if g.ready(&self.profile) {
                ("◆", ui::ACCENT)
            } else {
                ("□", ui::TEXT_DIM)
            };
            let col = if g.claimed(&self.profile) {
                ui::TEXT_DIM
            } else if selected || g.ready(&self.profile) {
                ui::TEXT
            } else {
                Color::new(0.78, 0.72, 0.64, 1.0)
            };
            ui.text(mark, r.x + 2.0, r.y + 2.0, FONT_PX, mc);
            let prog = format!("{}/{}", g.progress(&self.profile), g.goal);
            let pw = ui.text_w(&prog, FONT_PX);
            let title = ui.fit(tr(g.title), r.w - 22.0 - pw, FONT_PX);
            ui.text(&title, r.x + 14.0, r.y + 2.0, FONT_PX, col);
            ui.text_right(&prog, r.right() - 3.0, r.y + 2.0, FONT_PX, col);
            self.hits.push((r, Hit::JournalItem(k)));
        }
        // 목록이 더 있으면 ▲▼.
        if self.journal_scroll > 0 {
            ui.text_right("▲", c.x + list_w, c.y - 10.0, FONT_PX, ui::TEXT_DIM);
        }
        if self.journal_scroll + rows < goals.len() {
            ui.text_right("▼", c.x + list_w, c.bottom() - 2.0, FONT_PX, ui::TEXT_DIM);
        }
        let g = goals[self.journal_cursor];
        let dx = c.x + list_w + 8.0;
        let dw = c.right() - dx;
        ui.text(tr(g.cat.name()), dx, c.y, FONT_PX, ui::TEXT_DIM);
        ui.text(
            &ui.fit(tr(g.title), dw, FONT_PX),
            dx,
            c.y + 13.0,
            FONT_PX,
            ui::ACCENT,
        );
        let mut y = c.y + 29.0;
        for l in ui::wrap(ui, tr(g.desc), dw, FONT_PX).iter().take(3) {
            ui.text(l, dx, y, FONT_PX, ui::TEXT);
            y += 13.0;
        }
        let what = match g.reward {
            Reward::Light(n) => fill(s.r_light, &[&n]),
            Reward::Seeds(n) => fill(s.village_seeds, &[&n]),
            Reward::Decor(i) => fill(s.r_decor, &[&tr(crate::village::decor_name(i))]),
            Reward::Fur(n) => tr(crate::journal::fur_name(n)).to_string(),
            Reward::Charm(i) => fill(s.r_charm, &[&tr(crate::equipment::CHARMS[i].name)]),
        };
        let what = fill(s.j_reward, &[&what]);
        let by = c.bottom() - 39.0;
        ui.text(&ui.fit(&what, dw, FONT_PX), dx, by, FONT_PX, ui::TEXT);
        // 상태: 받은 뒤엔 보상이 어디 있는지(가방·마을·큰 나무…, 0050). 두 줄까지.
        let state = if g.claimed(&self.profile) {
            tr(crate::journal::reward_place(g.reward)).to_string()
        } else if g.ready(&self.profile) {
            vfill(s.j_claim, &[Verb::Confirm])
        } else {
            fill(s.j_progress, &[&g.progress(&self.profile), &g.goal])
        };
        let sc = if g.ready(&self.profile) {
            ui::ACCENT
        } else {
            ui::TEXT_DIM
        };
        for (k, l) in ui::wrap(ui, &state, dw, FONT_PX).iter().take(2).enumerate() {
            ui.text(l, dx, by + 13.0 + k as f32 * 13.0, FONT_PX, sc);
        }
    }

    fn draw_map(&self, ui: &Ui, c: Rect) {
        use crate::adventure::Place;
        let s = i18n::s();
        let biome = tr(self.world.biome.name());
        let title = match self.world.adv.place {
            Place::Arena => fill(s.map_floor, &[&biome]),
            Place::Field => fill(s.map_field, &[&biome]),
            Place::Camp => fill(s.map_camp, &[&self.world.adv.stage]),
            Place::Village => s.village_name.to_string(),
            Place::Dungeon => fill(
                s.map_dungeon,
                &[
                    &(self.world.adv.dungeon.as_ref().map_or(0, |d| d.depth) + 1),
                    &biome,
                ],
            ),
        };
        ui.text(&ui.fit(&title, c.w, FONT_PX), c.x, c.y, FONT_PX, ui::ACCENT);
        let area = Rect::new(c.x, c.y + 16.0, c.w, c.h - 32.0);
        match self.world.adv.place {
            Place::Dungeon => self.draw_dungeon_map(ui, area),
            _ => self.draw_room_map(ui, area),
        }
        let note = if self.world.adv.place == Place::Arena {
            s.map_note
        } else {
            s.map_legend
        };
        ui.text(
            &ui.fit(note, c.w, FONT_PX),
            c.x,
            area.bottom() + 3.0,
            FONT_PX,
            ui::TEXT_DIM,
        );
    }

    /// 아레나·필드: 지금 지도 전체를 칸에 맞게 줄여서. 벽·소품·특별한 칸·적·나.
    fn draw_room_map(&self, ui: &Ui, area: Rect) {
        use crate::map::Cell;
        let room = &self.world.room;
        let b = room.inner;
        let (bw, bh) = match &room.grid {
            Some(g) => (g.w as f32 * 16.0, g.h as f32 * 16.0),
            None => (t::VIEW_W, t::VIEW_H),
        };
        let k = (area.w / bw).min(area.h / bh);
        let origin = vec2(
            area.x + (area.w - bw * k) / 2.0,
            area.y + (area.h - bh * k) / 2.0,
        );
        let m = |p: Vec2| origin + p * k;
        let wall = Color::new(0.30, 0.22, 0.20, 1.0);
        let floor = Color::new(0.12, 0.09, 0.14, 1.0);
        ui.rect(Rect::new(origin.x, origin.y, bw * k, bh * k), wall);
        match &room.grid {
            Some(g) => {
                let cell = 16.0 * k;
                for y in 0..g.h {
                    for x in 0..g.w {
                        let col = match g.get(x, y) {
                            Cell::Wall => continue,
                            Cell::Save => Color::new(0.45, 0.95, 0.5, 1.0),
                            Cell::Entrance | Cell::Exit => Color::new(0.02, 0.02, 0.02, 1.0),
                            _ => floor,
                        };
                        let p = m(vec2(x as f32 * 16.0, y as f32 * 16.0));
                        ui.rect(Rect::new(p.x, p.y, cell + 0.05, cell + 0.05), col);
                    }
                }
            }
            None => {
                ui.rect(
                    Rect::new(origin.x + b.x * k, origin.y + b.y * k, b.w * k, b.h * k),
                    floor,
                );
            }
        }
        for p in &room.solids {
            if room.grid.is_none() {
                ui.rect(
                    Rect::new(origin.x + p.x * k, origin.y + p.y * k, p.w * k, p.h * k),
                    wall,
                );
            }
        }
        for e in &self.world.enemies {
            let p = m(e.pos);
            ui.rect(
                Rect::new(p.x - 1.0, p.y - 1.0, 2.0, 2.0),
                crate::world::enemy_color(e.kind),
            );
        }
        let p = m(self.world.player.pos);
        ui.rect(Rect::new(p.x - 1.5, p.y - 1.5, 3.0, 3.0), WHITE);
    }

    /// 던전: 들른 방만 칸으로, 문은 짧은 선. 지금 방은 강조.
    fn draw_dungeon_map(&self, ui: &Ui, area: Rect) {
        use crate::dungeon::{RoomKind, DIRS};
        let Some(d) = &self.world.adv.dungeon else {
            return;
        };
        let (cw, ch, gap) = (30.0, 18.0, 6.0);
        let origin = vec2(area.center().x - 2.5 * (cw + gap), area.y + 2.0);
        let at =
            |r: (i32, i32)| origin + vec2(r.0 as f32 * (cw + gap), r.1 as f32 * (ch + gap) * 0.62);
        for r in d.rooms.iter().filter(|r| r.visited) {
            let p = at(r.at);
            let rect = Rect::new(p.x, p.y, cw, ch * 0.62);
            let current = r.at == self.world.adv.current_room;
            let fill_c = match r.kind {
                RoomKind::Boss => Color::new(0.45, 0.15, 0.18, 1.0),
                RoomKind::Treasure => Color::new(0.45, 0.38, 0.15, 1.0),
                RoomKind::Rest => Color::new(0.18, 0.38, 0.22, 1.0),
                _ => Color::new(0.16, 0.12, 0.18, 1.0),
            };
            ui.rect(rect, fill_c);
            ui.rect_lines(rect, if current { ui::ACCENT } else { ui::BORDER });
            for (k, (dx, dy)) in DIRS.iter().enumerate() {
                if r.links[k] {
                    let c = rect.center();
                    let e = c + vec2(
                        *dx as f32 * (cw / 2.0 + gap / 2.0),
                        *dy as f32 * (ch * 0.31 + gap * 0.31),
                    );
                    ui.line(
                        c + vec2(*dx as f32 * cw / 2.0, *dy as f32 * ch * 0.31),
                        e,
                        ui::BORDER,
                    );
                }
            }
            if current {
                let c = rect.center();
                ui.rect(Rect::new(c.x - 1.5, c.y - 1.5, 3.0, 3.0), WHITE);
            }
        }
    }
}

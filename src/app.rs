//! 장면 상태 기계와 메뉴. 인트로 → 타이틀 → (설정) → 게임 ⇄ 일시정지(메뉴·상태·가방·저널·지도).
//!
//! 메뉴는 커서 없이 동작한다(상하좌우 + 확인 + 뒤로, 탭은 Q/E·L1/R1). 탭/클릭은 보조로,
//! 직전 프레임에 그린 요소의 사각형(`hits`)에 맞춰 같은 동작을 한다. (CLAUDE.md "UI는 커서 없이")
//! 아이템·퀘스트·층 구조는 아직 데이터가 없다. 틀만 있고 내용은 다음 단계에서 들어온다(docs/decisions/0007).

use macroquad::prelude::*;

mod after;
mod dream;
mod flow;
mod pick;
mod run;
mod slots;
mod stage;
// 0169 R4: 장면별로 나눴다. 자식은 `use super::*`로 부모의 가져오기·App 필드를 함께 쓴다.
mod album;
mod help;
mod pause;
mod settings;
mod title;
use pause::*;
use settings::*;
use title::*;

use crate::art::Art;
use crate::cutscene::{self, Cutscene};
use crate::equipment::{Character, CHARMS, WEAPONS};
use crate::i18n::{self, fill, tr, Tr};
use crate::input::{self, Actions, Device, Input, Verb};
use crate::render::{self, Renderer};
use crate::scenario::{self, Scenario};
use crate::sfx::Sfx;
use crate::tuning as t;
use crate::tutorial::Tutorial;
use crate::ui::{self, Ui, FONT_PX};
use crate::world::World;
/// 안내 글의 동사 자리(`{0}`, `{1}` …)를 지금 장치(키보드·패드·터치)의 이름으로 채운다.
fn vfill(t: &str, verbs: &[Verb]) -> String {
    // 키·버튼 이름은 키캡으로(0153). 터치의 "탭"·"바깥 탭" 같은 손짓은 그대로.
    let names: Vec<String> = verbs
        .iter()
        .map(|v| {
            let n = input::verb(*v);
            if input::verb_is_button(*v) {
                ui::key(n)
            } else {
                n.to_string()
            }
        })
        .collect();
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
    /// 라이선스(설정 안): 제3자 고지 글을 굴려 본다. 돌아가면 설정(2026-10-29, OFL·MIT·Apache 고지를 앱 안에서 볼 수 있게).
    Licenses {
        back: Back,
    },
    /// 개발용 시나리오 목록. 운영 빌드(`--no-default-features`)에는 들어가는 길이 없다(0193).
    #[cfg_attr(not(feature = "dev"), allow(dead_code))]
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
    /// 저장 슬롯(타이틀, docs/decisions/0078).
    Slots,
}

/// 장면이 무엇 위에 떠 있나(0235 R21): 월드를 그리는지, 타이틀 바탕을 그리는지, 타이틀 곡인지가 이것 하나로 갈린다.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Layer {
    /// 타이틀 쪽, 제 바탕을 스스로 그린다(첫 화면·앨범). 타이틀 곡.
    Title,
    /// 타이틀 쪽, 타이틀 바탕 그림 위(0083). 타이틀 곡.
    TitleArt,
    /// 게임 월드 위(창이면 월드를 뒤에 그린다). 장소의 곡.
    World,
}

impl Scene {
    /// `_` 없이 모든 장면을 적는다 — 새 장면을 더하면 컴파일러가 여기를 묻는다(예전에는 세 `matches!` 목록이라
    /// 빠뜨려도 컴파일됐다. 라이선스 화면 때 셋 모두에 손으로 넣었다).
    fn layer(self) -> Layer {
        match self {
            Scene::Intro | Scene::Album => Layer::Title,
            Scene::Title | Scene::Slots | Scene::Dev => Layer::TitleArt,
            Scene::Settings { back } | Scene::Controls { back } | Scene::Licenses { back } => {
                match back {
                    Back::Title => Layer::TitleArt,
                    Back::Pause => Layer::World,
                }
            }
            Scene::Cutscene
            | Scene::Game
            | Scene::Pause
            | Scene::Route
            | Scene::Shop
            | Scene::KnotPick
            | Scene::Decor
            | Scene::Tree
            | Scene::DreamPick
            | Scene::CharPick => Layer::World,
        }
    }
}

/// 컷씬이 끝난 뒤 갈 곳.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AfterCut {
    Game,
    Title,
    Village,
    /// 실에 잠겼다(0163): 장면이 끝나면 맡겨 둔 쉼터로 되살아난다.
    Respawn,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Back {
    Title,
    Pause,
}

/// 런 통계를 기록에 쌓을 때의 기준(저널, docs/decisions/0048). 월드가 새로 만들어지면 0부터.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct Tally([u32; 10]);

impl Tally {
    const KEYS: [&'static str; 10] = [
        crate::journal::stat::KILLS,
        crate::journal::stat::EMERGE_KILLS,
        crate::journal::stat::PARRIES,
        crate::journal::stat::DRILLS,
        crate::journal::stat::SLAMS,
        crate::journal::stat::FINISHERS,
        crate::journal::stat::EATEN,
        crate::journal::stat::CHESTS,
        crate::journal::stat::SPRINGS,
        crate::journal::stat::HEAVY,
    ];

    fn of(w: &World) -> Self {
        Self([
            w.stats.kills,
            w.stats.emerge_kills,
            w.stats.parries,
            w.stats.drills,
            w.stats.slams,
            w.stats.finisher_hits,
            w.stats.items_used,
            w.stats.chests,
            w.stats.springs,
            w.stats.heavy_hits,
        ])
    }
}

pub enum Item {
    /// equipment::WEAPONS 번호.
    Weapon(usize),
    /// equipment::CHARMS 번호. 끼고 있는지는 world.inv.charms.
    Charm(usize),
}

/// 가방. 지금은 무기만 들어간다. 그 밖의 아이템은 2단계(방 클리어 보상)에서.
/// 가방은 캐릭터마다(0246): 그 캐릭터가 쓸 수 있는 무기·장신구만 든다. 다른 캐릭터 것은 기록에 남아 그 캐릭터로 내려갈 때 들고 간다.
pub struct Inventory {
    pub owner: Character,
    pub slots: Vec<Option<Item>>,
    pub cols: usize,
    /// 장착 중인 무기(WEAPONS 번호). 맨손이면 None.
    pub equipped: Option<usize>,
}

impl Inventory {
    /// `weapons` 가운데 주인이 쓰는 것만 넣고 그 첫째를 장착한다. 없으면 맨손.
    fn new(owner: Character, weapons: &[usize]) -> Self {
        let mut inv = Self {
            owner,
            slots: (0..15).map(|_| None).collect(),
            cols: 5,
            equipped: None,
        };
        for w in weapons {
            inv.add(*w);
        }
        inv.equipped = weapons.iter().copied().find(|&w| inv.fits(w));
        inv
    }

    /// 이 가방 주인이 쓰는 무기 종류인가.
    fn fits(&self, w: usize) -> bool {
        WEAPONS[w].kind == self.owner.weapon_kind()
    }

    /// 이 가방 주인이 낄 수 있는 장신구인가(`only`가 다른 캐릭터면 아니다).
    fn fits_charm(&self, c: usize) -> bool {
        CHARMS[c].only.is_none_or(|o| o == self.owner)
    }

    fn has(&self, w: usize) -> bool {
        self.slots
            .iter()
            .any(|s| matches!(s, Some(Item::Weapon(x)) if *x == w))
    }

    /// 장신구를 빈 칸에 넣는다. 이미 있으면 그대로.
    fn add_charm(&mut self, c: usize) {
        if !self.fits_charm(c)
            || self
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
        if !self.fits(w) || self.has(w) {
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

/// 직전 프레임에 그린, 탭/클릭에 반응하는 요소.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Hit {
    TitleItem(usize),
    SlotPlay(usize),
    SlotErase(usize),
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
    /// 안내 주제·설명(누르면 다음 쪽, 0178).
    HelpItem(usize),
    HelpPage,
    /// 상태 탭 ▲(-1)·▼(+1).
    StatusScroll(i8),
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
    /// 앨범 ▲▼(누르면 한 줄 올리거나 내린다).
    AlbumRow(bool),
    /// 실뜨기: 단계 내리기·올리기·시작.
    KnotLeft,
    KnotRight,
    KnotGo,
    /// 큰 나무의 재주 칸(자리, 갈래).
    TreeCell(usize, usize),
    /// 큰 나무의 캐릭터 탭(0 Naïl, 1 모모, 0142).
    /// 목록의 ▲▼(0198·0199): -1 위, 1 아래. 그 화면의 목록이 한 칸 굴린다.
    Scroll(i8),
    TreeTab(usize),
    /// 실의 꿈 줄.
    DreamRow(usize),
    /// 캐릭터 고르기 칸.
    CharCard(usize),
}

pub struct App {
    scene: Scene,
    intro_t: f32,
    title_cursor: usize,
    /// 저장 슬롯(0078): 지금 슬롯 밖의 기록(요약·비었나), 창의 커서·단추(0 이어하기, 1 지우기)·지우기 대기.
    slot_cache: Vec<crate::profile::Profile>,
    slots_cursor: usize,
    slots_col: usize,
    slots_armed: Option<usize>,
    /// 눌린 단추(슬롯, 단추, 누른 시각): 단추감(0157). 잠깐 내려앉았다가 그 일을 한다.
    slots_press: Option<(usize, usize, f64)>,
    /// 슬롯 창이 이어하기로 열렸나(false = 시작하기).
    slots_load: bool,
    /// 쉼터부터 이어하는 중(0080): `start`가 새 런 셈·이야기 한마디·이어하기 지우기를 건너뛴다.
    resuming: bool,
    /// 엔딩 뒤 마을로 올라오는 중(0204): 마을의 이야기 한마디는 땅 위로 튀어나오고 잠깐 뒤에(튀어나오는 모습을 가리지 않게).
    /// 값은 튀어나온 뒤 지난 시간.
    story_after_ascent: Option<f32>,
    pause_tab: usize,
    pause_cursor: usize,
    settings_cursor: usize,
    inv_cursor: usize,
    journal_cursor: usize,
    bestiary_cursor: usize,
    /// 도감 설명 쪽(0 이야기 · 1 공격 · 2 요령). 칸마다 두 줄로 자르던 것을 쪽으로 나눴다(0051).
    bestiary_page: usize,
    help_cursor: usize,
    help_page: usize,
    /// 상태 탭을 내린 만큼(논리 px)과 내릴 수 있는 끝. 끝은 그릴 때 잰다.
    status_scroll: f32,
    status_max: f32,
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
    /// 저널 목록의 창(0199).
    journal_view: crate::ui::ListView,
    /// 굴리는 목록들의 창(0199): 설정·안내 주제·도감·개발 목록.
    settings_view: crate::ui::ListView,
    help_view: crate::ui::ListView,
    /// 라이선스 글을 창 폭에 맞춰 접은 줄(처음 열 때 한 번)과 굴리는 창.
    license_lines: Vec<String>,
    license_view: crate::ui::ListView,
    bestiary_view: crate::ui::ListView,
    dev_view: crate::ui::ListView,
    /// 이번 프레임의 굴리기 입력(논리 px 끌기, 칸: 휠·▲▼, 0199).
    scroll_in: (f32, i32),
    journal_check: f32,
    inventory: Inventory,
    sfx: Sfx,
    music: crate::music::Music,
    /// 환경음 층(0234): 음악 밑의 바람·새·물방울·땅 울림. 크기는 설정의 효과음.
    amb: crate::music::Music,
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
    /// 전환이 끝난 뒤에 할 일(깨어난 뒤·쉼터 도착·런 첫 한마디·길 지도, 0230 R15).
    after: after::After,
    /// 설정 "기록 지우기"를 한 번 눌렀다(한 번 더 누르면 지운다).
    erase_armed: bool,
    /// 이 컷씬이 끝나면 갈 곳(엔딩 = 런의 끝).
    after_cutscene: AfterCut,
    /// 마을 꾸미기 가게의 커서.
    decor_cursor: usize,
    /// 재주 창 커서(자리, 갈래).
    tree_cursor: (usize, usize),
    /// 큰 나무에서 보고 있는 캐릭터의 재주 표(0142).
    tree_char: Character,
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
    /// 앨범에서 보이는 첫 줄(그림이 두 줄을 넘으면 굴린다, 0085).
    /// 앨범의 줄 창(0199, 다섯 칸이 한 줄).
    album_rows: crate::ui::ListView,
    /// 클립 찍기 전용 자동 조종(`--features clip`, 0228).
    #[cfg(feature = "clip")]
    pilot: crate::autopilot::Pilot,
}

impl App {
    pub fn new(
        sfx: Sfx,
        mut music: crate::music::Music,
        fonts: render::Fonts,
        art: Option<Art>,
    ) -> Self {
        let saved = crate::prefs::load();
        let settings = Settings {
            volume: saved.map(|p| p.volume).unwrap_or(7),
            music: saved.map(|p| p.music).unwrap_or(7),
            shake: saved.map(|p| p.shake).unwrap_or(10),
            effects: saved.map(|p| p.effects).unwrap_or(10),
            tells: saved
                .map(|p| p.tells)
                .unwrap_or(crate::prefs::TELLS_DEFAULT),
            debug_overlay: false,
            lang: saved.and_then(|p| p.lang),
            fullscreen: crate::DESKTOP && saved.is_some_and(|p| p.fullscreen),
        };
        if settings.fullscreen {
            macroquad::window::set_fullscreen(true);
        }
        if let Some(p) = saved {
            crate::input::set_bindings(p.bindings);
        }
        if let Some(l) = settings.lang {
            i18n::set_lang(l);
        }
        sfx.set_volume(settings.volume_f());
        music.set_volume(settings.music as f32 / 10.0);
        let mut amb = crate::music::Music::ambience();
        amb.set_volume(settings.volume_f());
        let mut world = World::new();
        world.debug.overlay = settings.debug_overlay;
        Self {
            scene: Scene::Intro,
            intro_t: 0.0,
            title_cursor: 0,
            slot_cache: (0..crate::profile::SLOTS)
                .map(crate::profile::load_slot)
                .collect(),
            slots_cursor: 0,
            slots_col: 0,
            slots_armed: None,
            slots_press: None,
            slots_load: false,
            resuming: false,
            story_after_ascent: None,
            pause_tab: 0,
            pause_cursor: 0,
            settings_cursor: 0,
            inv_cursor: 0,
            journal_cursor: 0,
            bestiary_cursor: 0,
            bestiary_page: 0,
            help_cursor: 0,
            help_page: 0,
            status_scroll: 0.0,
            status_max: 0.0,
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
            journal_view: Default::default(),
            settings_view: Default::default(),
            help_view: Default::default(),
            license_lines: Vec::new(),
            license_view: Default::default(),
            bestiary_view: Default::default(),
            dev_view: Default::default(),
            scroll_in: (0.0, 0),
            journal_check: 0.0,
            inventory: Inventory::new(Character::Mole, &[]),
            gift: None,
            sfx,
            music,
            amb,
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
            after: after::After::default(),
            erase_armed: false,
            album_cursor: 0,
            album_view: None,
            album_rows: Default::default(),
            after_cutscene: AfterCut::Game,
            decor_cursor: 0,
            tree_cursor: (0, 0),
            tree_char: Character::Mole,
            dream_cursor: 0,
            dream: None,
            dream_lock: 0.0,
            char_cursor: 0,
            knot_level: 0,
            title_note: None,
            #[cfg(feature = "clip")]
            pilot: Default::default(),
        }
    }

    /// 지금 들려야 할 곡·환경음과 낮출지. 규칙은 `music::choose`(시험이 있다), 여기서는 보는 것만 모은다(0235 R21).
    fn music_track(&self) -> crate::music::Choice {
        let w = &self.world;
        crate::music::choose(&crate::music::Listen {
            title: self.scene.layer() != Layer::World,
            alt: self
                .profile
                .count(crate::profile::key::RUNS)
                .is_multiple_of(2),
            place: w.adv.place,
            biome: w.biome,
            stage: w.adv.stage,
            menu: self.scene != Scene::Game,
            hushed: w.juice.hush && self.scene == Scene::Cutscene,
            deep_fight: w.deep.as_ref().is_some_and(|d| d.done_t.is_none()),
            boss: self.dream.is_some() || w.enemies.iter().any(|e| e.is_boss() && !e.asleep),
            roused: w.enemies.iter().any(|e| e.is_boss() && e.roused),
        })
    }

    /// 기록을 저장한다. 이번 런에서 만난 이웃도 함께 남긴다.
    fn save_profile(&mut self) {
        for k in &self.world.seen {
            self.profile.set(&crate::profile::key::mob(k));
        }
        crate::profile::save(&self.profile);
    }

    /// 첫 시작(Normal)만 이야기 기록을 남긴다. 개발 시나리오는 기록을 건드리지 않는다.
    /// 이 무기를 가졌나(기록 깃발이나 이번 런 가방, 0246).
    fn owns_weapon(&self, w: usize) -> bool {
        WEAPONS[w].key.is_some_and(|k| self.profile.flag(k)) || self.inventory.has(w)
    }

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
        self.world.assist = self.profile.flag(crate::profile::key::ASSIST);
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
        if self.input.fullscreen_pressed() {
            self.toggle_fullscreen();
        }
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
        let heard = self.music_track();
        self.music.update(heard.music, heard.ducked, dt);
        self.amb.update(heard.ambience, heard.ducked, dt);
        match self.scene {
            Scene::Game => {
                let heard = self.sfx.played();
                self.update_game(dt);
                if let (Some(w), None) = (self.story_after_ascent, self.world.adv.fade) {
                    let w = w + dt;
                    self.story_after_ascent = if w >= t::ASCENT_STORY_DELAY {
                        self.say_story(crate::story::When::Village);
                        None
                    } else {
                        Some(w)
                    };
                }
                // 게임에서 멈춤 창을 열면 메뉴 소리 하나(다른 창은 말 걸기·상자 소리가 이미 난다).
                if self.scene == Scene::Pause && self.sfx.played() == heard {
                    self.sfx.play(crate::sfx::Id::UiConfirm);
                }
            }
            Scene::Cutscene => {
                let mut a = self.input.take();
                // 건너뛰기 안내를 누르면 뒤로와 같다(터치).
                if a.tap.is_some_and(|p| self.hit_at(p) == Some(Hit::Skip)) {
                    a.cancel = true;
                    a.tap = None;
                }
                if let Some(c) = self.cutscene.as_mut() {
                    c.update(&a, dt, &mut self.world);
                    self.sfx.flush(&mut self.world.juice.audio);
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
                    self.gift_queue.append(&mut self.world.inv.gifts);
                    if c.is_done() {
                        self.cutscene = None;
                        match std::mem::replace(&mut self.after_cutscene, AfterCut::Game) {
                            AfterCut::Game => self.scene = Scene::Game,
                            AfterCut::Title => {
                                self.title_cursor = 0;
                                self.scene = Scene::Title;
                            }
                            AfterCut::Village => {
                                // 엔딩 뒤(0204): 가장 깊은 곳에서 땅 위까지 올라와 뿌리 길 곁으로 튀어나온다.
                                self.story_after_ascent = Some(0.0);
                                self.start(Scenario::Village);
                                self.world.start_ascent();
                            }
                            AfterCut::Respawn if self.world.run_over() => {
                                self.lose_run(flow::back_to_village(self.current))
                            }
                            AfterCut::Respawn => {
                                self.scene = Scene::Game;
                                self.world.start_fade(crate::adventure::Transition::Respawn);
                            }
                        }
                    }
                } else {
                    self.scene = Scene::Game;
                }
            }
            _ => {
                let a = self.input.take();
                let (before, heard) = (self.scene, self.sfx.played());
                let tapped = a.tap.and_then(|p| self.hit_at(p));
                self.update_menu(&a, dt);
                self.menu_sound(&a, tapped, before, heard);
            }
        }
        self.draw();
    }

    /// 메뉴 소리(docs/decisions/0081): 메뉴 장면에서 이번 프레임에 다른 소리(살 때·안 될 때·크기 미리 듣기 …)가
    /// 나지 않았으면 입력을 따라 하나 — 뒤로 > 고르기 > 커서. 화면마다 따로 달지 않는다(새 창도 저절로 난다).
    fn menu_sound(&self, a: &Actions, tapped: Option<Hit>, before: Scene, heard: u32) {
        use crate::sfx::Id;
        if self.sfx.played() != heard || before == Scene::Intro {
            return;
        }
        let back =
            a.cancel || matches!(tapped, Some(Hit::Close)) || (a.menu && self.scene != before);
        let id = if back && before != Scene::Title {
            Id::UiBack
        } else if a.confirm || tapped.is_some() {
            Id::UiConfirm
        } else if a.up || a.down || a.left || a.right {
            Id::UiMove
        } else {
            return;
        };
        self.sfx.play(id);
    }

    // ---- 갱신 ---------------------------------------------------------------

    fn update_game(&mut self, dt: f32) {
        // 지도를 굽는 동안(화면 검정, 0068)은 월드를 멈춘다. 끝나면 밝아짐을 처음부터 본다.
        if self.renderer.baking() {
            self.acc = 0.0;
            let _ = self.input.take();
            return;
        }
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
            #[cfg(feature = "clip")]
            {
                a = self.pilot.act(&self.world, a, t::TICK_DT);
            }
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
            self.world.juice.effects = self.settings.effects_f();
            self.world.juice.tells = self.settings.tells_f();
            self.world.tick(&a);
            self.sfx.flush(&mut self.world.juice.audio);
            self.acc -= t::TICK_DT;
        }
        if self.world.player.is_alive() {
            self.run_time += dt;
            self.death_noted = false;
        } else {
            self.death_t += dt;
            if !self.death_noted {
                self.death_noted = true;
                if self.world.run_over() {
                    // 런이 끝나는 쓰러짐: 땅이 울고 뿌리가 감는다(보통 쓰러짐과 다른 소리·그림, draw_dead).
                    self.sfx.play(crate::sfx::Id::Rumble);
                }
                self.on_death();
            }
        }
        // 전환이 끝난 뒤에 할 일(0230 R15). 문은 일마다 새로 잰다 — 앞의 한마디가 컷씬을 열면 뒤의 일은 기다린다.
        for kind in after::ORDER {
            let gate = after::Gate {
                settled: self.world.adv.fade.is_none(),
                alive: self.world.player.is_alive(),
                in_game: self.scene == Scene::Game,
                cutscene: self.cutscene.is_some(),
            };
            match self.after.take(kind, gate) {
                // 깨어나는 어두워짐·밝아짐이 끝나면 이웃의 한마디(갈색 뿌리가 데려왔다, 0037).
                Some(after::Do::SayWake) => {
                    self.say_story(crate::story::When::Wake);
                }
                Some(after::Do::CampArrive(n)) => self.play(cutscene::camp_arrive(&self.world, n)),
                Some(after::Do::SayRunStart) => {
                    self.say_story(crate::story::When::RunStart);
                }
                Some(after::Do::OpenRoute) => self.open_route(true),
                None => {}
            }
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
            let fall = flow::fall(
                self.current,
                self.world.adv.place == crate::adventure::Place::Arena,
                self.world.player.is_alive(),
                self.world.run_over(),
            );
            match fall {
                flow::Fall::RunLost { to_village } => self.lose_run(to_village),
                flow::Fall::Respawn { wake_line } => {
                    self.world.begin_respawn();
                    self.death_t = 0.0;
                    self.after.set_wake(wake_line);
                }
                flow::Fall::Restart => self.start_game(),
            }
        } else if open_pause {
            self.open_pause();
        } else if open_map {
            self.open_pause();
            self.pause_tab = TAB_MAP;
        }
    }

    /// 실뿌리가 다 감긴 뒤 또 쓰러졌다(0210): 런이 끝난다. 쉼터 뒤에 주운 반딧불은 마을로 가지 않는다(잃는다).
    /// 이야기 런은 마을로(뿌리가 데려다 놓는다), 오늘의 굴은 타이틀로. 맡겨 둔 반딧불·기록은 그대로.
    fn lose_run(&mut self, to_village: bool) {
        self.world.inv.glowbugs = 0;
        self.clear_run();
        if self.recording() {
            self.profile.add(crate::profile::key::RUNS_LOST, 1);
            self.save_profile();
        }
        if to_village {
            self.start(Scenario::Village);
            self.world.juice.toast = Some((|s| s.toast_run_lost, 5.0));
        } else {
            self.title_cursor = 0;
            self.title_note = Some((|s| s.toast_run_lost, 5.0));
            self.scene = Scene::Title;
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
        self.status_scroll = 0.0;
        self.scene = Scene::Pause;
    }

    /// 시나리오로 새 런을 시작한다. Normal과 CutsceneOnly는 오프닝 컷씬을 먼저 재생한다.
    fn start(&mut self, s: Scenario) {
        // 개발(0204): 엔딩 뒤 마을로 올라오는 장면만.
        if s == Scenario::Ascent {
            self.story_after_ascent = Some(0.0);
            self.start(Scenario::Village);
            self.world.start_ascent();
            return;
        }
        if s != Scenario::Village {
            self.story_after_ascent = None;
        }
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
        self.journal_view = Default::default();
        let ch = flow::character(s, self.picked());
        self.world.character = ch;
        self.world.player.kit = ch;
        // 무기는 스토리를 따라 하나씩(docs/decisions/0012). 첫 시작·튜토리얼은 맨손에서 출발해
        // 튜토리얼에서 두더지 발톱을 줍는다. 개발 시나리오는 테스트를 위해 기본 발톱(또는 전부)을 들고 시작.
        let start = ch.starting_weapon();
        let veteran = flow::veteran(s, self.profile.flag(crate::profile::key::TUTORIAL));
        self.note_run_start(s);
        self.death_noted = false;
        self.after.clear();
        self.apply_assist();
        self.seed_run_from_profile(s, ch);
        self.fill_inventory(s, ch, start, veteran);
        self.gift = None;
        self.gift_queue.clear();
        self.apply_equipped();
        self.run_time = 0.0;
        self.acc = 0.0;
        self.death_t = 0.0;
        self.tutorial = None;
        self.enter_first_scene(s, veteran);
    }

    /// 런 시작의 기록: 첫 시작은 런 수 +1, 새 런·마을이면 쉼터 이어하기 끝(0080), 오늘의 굴은 묵은 기록 정리.
    fn note_run_start(&mut self, s: Scenario) {
        if s == Scenario::Normal && !self.resuming {
            self.profile.add(crate::profile::key::RUNS, 1);
            crate::profile::save(&self.profile);
        }
        // 새 런을 시작하거나 마을로 돌아오면 쉼터 이어하기는 끝났다(0080).
        if matches!(s, Scenario::Normal | Scenario::Village) && !self.resuming {
            self.clear_run();
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
    }

    /// 기록(프로필)이 새 월드에 주는 것: 도감·시든 꽃잎, 실뜨기 단계, 저널 보상(장신구 풀·털빛), 재주, 보스 깃발과 열린 길.
    /// 시나리오가 만든 월드 위에 얹는다 — `scenario::build`는 기록을 모른다.
    fn seed_run_from_profile(&mut self, s: Scenario, ch: Character) {
        // 도감은 런을 넘어 남는다. 시든 꽃잎(0164)도.
        self.world
            .story
            .set_petal_found(self.profile.flag(crate::profile::key::PETAL));
        for k in crate::species::BESTIARY {
            if self.profile.flag(&crate::profile::key::mob(k)) && !self.world.seen.contains(&k) {
                self.world.seen.push(k);
            }
        }
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
        self.world.inv.charm_pool = crate::journal::charm_pool(&self.profile, ch);
        self.world.inv.fur = crate::journal::fur(&self.profile);
        self.tally = Tally::of(&self.world);
        self.journal_ready = crate::journal::ready_count(&self.profile);
        // 큰 나무의 재주(0047): 튜토리얼만 빼고 모든 시작에(개발 시나리오도 고른 재주로 시험한다).
        if s != Scenario::Tutorial {
            self.world.player.talents = self.talents();
            // 큰 나무 덤(0246)이 최대 체력을 올렸으면 그만큼 채워서 시작(시나리오가 정한 체력은 그대로 두고 더하기만).
            let before = self.world.player.stats.max_hp;
            self.world.refresh_stats();
            let gained = self.world.player.stats.max_hp - before;
            if gained > 0 {
                self.world.player.heal(gained);
            }
        }
        // 모모(0046): 이긴 수는 기록에서(빈 자리의 꽃잎). 첫 길 끝에서 싸우는 건 첫 시작에서만, 돌아오기 전까지.
        // 개발 시나리오 "모모"는 시나리오가 정한 값을 그대로 쓴다.
        if matches!(s, Scenario::Normal | Scenario::Village) {
            use crate::profile::key;
            // 기록이 이야기에 주는 것을 한 번에(0170 R11). 모모로 내려가면 모모 보스는 없다(0056); 도토(0052)·콩이(0054)·토끼풀(0165)은
            // 첫 시작에서만, 돌아오기(기억해 내기) 전까지; 굴토끼 옛 굴(0165)은 첫 시작은 모모가 돌아온 뒤, 오늘의 굴엔 없고, 개발 시나리오는 늘.
            let normal = s == Scenario::Normal;
            self.world.story.seed(crate::world::StorySeed {
                momo_wins: self.profile.count(key::MOMO_WINS).min(t::MOMO_FIGHTS) as u8,
                momo_fight: normal && !self.profile.flag(key::MOMO_BACK) && ch != Character::Momo,
                doto_fight: normal && !self.profile.flag(key::DOTO_MEMORY),
                kongi_fight: normal && !self.profile.flag(key::KONGI_BACK),
                clover_fight: normal && !self.profile.flag(key::CLOVER_BACK),
                clover_back: self.profile.flag(key::CLOVER_BACK),
                warren_open: match s {
                    Scenario::Normal => self.profile.flag(key::MOMO_BACK),
                    Scenario::Daily => false,
                    _ => true,
                },
            });
            // 개발 시나리오는 길을 이미 짰으니(쉼터에서 시작) 갈래를 넣어 다시 짠다.
            if self.world.story.warren_open()
                && self
                    .world
                    .adv
                    .route
                    .as_ref()
                    .is_some_and(|r| r.at.is_none())
            {
                self.world.reroll_route();
            }
        }
    }

    /// 들고 시작하는 무기·장신구(0012·0050·0053): 시나리오별 기본 + 기록으로 받은 것.
    fn fill_inventory(&mut self, s: Scenario, ch: Character, start: usize, veteran: bool) {
        self.inventory = match s {
            Scenario::Normal if veteran => Inventory::new(ch, &[start]),
            Scenario::Normal | Scenario::Tutorial => Inventory::new(ch, &[]),
            Scenario::AllClaws => {
                // 그 캐릭터 무기 전부(가방이 주인 것만 받는다, 0246). 첫째(낀 것)는 시작 무기.
                let all: Vec<usize> = std::iter::once(start).chain(0..WEAPONS.len()).collect();
                Inventory::new(ch, &all)
            }
            Scenario::Gear => {
                let mut inv = Inventory::new(ch, &[start]);
                for c in 0..CHARMS.len() {
                    inv.add_charm(c);
                }
                inv
            }
            _ => Inventory::new(ch, &[start]),
        };
        // 기록으로 가진 무기(수수의 새싹 발톱 0053·콩이의 바위 발톱·큰 나무의 독 발톱 0052, 모모·도토 무기 0246)는
        // 런마다 가방에(끼는 건 가방에서, 가방이 주인 것만 받는다). 맨손 시작(튜토리얼 전)은 빼고.
        for (w, wp) in WEAPONS.iter().enumerate() {
            if wp.key.is_some_and(|k| self.profile.flag(k)) && self.inventory.equipped.is_some() {
                self.inventory.add(w);
            }
        }
        // 이 런의 길(상자·가게)에서 찾을 무기: 이 캐릭터 것 중 아직 없는 것(0246).
        self.world.inv.road_weapons = (0..WEAPONS.len())
            .filter(|&w| {
                WEAPONS[w].found_on_road && self.inventory.fits(w) && !self.inventory.has(w)
            })
            .collect();
        // 저널로 받은 장신구는 런마다 가방에 들고 시작한다(0050). 상자에서는 다시 안 나온다.
        if s != Scenario::Tutorial {
            for c in crate::journal::owned_charms(&self.profile, ch) {
                self.inventory.add_charm(c);
                if !self.world.inv.charms_found.contains(&c) {
                    self.world.inv.charms_found.push(c);
                }
            }
        }
    }

    /// 첫 장면: 마을·지름길(숙련자)·오프닝 컷씬·튜토리얼·가장 깊은 곳·실의 꿈·이야기 미리보기.
    fn enter_first_scene(&mut self, s: Scenario, veteran: bool) {
        let bori_back = self.profile.flag(crate::profile::key::BORI_BACK);
        match flow::first(s, veteran, bori_back, self.resuming) {
            flow::First::Village => {
                self.scene = Scene::Game;
                if self.story_after_ascent.is_none() {
                    self.say_story(crate::story::When::Village);
                }
            }
            flow::First::Descend => {
                // 마을 뿌리 길에서 첫 쉼터로 곧장 내려간다(0225, 지름길 0151을 대신한다). 런 첫 한마디는 닿은 뒤에.
                self.world.open_glade();
                self.world.skip_glade_boss();
                self.world.descend_to_first_camp();
                self.scene = Scene::Game;
                self.after.village_run();
            }
            flow::First::Field {
                skip_boss,
                near_entrance,
                say,
            } => {
                self.world.open_glade();
                if skip_boss {
                    self.world.skip_glade_boss();
                }
                if near_entrance {
                    // 쉼터 이어하기는 곧바로 그 쉼터를 다시 세운다(`resume_run`).
                    self.world.start_near_entrance();
                }
                self.scene = Scene::Game;
                if say {
                    self.say_story(crate::story::When::RunStart);
                }
            }
            flow::First::Opening { tutorial } => {
                // 첫 시작은 컷씬 → 플레이 튜토리얼 → 웨이브 1. 튜토리얼이 끝나야 자동 웨이브가 켜진다.
                if tutorial {
                    self.tutorial = Some(Tutorial::new(&mut self.world));
                }
                // 아레나(컷씬만)는 비우고, 필드(첫 시작)는 멀리 자는 적을 둔다. 컷씬 좌표는 왼쪽 위 화면 기준.
                if self.world.adv.place == crate::adventure::Place::Arena {
                    self.world.enemies.clear();
                }
                self.world.wave = 0;
                self.world.player.pos = vec2(40.0, 90.0);
                self.world.snap_cam();
                self.cutscene = Some(cutscene::opening(&self.world));
                self.scene = Scene::Cutscene;
            }
            flow::First::Deepest => {
                // 가장 깊은 곳(0162): 할매와 매듭 셋. 들어서면 할매의 말, 셋 다 끊으면 반쪽 엔딩.
                self.world.enter_deep_now();
                self.scene = Scene::Game;
            }
            flow::First::Dream => {
                self.begin_dream();
                self.scene = Scene::Game;
            }
            flow::First::StoryPreview => {
                self.world.wave = 0;
                self.cutscene = Some(cutscene::story_preview(&self.world));
                self.scene = Scene::Cutscene;
            }
            flow::First::Play { tutorial } => {
                if tutorial {
                    self.tutorial = Some(Tutorial::new(&mut self.world));
                }
                self.scene = Scene::Game;
            }
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
        // 굴리기(0199): 손가락 끌기는 화면 px → 논리 px, 휠·▲▼는 칸. 목록이 있는 화면이 쓴다.
        let mut steps = a.wheel;
        if let Some(Hit::Scroll(d)) = tap_hit {
            steps += d as i32;
        }
        self.scroll_in = (a.drag / self.last_ui.0.max(0.01), steps);
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
                    Some(TitleItem::New) => self.open_slots(false),
                    Some(TitleItem::Continue) => self.open_slots(true),
                    Some(TitleItem::Daily) => {
                        self.knot_level = 0;
                        self.start(Scenario::Daily);
                    }
                    Some(TitleItem::Album) => {
                        self.album_cursor = 0;
                        self.album_rows = Default::default();
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
                    // 기록·설정은 바뀔 때마다 이미 저장했다(0240).
                    Some(TitleItem::Quit) => macroquad::miniquad::window::order_quit(),
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
                self.dev_view
                    .scroll(self.scroll_in, &mut self.dev_cursor, n);
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
                // 끌기·휠·▲▼(0198·0199): 창을 굴리고 커서를 창 안으로.
                if self
                    .settings_view
                    .scroll(self.scroll_in, &mut self.settings_cursor, n)
                {
                    self.erase_armed = false;
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
                let open_licenses = (a.confirm && self.settings_cursor == SETTING_LICENSES)
                    || matches!(tap_hit, Some(Hit::SettingRow(SETTING_LICENSES)));
                if open_licenses {
                    self.settings_cursor = SETTING_LICENSES;
                    self.license_view.first = 0;
                    self.scene = Scene::Licenses { back };
                    return;
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
                let back_chosen = a.cancel || matches!(tap_hit, Some(Hit::Close));
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
            Scene::Licenses { back } => self.update_licenses(a, tap_hit, back),
            Scene::Pause => self.update_pause(a, tap_hit),
            Scene::Route => self.update_route(a, tap_hit),
            Scene::Shop => self.update_shop(a, tap_hit, dt),
            Scene::Album => self.update_album(a, tap_hit, dt),
            Scene::Slots => self.update_slots(a, tap_hit),
            Scene::KnotPick => self.update_knot_pick(a, tap_hit),
            Scene::Decor => self.update_decor(a, tap_hit, dt),
            Scene::Tree => self.update_tree(a, tap_hit, dt),
            Scene::DreamPick => self.update_dream_pick(a, tap_hit, dt),
            Scene::CharPick => self.update_char_pick(a, tap_hit),
            Scene::Game | Scene::Cutscene => {}
        }
    }

    fn draw(&mut self) {
        let in_world = self.scene.layer() == Layer::World;
        let alpha = if self.scene == Scene::Game {
            self.acc / t::TICK_DT
        } else {
            1.0
        };
        self.renderer.effects = self.settings.effects_f();
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
        // 타이틀과 타이틀에서 여는 창은 같은 바탕 그림(0083).
        let title_bg = self.scene.layer() == Layer::TitleArt;
        if title_bg {
            self.renderer.draw_title_art(&ui, get_time() as f32);
        }

        match self.scene {
            Scene::Intro => self.draw_intro(&ui),
            Scene::Title => self.draw_title(&ui),
            Scene::Album => self.draw_album(&ui),
            Scene::Slots => self.draw_slots(&ui),
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
            Scene::Licenses { back } => {
                if back == Back::Pause {
                    ui.dim();
                }
                self.draw_licenses(&ui);
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
                // 마을: 마을 씨앗·맡긴 반딧불(오른쪽 위 막대 아래).
                // 깨울 수 있는 재주가 있으면 반딧불 줄이 빛나고, 나무 밑동도 빛난다(0061).
                // 올라오는 단면(0204)이 덮고 있으면 숨긴다(깊이 숫자만).
                let strata = self.world.descent().is_some_and(|d| d.cover > 0.5);
                if self.world.adv.place == crate::adventure::Place::Village && !strata {
                    let ready = self.tree_affordable();
                    if let Some(v) = self.world.adv.village.as_mut() {
                        v.tree_ready = ready;
                    }
                    let n = self.profile.count(crate::village::KEY_SEEDS);
                    let t = fill(i18n::s().village_seeds, &[&n]);
                    ui.text_right(&t, 316.0, 14.0, FONT_PX, ui::TEXT);
                    let l = self.profile.count(crate::talents::KEY_LIGHT);
                    let t = fill(i18n::s().tree_light, &[&l]);
                    let pulse =
                        (0.5 + 0.5 * (self.world.time * 4.0).sin()) * f32::from(u8::from(ready));
                    let c = if ready { ui::ACCENT } else { ui::TEXT };
                    ui.text_right(&t, 316.0, 27.0, FONT_PX, c);
                    // 반딧불 점(글 왼쪽): 깨울 수 있으면 숨 쉬듯 커진다.
                    let x = 316.0 - ui.text_w(&t, FONT_PX) - 6.0;
                    let g = crate::world::GLOW;
                    ui.rect(
                        Rect::new(
                            x - 1.0 - pulse,
                            31.0 - pulse,
                            3.0 + 2.0 * pulse,
                            3.0 + 2.0 * pulse,
                        ),
                        Color::new(g.r, g.g, g.b, 0.35 + 0.35 * pulse),
                    );
                    ui.rect(Rect::new(x, 32.0, 1.0, 1.0), g);
                }
                // 말풍선은 마을 숫자(오른쪽 위) 뒤에: 할매·실 고리 곁에서 숫자가 말풍선 위로 찍혔다(0089).
                if self.gift.is_none() {
                    self.draw_interact_bubble(&ui);
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
                // HUD는 그리지 않는다: 창이 화면 위아래 끝까지 와서(0089) 반딧불·씨앗 숫자와 퀵 슬롯이 창 가장자리에 반쯤 비쳤다.
                // 씨앗은 창 제목 줄에 있다.
                ui.dim();
                self.draw_shop(&ui);
            }
        }
        if let Some(mut o) = self.input.overlay() {
            use crate::input::TouchKind;
            let p = &self.world.player;
            let bag = &self.world.inv.bag;
            let burrow = crate::tuning::BURROW_COOLDOWN * p.stats.burrow_cd;
            // 모든 행동에 대기 표시(사장 2026-10-29): 공격은 휘두르는 동안, 회피는 대기, 필살기는 게이지, 아이템은 먹은 뒤 대기.
            o.button_mut(TouchKind::Attack).ready = p.attack_ready();
            o.button_mut(TouchKind::Dodge).ready = 1.0 - p.burrow_cd / burrow;
            o.button_mut(TouchKind::Special).ready = p.sp / crate::tuning::SP_MAX;
            let item = o.button_mut(TouchKind::Item);
            item.ready = 1.0 - p.item_cd / (crate::tuning::ITEM_COOLDOWN * p.eat_cd_mul);
            item.dim = bag.count(bag.selected) == 0;
            let icon = match bag.selected {
                crate::items::Consumable::Berry => crate::characters::Icon::Berry,
                crate::items::Consumable::Worm => crate::characters::Icon::Worm,
            };
            render::draw_touch_overlay(&o, self.world.character.def(), icon);
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

    /// 뿌리 길에서 고른 캐릭터(0056). 모모는 돌아온 뒤에만.
    fn picked(&self) -> Character {
        use crate::profile::key;
        let code = self.profile.count(key::CHAR_PICK);
        crate::characters::ALL
            .iter()
            .find(|d| d.code as u32 == code && d.available(&self.profile))
            .map_or(Character::Mole, |d| d.id)
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
            // 고른 캐릭터로 마을을 걸으면(모모·도토) 그 이웃은 자리에 따로 서 있지 않는다 — 플레이어가 그 자신이다.
            momo: self.profile.flag(key::MOMO_BACK) && self.picked() != Character::Momo,
            doto: self.profile.flag(key::DOTO_MEMORY) && self.picked() != Character::Doto,
            kongi: self.profile.flag(key::KONGI_BACK),
            clover: self.profile.flag(key::CLOVER_BACK),
            decor,
            // 토끼풀 가족이 돌아오면(0165) 큰 나무의 잎이 한 단계 더.
            growth: crate::village::growth(
                crate::characters::ALL
                    .iter()
                    .map(|d| self.profile.count(&key::clears(d.id)))
                    .sum::<u32>()
                    + u32::from(self.profile.flag(key::CLOVER_BACK)),
            ),
            tree_ready: self.tree_affordable(),
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
        // 창 아래 +4면 글자 아래 2px가 화면 밖(180)으로 잘렸다(0089).
        ui.text_center(&hint, 160.0, panel.bottom() + 1.0, FONT_PX, ui::TEXT_DIM);
        self.close_button(ui, panel);
    }

    /// JRPG식 대화창: 아래 전체 폭, 이름표, 초상 자리, 타자 효과, ▼.
    fn draw_cutscene(&mut self, ui: &Ui) {
        let Some(c) = &self.cutscene else { return };
        // 일러스트는 월드·레터박스 위, 대화창 아래(docs/decisions/0039).
        if let Some(il) = c.illust {
            self.renderer.draw_illust(ui, il, c.illust_t);
        }
        // 보스 이름 카드(0159)는 HUD가 없는 컷씬에서도.
        self.renderer.draw_boss_card(ui, &self.world);
        if let Some(d) = c.dialogue().filter(|d| d.narration) {
            // 내레이션: 이름·초상화 없이 아래쪽에 가운데 맞춤. 상자 대신 화면 아래에서 위로 옅어지는 어둠(0152):
            // 네모 띠는 양옆·위 모서리가 그림 위에 뜬 UI 상자처럼 보였다(사장 "미묘하게 거슬린다"). 글은 테두리로 읽힌다.
            let b = Rect::new(8.0, 132.0, 304.0, 42.0);
            let (top_y, bottom_y) = (112.0, t::VIEW_H);
            let mut y = top_y;
            while y < bottom_y {
                let k = ((y - top_y) / (b.y - top_y)).clamp(0.0, 1.0);
                let a = 0.72 * k * k * (3.0 - 2.0 * k);
                ui.rect(
                    Rect::new(0.0, y, t::VIEW_W, 2.0),
                    Color::new(0.03, 0.02, 0.05, a),
                );
                y += 2.0;
            }
            let ui = &ui.with_outline();
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
        self.gift_queue.append(&mut self.world.inv.gifts);
        if self.gift.is_some() || self.gift_queue.is_empty() {
            return;
        }
        let item = self.gift_queue.remove(0);
        match item {
            crate::world::Gift::Weapon(w) => {
                // 가진 무기는 기록에 남는다(0246): 다음 런부터 그 캐릭터의 가방에. 개발 시나리오는 이번 런에만.
                if let Some(k) = WEAPONS[w].key {
                    if self.recording() && !self.profile.flag(k) {
                        self.profile.set(k);
                        self.save_profile();
                    }
                }
                self.world.inv.road_weapons.retain(|&x| x != w);
                self.inventory.add(w);
                if self.inventory.equipped.is_none() && self.inventory.fits(w) {
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

    /// 받은 것이 지금 캐릭터 것이 아니면 그 주인(가방에 넣지 않고 기록에 남는다, 0246).
    fn gift_owner(&self, item: crate::world::Gift) -> Option<Character> {
        match item {
            crate::world::Gift::Weapon(w) if !self.inventory.fits(w) => crate::characters::ALL
                .iter()
                .find(|d| d.weapon_kind == WEAPONS[w].kind)
                .map(|d| d.id),
            crate::world::Gift::Charm(c) if !self.inventory.fits_charm(c) => CHARMS[c].only,
            _ => None,
        }
    }

    fn draw_gift(&self, ui: &Ui, g: &Gift) {
        let s = i18n::s();
        let (title, name, desc) = match g.item {
            crate::world::Gift::Weapon(w) => (
                if WEAPONS[w].kind == crate::equipment::WeaponKind::Claw {
                    s.gift_title
                } else {
                    s.gift_weapon
                },
                WEAPONS[w].name,
                WEAPONS[w].desc,
            ),
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
            crate::world::Gift::Weapon(w) => draw_weapon_icon(ui, icon, w),
            crate::world::Gift::Charm(c) => draw_charm_icon(ui, icon, CHARMS[c].color),
        }
        ui.text(tr(name), icon.right() + 8.0, r.y + 22.0, FONT_PX, ui::TEXT);
        // 다른 캐릭터 것이면(0246) 설명을 두 줄로 줄이고 누구 가방에 갔는지 아래에.
        let owner = self.gift_owner(g.item);
        let rows = if owner.is_some() { 2 } else { 3 };
        if let Some(o) = owner {
            let note = fill(s.gift_elsewhere, &[&o.name()]);
            let note = ui.fit(&note, r.w - 36.0, FONT_PX);
            ui.text(&note, r.x + 10.0, r.bottom() - 15.0, FONT_PX, ui::ACCENT);
        }
        let lines = ui::wrap(ui, tr(desc), r.right() - icon.right() - 16.0, FONT_PX);
        for (i, l) in lines.iter().take(rows).enumerate() {
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
        // 넓으면 왼쪽 아래 퀵 슬롯 판(칸 둘 + "I 먹기")을 덮었다(일본어, 0089): 그때는 그 판 위로 올린다.
        let quick = fill(
            i18n::s().quick_hint,
            &[&ui::key(crate::input::prompt(crate::input::Act::UseItem))],
        );
        let quick_right = 57.0 + ui.text_w(&quick, FONT_PX) + 6.0;
        let bottom = if 160.0 - w / 2.0 < quick_right {
            148.0
        } else {
            171.0
        };
        let r = Rect::new(160.0 - w / 2.0, bottom - h, w, h);
        ui.rect(r, ui::PANEL);
        ui.rect_lines(r, ui::BORDER);
        for (i, l) in lines.iter().enumerate() {
            ui.text_center(l, 160.0, r.y + 4.0 + i as f32 * 13.0, FONT_PX, ui::TEXT);
        }
    }

    /// 사망 연출: 서서히 어두워지고 글자가 떠오른 뒤, 입력 잠금이 풀리면 안내가 깜빡인다.
    /// 런이 끝나는 쓰러짐(실뿌리 셋째, 0225)은 따로: 화면 가장자리에서 실뿌리가 자라 나일을 감고, 글·안내가 다르다.
    fn draw_dead(&self, ui: &Ui) {
        if self.world.run_over() {
            self.draw_run_lost(ui);
            return;
        }
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

    /// 런이 끝나는 쓰러짐(0225): 더 어둡게 + 화면 가장자리에서 실뿌리 열넷이 나일 쪽으로 자라 감고, 보랏빛 글.
    fn draw_run_lost(&self, ui: &Ui) {
        let tt = self.death_t;
        let k = (tt / t::DEATH_FADE).min(1.0);
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.03, 0.01, 0.04, 0.7 * k),
        );
        // 뿌리가 자라는 정도 0..1(1.4초), 끝은 나일(화면 좌표) 둘레 14px에서 멈춘다.
        let grow = (tt / t::RUN_LOST_GROW).min(1.0);
        let grow = 1.0 - (1.0 - grow) * (1.0 - grow);
        let target = self.renderer.to_view(self.world.player.pos);
        for i in 0..14 {
            let a = i as f32 / 14.0 * std::f32::consts::TAU + 0.37;
            // 가장자리의 출발점: 방향 a로 화면 밖까지 뻗은 자리.
            let dir = vec2(a.cos(), a.sin());
            let reach = (170.0 / dir.x.abs().max(0.01)).min(100.0 / dir.y.abs().max(0.01));
            let start = target + dir * reach;
            let end = target + dir * 14.0;
            let n = 18;
            let shown = (n as f32 * grow) as usize;
            let mut prev = start;
            for j in 1..=shown {
                let f = j as f32 / n as f32;
                let side = vec2(-dir.y, dir.x);
                let wob = (f * 9.0 + i as f32 * 1.7).sin() * 5.0 * (1.0 - f);
                let p = start.lerp(end, f) + side * wob;
                // 뿌리는 끝으로 갈수록 가늘게(3 → 1.5), 어두운 테두리 위에 보랏빛.
                let wd = 3.0 - 1.5 * f;
                let c = Color::new(0.50, 0.30, 0.48, 1.0);
                ui.line_w(prev, p, wd + 1.5, Color::new(0.10, 0.04, 0.10, 1.0));
                ui.line_w(prev, p, wd, c);
                if j % 6 == 3 {
                    // 곁뿌리 하나.
                    let off = p + side * (if i % 2 == 0 { 5.0 } else { -5.0 }) + dir * 3.0;
                    ui.line_w(p, off, 2.0, Color::new(0.10, 0.04, 0.10, 1.0));
                    ui.line_w(p, off, 1.0, c);
                }
                prev = p;
            }
        }
        // 다 자라면 나일 둘레를 감는 고리 셋(실뿌리 셋이 다 감겼다).
        if grow >= 1.0 {
            let wrap = ((tt - t::RUN_LOST_GROW) / 0.5).clamp(0.0, 1.0);
            for r in 0..3 {
                let ry = 4.0 + r as f32 * 3.0;
                let rx = 11.0 - r as f32;
                let n = (24.0 * wrap) as usize;
                let at = |k: usize| {
                    let a = k as f32 / 24.0 * std::f32::consts::TAU;
                    target + vec2(a.cos() * rx, -6.0 + ry + a.sin() * 3.0)
                };
                for k in 0..n {
                    ui.line_w(at(k), at(k + 1), 2.5, Color::new(0.10, 0.04, 0.10, 1.0));
                    ui.line_w(at(k), at(k + 1), 1.2, Color::new(0.62, 0.40, 0.60, 1.0));
                }
            }
        }
        let s = i18n::s();
        let tk = ((tt - 0.6) / 0.6).clamp(0.0, 1.0);
        ui.text_center(
            s.dead_final,
            160.0,
            48.0,
            FONT_PX * 2.0,
            Color::new(0.78, 0.55, 0.9, tk),
        );
        if tt >= t::DEATH_INPUT_LOCK && (get_time() * 2.0) as i64 % 2 == 0 {
            let hint = vfill(s.dead_final_hint, &[Verb::Confirm]);
            ui.text_center(&hint, 160.0, 140.0, FONT_PX, ui::TEXT);
        }
    }
}

#[cfg(test)]
mod bag_tests {
    use super::*;

    /// 가방은 캐릭터마다(0246): 도토 가방엔 발톱·모모 장신구가 안 들어가고, 누구나 장신구·도토 것은 들어간다.
    #[test]
    fn bag_takes_only_what_its_owner_can_use() {
        let doto = Character::Doto.starting_weapon();
        let mut inv = Inventory::new(Character::Doto, &[crate::equipment::STONE_CLAW, doto]);
        assert!(!inv.has(crate::equipment::STONE_CLAW));
        assert!(inv.has(doto));
        // 맞는 무기 가운데 첫째를 낀다.
        assert_eq!(inv.equipped, Some(doto));
        inv.add(crate::equipment::SPROUT_CLAW);
        assert!(!inv.has(crate::equipment::SPROUT_CLAW));
        let momo_only = CHARMS
            .iter()
            .position(|c| c.only == Some(Character::Momo))
            .unwrap();
        let anyone = CHARMS.iter().position(|c| c.only.is_none()).unwrap();
        inv.add_charm(momo_only);
        inv.add_charm(anyone);
        inv.add_charm(crate::equipment::MAPLE_SCARF);
        let charms: Vec<usize> = inv
            .slots
            .iter()
            .filter_map(|s| match s {
                Some(Item::Charm(c)) => Some(*c),
                _ => None,
            })
            .collect();
        assert_eq!(charms, vec![anyone, crate::equipment::MAPLE_SCARF]);
        assert!(CHARMS[crate::equipment::MAPLE_SCARF].only == Some(Character::Doto));
    }
}

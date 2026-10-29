//! 월드 상태와 한 틱의 순서. 모듈들을 엮는 유일한 곳.

use macroquad::prelude::*;

use crate::adventure::{Adventure, Place};
use crate::biome::Biome;
use crate::combat::{dist_to_segment, Feel, Strike};
use crate::enemy::{self, Enemy, Kind, Projectile};
use crate::equipment::Stats;
use crate::i18n::Tr;
use crate::input::Actions;
use crate::items::{Bag, Consumable};
use crate::loot::{self, Loot, LootKind};
use crate::player::{self, Player};
use crate::room::{PropKind, Room};
use crate::sfx;
use crate::tuning as t;

pub mod toss;
pub use toss::Toss;

#[derive(Debug, Clone, Copy)]
pub struct Debug {
    pub overlay: bool,
    pub hitboxes: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum FxKind {
    Ring {
        r0: f32,
        r1: f32,
        color: Color,
    },
    Spark {
        color: Color,
    },
    /// 먼지·연기(0081): 채운 동그라미가 커지며 옅어진다.
    Puff {
        r0: f32,
        r1: f32,
        color: Color,
    },
    /// 흙덩이(0081): 높이 `z`로 튀어 올랐다가 떨어져 한 번 통 튄다. 바닥엔 그림자.
    Clod {
        color: Color,
        z: f32,
        vz: f32,
    },
    /// 반짝임(0081): 십자 빛이 커졌다 줄어든다(주운 것·상자).
    Glint {
        color: Color,
    },
    /// 풀려 흩어지는 보라 실(0081): 쓰러진 이웃에서 실이 빠져나가 위로 떠오르며 사라진다(0037 이야기).
    Thread {
        angle: f32,
        len: f32,
    },
    /// 꽃잎(0159): 팔랑이며 천천히 내려앉는다. `phase`는 흔들림의 위상.
    Petal {
        color: Color,
        phase: f32,
    },
}

pub struct Fx {
    pub pos: Vec2,
    pub vel: Vec2,
    pub time: f32,
    pub dur: f32,
    pub kind: FxKind,
}

/// 토끼풀의 충격파(0165): 발을 구른 자리에서 퍼지는 고리. 고리가 지나는 순간 한 번 아프다(땅속·깡충은 안 맞는다).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Wave {
    pub pos: Vec2,
    pub time: f32,
    pub hit: bool,
}

/// 길의 샘(0159). 들어서는 순간이 아니라 발을 담가야 회복한다(말로만 샘이었다).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spring {
    pub pos: Vec2,
    /// 이미 마셨다(한 번만).
    pub drunk: bool,
}

/// 컷씬이 덮는 어둠(0159): 환경 어둠과 별개로 `at` 둘레 `radius`만 밝고 바깥은 `amount`만큼 검다.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gloom {
    pub amount: f32,
    pub at: Vec2,
    pub radius: f32,
}

impl Gloom {
    pub const NONE: Gloom = Gloom {
        amount: 0.0,
        at: Vec2::ZERO,
        radius: 70.0,
    };
}

/// 컷씬에서 부르는 이펙트 묶음(0159). 스크립트는 자리와 이름만 적고, 어떻게 보이는지는 월드가 안다.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cue {
    /// 실이 풀린다: 쓰러진 자리에서 보라 실이 한 올씩 빠져나가고 흰 고리·반짝임. 돌아오는 이웃.
    Unbind,
    /// 꽃잎이 흩날린다(모모). `true`면 핀 꽃의 분홍, 아니면 시든 꽃잎.
    Petals(bool),
    /// 물든 것이 일어선다: 보라 고리 + 어두운 먼지 + 땅 울림.
    Rise,
    /// 반짝임 한 줌(돌아온 이웃이 나타날 때).
    Sparkle,
    /// 굴 속으로 사라진다(0163, 모모 도망): 흙 튐 + 어두운 먼지 + 실 몇 올, 파고드는 소리.
    Vanish,
    /// 실에 감겨 잠긴다(0163): 보라 고리가 조여 오고 실이 휘감는다, 땅 울림.
    Wrap,
    /// 어둠 속 눈빛(0164): 보라 빛 두 점이 천천히 깜빡인다(나타나기 전).
    Eyes,
    /// 떠난 자리에 꽃잎이 흩날린다(0164). 주울 꽃잎은 남기지 않는다 — 첫 발톱 바로 옆이라 길의 꽃잎과 겹쳤다(0194).
    Leave,
    /// 길에 시든 꽃잎 하나(0190): 이펙트 없이 `at`에 — 모모가 지나간 자국(보리 공터 앞 길, `field::petal_road`).
    /// 밟으면 토스트 + 도감 '???'. 오프닝의 줍는 꽃잎은 이것 하나(0194).
    Trail,
    /// 아기들의 고치가 풀린다(0165): 고치마다 실이 풀리고 반짝임·분홍 꽃잎, 고치가 없어진다.
    Hatch,
}

/// 이웃이 누구인가(그림·말).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Who {
    /// 라쿤 포포: 튜토리얼 안내, 쉼터의 가게.
    Popo,
    /// 들쥐 대장 보리: 튜토리얼 보스였다가 돌아온 이웃(docs/decisions/0032).
    Bori,
    /// 누룩 할매: 반쪽 엔딩 뒤 마을에 온다. 실뜨기를 고른다(docs/decisions/0045).
    Halmae,
    /// 돌아온 모모: 마을의 빈 자리에 앉는다(docs/decisions/0046).
    Momo,
    /// 돌아온 도토: 마을 그루터기 밑(docs/decisions/0052).
    Doto,
    /// 땅강아지 장인 수수: 첫 쉼터. 새싹 발톱을 만들어 준다(docs/decisions/0053).
    Susu,
    /// 돌아온 콩이: 마을 위쪽 바위 곁(docs/decisions/0054).
    Kongi,
    /// 돌아온 토끼풀 + 아기 둘(박하·민들레): 마을 모모 자리 곁(0165).
    Clover,
}

/// 이웃 NPC: 서 있거나(쉼터) 플레이어 곁으로 걸어온다(튜토리얼의 포포). 막지 않는다.
#[derive(Debug, Clone, Copy)]
pub struct Npc {
    pub who: Who,
    pub pos: Vec2,
    pub prev_pos: Vec2,
    pub face_left: bool,
    /// 따라갈 곳(없으면 제자리).
    pub target: Option<Vec2>,
}

impl Npc {
    /// 제자리에 서 있는 이웃(쉼터).
    pub fn standing(who: Who, pos: Vec2) -> Self {
        Self {
            who,
            pos,
            prev_pos: pos,
            face_left: false,
            target: None,
        }
    }
}

/// 얻은 것(앱이 가방에 넣고 획득 창을 띄운다).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gift {
    /// equipment::WEAPONS 번호.
    Weapon(usize),
    /// equipment::CHARMS 번호.
    Charm(usize),
}

/// 도토가 묻은 도토리(0052): 흙이 부풀다가 터진다. 터지는 순간 둘레에 있으면 다친다. 발톱으로 미리 파낼 수 있다.
pub struct Mine {
    pub pos: Vec2,
    pub time: f32,
    pub fuse: f32,
}

/// 심층 싸움방에서 무너지는 천장 돌(0147). `time`이 `ROCK_FALL`이 되면 떨어진다.
pub struct Rockfall {
    pub pos: Vec2,
    pub time: f32,
}

/// 포자버섯이 뿜은 포자 구름(0146). 잠깐 뒤부터 안에 서 있으면 아프다(땅속은 괜찮다).
pub struct Cloud {
    pub pos: Vec2,
    pub time: f32,
}

/// 가장 깊은 곳의 실매듭(0162): 할매와 실로 이어진 매듭. 발톱으로 끊는다. 감기면 가장 가까운 매듭이 다시 조여진다.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Knot {
    pub pos: Vec2,
    pub hp: i32,
    pub cut: bool,
    /// 맞은 직후 떨림(초).
    pub shake: f32,
    /// 같은 공격 번호로 두 번 맞지 않게.
    last_hit_by: u32,
}

/// 할매의 당기기 한 번(0162): 가운데에서 `dir`로 실이 뻗는다. 예고(`DEEP_PULL_WINDUP`) 뒤 `DEEP_PULL_TIME` 동안 뻗은 채.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pull {
    pub dir: Vec2,
    pub len: f32,
    pub time: f32,
    /// 뻗는 틱에 판정을 한 번만.
    pub judged: bool,
    /// 받아쳐 끊겼다(남은 시간 동안 끊긴 실을 그린다).
    pub snapped: bool,
}

/// 가장 깊은 곳(0162): 할매는 때릴 수 없고, 매듭 셋을 끊으면 놓아준다. 세 번 감기면 실에 잠긴다(쉼터로).
#[derive(Debug, Clone, PartialEq)]
pub struct Deep {
    pub center: Vec2,
    pub knots: Vec<Knot>,
    pub time: f32,
    pub pull: Option<Pull>,
    pub next_pull: f32,
    pub wraps: u32,
    /// 감겨 있는 남은 시간(> 0이면 움직이지도 때리지도 못하고 가운데로 끌려간다).
    pub wrapped: f32,
    /// 셋 다 끊은 뒤 흐른 시간(엔딩까지 `DEEP_DONE_DELAY`). None = 아직.
    pub done_t: Option<f32>,
    /// 실에 잠겼다(앱이 쉼터로).
    pub asleep: bool,
    /// 감긴 뒤 한 번 알려 준 힌트.
    hinted: bool,
    /// 방금 당기기가 2단계의 둘째(곧바로 오는) 당기기로 예약됐다(0233).
    echo: bool,
}

impl Deep {
    pub fn new(center: Vec2) -> Self {
        // 매듭 셋: 할매 앞쪽(아래) 반원에, 플레이어가 서는 자리를 비워 둔다.
        let knots = [-0.78f32, 0.0, 0.78]
            .iter()
            .map(|a| {
                let ang = std::f32::consts::FRAC_PI_2 + a;
                Knot {
                    pos: center + vec2(ang.cos() * 92.0, ang.sin() * 52.0),
                    hp: t::DEEP_KNOT_HP,
                    cut: false,
                    shake: 0.0,
                    last_hit_by: 0,
                }
            })
            .collect();
        Self {
            center,
            knots,
            time: 0.0,
            pull: None,
            next_pull: t::DEEP_PULL_EVERY,
            wraps: 0,
            wrapped: 0.0,
            done_t: None,
            asleep: false,
            hinted: false,
            echo: false,
        }
    }

    pub fn cut_count(&self) -> usize {
        self.knots.iter().filter(|k| k.cut).count()
    }

    pub fn all_cut(&self) -> bool {
        self.knots.iter().all(|k| k.cut)
    }

    /// 당기기 사이: 끊은 매듭마다 짧아진다.
    fn pull_every(&self) -> f32 {
        (t::DEEP_PULL_EVERY - t::DEEP_PULL_STEP * self.cut_count() as f32).max(1.2)
    }
}

/// 바닥에 놓인 무기·장신구. 닿으면 얻는다(장신구도 상자·엘리트에서 바닥에 떨어진다 — 사장 2026-10-29 "눈으로 봐야 재밌다").
pub struct Pickup {
    pub pos: Vec2,
    pub item: Gift,
    /// 떨어진 뒤 흐른 시간(떨어지는 연출·줍기 대기).
    pub age: f32,
}

impl Pickup {
    pub fn new(pos: Vec2, item: Gift) -> Self {
        Self {
            pos,
            item,
            age: 0.0,
        }
    }
}

/// 런이 들고 다니는 것(0169 R2): 가방·장신구·반딧불·씨앗·뿌리 심장·털빛·받을 선물. `suspend::RunSave`가 적는 쪽.
#[derive(Default)]
pub struct Inventory {
    /// 모은 반딧불: 기술을 깨우는 데 쓴다(해금은 나중에). 쓰러지면 조금 잃는다.
    pub glowbugs: u32,
    /// 씨앗: 상점 화폐. 쓰러져도 잃지 않는다.
    pub seeds: u32,
    /// 퀵 아이템(산딸기·쥬시 지렁이).
    pub bag: Bag,
    /// 끼고 있는 장신구(equipment::CHARMS 번호). 바꾸면 `refresh_stats`.
    pub charms: Vec<usize>,
    /// 먹은 뿌리 심장 수(최대 체력 +1씩).
    pub root_hearts: i32,
    /// 체력이 깎인 채로 산딸기 없이 지나간 처치 수(불운 보정).
    kills_since_berry: u32,
    /// 이번 틱에 얻은 것. 앱이 가방에 넣고 비운다. 획득물·컷씬·보물 상자 모두 여기로.
    pub gifts: Vec<Gift>,
    /// 지금까지 찾은 장신구(같은 걸 두 번 주지 않게).
    pub charms_found: Vec<usize>,
    /// 상자에서 나올 수 있는 장신구(기본 + 저널로 연 것, 0048)와 Naïl의 털빛(그리기만).
    /// 상자에서 나올 장신구. 기록이 정하므로 앱이 `seed_run_from_profile`에서 넣는다(0170 R-소) — 월드는 기록을 모른다.
    pub charm_pool: Vec<usize>,
    /// 이 런의 길(상자·가게)에서 찾을 수 있는 무기(그 캐릭터 것 중 아직 없는 것, 0246). 앱이 기록에서 넣는다.
    pub road_weapons: Vec<usize>,
    pub fur: u8,
}

/// 런의 이야기 깃발(0169 R2): 어떤 보스와 싸우는 중인지·이겼는지, 길이 열렸는지, 한 번만 알리는 토스트를 알렸는지.
#[derive(Default)]
pub struct Story {
    // 0170 R11: 필드는 모두 비공개. 바깥은 아래 메서드로만 읽고 바꾼다 — 이야기 깃발의 진실은 기록(profile)이고, 여기는 이번 런의 상태.
    /// 모모(0046): 첫 길 끝에서 싸우나(앱이 기록에서 정하고, 모모에게 쓰러지면 이번 런엔 끈다), 지금까지 이긴 수(빈 자리의 꽃잎).
    momo_fight: bool,
    /// 굴토끼 옛 굴(0165)이 길 지도에 열렸나(모모가 돌아온 뒤). 길을 짤 때 본다.
    warren_open: bool,
    momo_wins: u8,
    /// 도토(0052): 둘째 길 끝에서 싸우나(돌아오기 전까지, 쓰러지면 이번 런엔 끈다). 묻힌 도토리.
    doto_fight: bool,
    /// 이번 런에 덤불에 찔린 적이 있나(처음 한 번만 알려 준다, 0149).
    thorn_told: bool,
    /// 흰 실 그물(0165)에 처음 걸렸을 때 한 번 알려 줬나.
    web_told: bool,
    /// 콩이(0054): 셋째 길 끝에서 싸우나(돌아오기 전까지, 쓰러지면 이번 런엔 끈다).
    kongi_fight: bool,
    /// 토끼풀(0165): 굴토끼 옛 굴 끝에서 싸우나(돌아오기 전까지, 쓰러지면 이번 런엔 끈다).
    clover_fight: bool,
    /// 토끼풀 가족이 마을에 돌아왔다(기록 `CLOVER_BACK`): 참 엔딩 에필로그에 한 줄 더.
    clover_back: bool,
    /// 고치를 쳤을 때 한 번 알려 줬나 / 마지막으로 튕긴 공격 번호.
    cocoon_told: bool,
    /// 모모가 떠난 자리의 시든 꽃잎(0164). 밟으면 줍는다(`petal_tick`).
    petals: Vec<Vec2>,
    /// 시든 꽃잎을 주웠다(기록 `key::PETAL`): 모모를 만나기 전에도 도감에 '???' 칸이 선다.
    petal_found: bool,
}

/// 기록(profile)이 새 런의 이야기에 주는 것(0170 R11). 앱이 기록에서 만들어 `Story::seed`에 넣는다. 월드는 기록을 모른다.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StorySeed {
    pub momo_wins: u8,
    pub momo_fight: bool,
    pub doto_fight: bool,
    pub kongi_fight: bool,
    pub clover_fight: bool,
    pub clover_back: bool,
    pub warren_open: bool,
}

/// 보스를 이긴 결과: 이긴 수와 돌아왔나(모모만 세 번).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BossBeaten {
    pub wins: u8,
    pub back: bool,
}

impl Story {
    pub fn seed(&mut self, s: StorySeed) {
        self.momo_wins = s.momo_wins;
        self.momo_fight = s.momo_fight;
        self.doto_fight = s.doto_fight;
        self.kongi_fight = s.kongi_fight;
        self.clover_fight = s.clover_fight;
        self.clover_back = s.clover_back;
        self.warren_open = s.warren_open;
    }

    /// 이번 런에 그 보스와 길 끝에서 싸우나.
    pub fn fight(&self, k: Kind) -> bool {
        match k {
            Kind::Momo => self.momo_fight,
            Kind::Doto => self.doto_fight,
            Kind::Kongi => self.kongi_fight,
            Kind::Clover => self.clover_fight,
            _ => false,
        }
    }

    /// 싸움을 켜고 끈다(쓰러지면 이번 런엔 끈다, 이어하기 복원).
    pub fn set_fight(&mut self, k: Kind, on: bool) {
        match k {
            Kind::Momo => self.momo_fight = on,
            Kind::Doto => self.doto_fight = on,
            Kind::Kongi => self.kongi_fight = on,
            Kind::Clover => self.clover_fight = on,
            _ => {}
        }
    }

    /// 보스를 이겼다: 이번 런엔 다시 안 싸운다. 모모(0046)는 이긴 수를 세고 세 번째에야 돌아온다.
    pub fn beat(&mut self, k: Kind) -> BossBeaten {
        if k == Kind::Momo {
            let wins = self.momo_wins.saturating_add(1);
            self.momo_wins = wins;
            let back = wins as u32 >= t::MOMO_FIGHTS;
            if back {
                self.momo_fight = false;
            }
            BossBeaten { wins, back }
        } else {
            self.set_fight(k, false);
            BossBeaten {
                wins: 1,
                back: true,
            }
        }
    }

    pub fn momo_wins(&self) -> u8 {
        self.momo_wins
    }
    pub fn warren_open(&self) -> bool {
        self.warren_open
    }
    pub fn clover_back(&self) -> bool {
        self.clover_back
    }
    pub fn petal_found(&self) -> bool {
        self.petal_found
    }
    pub fn set_petal_found(&mut self, on: bool) {
        self.petal_found = on;
    }
    /// 바닥에 남은 시든 꽃잎들(0164·0190).
    pub fn petals(&self) -> &[Vec2] {
        &self.petals
    }
    pub fn add_petal(&mut self, at: Vec2) {
        self.petals.push(at);
    }
    pub fn clear_petals(&mut self) {
        self.petals.clear();
    }
}

/// 런 통계(0169 R2): 저널 `stat.*`·튜토리얼 단계가 읽는 수. `app/run.rs`가 저장한다.
#[derive(Default)]
pub struct RunStats {
    /// 튜토리얼용 통계: 패링·아이템 먹기·필살기(튀어 나간 수)·솟구침 명중.
    pub parries: u32,
    pub items_used: u32,
    pub drills: u32,
    pub emerge_hits: u32,
    /// 런 통계(저널, 0048): 연 상자, 들른 샘, 드릴 벽 쿵.
    pub chests: u32,
    pub springs: u32,
    pub slams: u32,
    pub kills: u32,
    /// 솟구침으로 잡은 수(저널용).
    pub emerge_kills: u32,
    /// 콤보 마지막 타가 명중한 수(튜토리얼용).
    pub finisher_hits: u32,
    /// 강공격 명중 수(튜토리얼용, 0161).
    pub heavy_hits: u32,
    /// 굴파기 횟수(튜토리얼용).
    pub burrows: u32,
    /// 부순 것 수(0175).
    pub broken: u32,
}

/// 싸움 중 바닥의 위험물(0169 R2): 도토리 지뢰·포자 구름·떨어지는 돌·고치·충격파·실 가닥.
pub struct Hazards {
    pub mines: Vec<Mine>,
    pub clouds: Vec<Cloud>,
    pub rocks: Vec<Rockfall>,
    /// 다음 돌까지(초).
    rock_cd: f32,
    /// 아기들의 고치(0165): 토끼풀의 끝방에 둘. 때릴 수 없다(발톱이 튕긴다). 장소를 옮기면 없어진다.
    pub cocoons: Vec<Vec2>,
    /// 토끼풀의 충격파(0165).
    pub waves: Vec<Wave>,
    /// 뻗은 실 한 가닥(0165): 시작·끝·흐른 초. 잠깐 그린다.
    pub thread_lash: Option<(Vec2, Vec2, f32)>,
    cocoon_bounce_id: u32,
}

/// 손맛·연출 상태(0169 R2): 멈춤·흔들림·킥(`feel`), 입자, 소리 큐, 토스트, 거절 표시, 이름 카드, 어둠·음악 끊김.
pub struct Juice {
    pub fx: Vec<Fx>,
    /// 입자 효과 설정(0..=1, 0141). 1보다 작으면 새로 생긴 장식 효과(먼지·흙덩이·불똥·반짝임·실)를 그 비율만 남긴다.
    /// 공격 범위를 보여 주는 고리(Ring)는 놀이 정보라 늘 남긴다.
    pub effects: f32,
    /// 공격 예고 진하기(설정, 0198): 잉크 투명도에 곱한다. 1 = 0196·0197의 값(설정 70%), 기본 설정 50%는 약 0.71.
    pub tells: f32,
    /// `fx` 앞에서부터 이미 남길지 정한 개수(새 효과는 늘 뒤에 붙는다). 줄이는 몫의 누적값.
    fx_checked: usize,
    fx_acc: f32,
    /// 화면 위 한 줄 알림(되살아남, 씨앗)과 남은 시간. 글은 Tr이라 월드는 언어를 모른다.
    pub toast: Option<(Tr, f32)>,
    pub feel: Feel,
    /// 모으는 동안 빨려 드는 먼지의 간격 시계(0168).
    charge_fx_t: f32,
    /// 할 수 없는 행동을 눌렀을 때 HUD가 붉게 떠는 남은 시간(0 = 필살기 게이지, 1 = 퀵 슬롯, 0149).
    pub deny: [f32; 2],
    /// 컷씬 장치(0159). `hush`면 음악을 내린다(보스 곡도 아직). `gloom`은 환경 어둠 위에 덮는 어둠(0~1)과 빛의 자리·반지름.
    /// `boss_card`는 화면 가운데 큰 이름 카드(이름, 설명, 남은 시간). `petals`면 시든 꽃잎이 방을 떠돈다(모모 복선).
    pub hush: bool,
    pub gloom: Gloom,
    /// 컷씬 이펙트가 낸 소리(0159): 앱이 프레임마다 꺼내 튼다(컷씬은 Sfx를 모른다 — 테스트에서 돌릴 수 있게).
    /// 이번 틱에 낼 소리(0169 R1): 앱이 `Sfx::flush`로 비운다. 로직은 오디오 장치를 모른다.
    pub audio: sfx::Queue,
    pub boss_card: Option<(&'static str, Tr, f32)>,
    pub petals: bool,
    /// 발걸음 먼지(0081): 걸은 거리를 모아 한 걸음마다.
    step_dist: f32,
    /// 걸음 수: 두 걸음에 한 번 발소리(0234).
    steps: u32,
}

pub struct World {
    /// 지금 조작하는 캐릭터(게이지 종류 등).
    pub character: crate::equipment::Character,
    pub room: Room,
    pub player: Player,
    pub inv: Inventory,
    pub story: Story,
    pub stats: RunStats,
    pub hazards: Hazards,
    pub juice: Juice,
    pub enemies: Vec<Enemy>,
    pub projectiles: Vec<Projectile>,
    /// 도토가 던진 도토리·묻은 도토리(0195). 적의 탄과 따로.
    pub toss: Toss,
    pub pickups: Vec<Pickup>,
    /// 바닥의 반딧불·산딸기.
    pub loot: Vec<Loot>,
    /// 따라다니는 이웃(튜토리얼의 포포).
    pub npc: Option<Npc>,
    /// 제자리의 이웃(쉼터의 포포·보리, 공터에서 돌아온 보리).
    pub neighbors: Vec<Npc>,
    /// 연습: 맞아도 체력이 줄지 않는다(튜토리얼).
    pub no_damage: bool,
    /// 도움 모드 "포포의 응원"(0041)이 켜져 있다. 앱이 넣는다. 쓰러짐이 너그럽다: 실뿌리 없음, 반딧불 30%만,
    /// 길의 주인에게 쓰러지면 그 런은 지나간다(0210).
    pub assist: bool,
    /// 도감: 만난(깨어 있는 것을 본) 적 종류.
    pub seen: Vec<Kind>,
    /// 모험 구조: 장소·카메라·던전·저장 지점(adventure.rs).
    pub adv: Adventure,
    pub debug: Debug,
    /// 적을 다 잡으면 다음 웨이브를 자동으로 띄운다. 시나리오는 끌 수 있다.
    pub auto_waves: bool,
    /// 이번 런의 실뜨기(엔딩 뒤 어려움, docs/decisions/0043). 새로 나온 적에 한 번 건다.
    pub knots: crate::knots::Knots,
    /// 실의 꿈(0055): 쓰러진 적이 아무것도 떨어뜨리지 않는다(기록이 공정하게).
    pub dream: bool,
    /// 길의 샘(0159): 쉼방 가운데의 맑은 샘. 발을 담그면 한 번 가득 회복.
    pub spring: Option<Spring>,
    /// 가장 깊은 곳의 실매듭 끊기(0162). 있으면 할매·매듭·당기기가 돈다.
    pub deep: Option<Deep>,
    /// 끝방의 출구 자리(0159)와 생긴 뒤 흐른 초: 화면 밖이면 HUD 가장자리 표시가 가리킨다(`EXIT_MARK_DELAY` 뒤).
    /// 장소를 옮기면 없어진다.
    pub exit_at: Option<Vec2>,
    pub exit_age: f32,
    /// 지금 환경. `biome_by_wave`면 웨이브가 깊이를 대신해 바뀐다.
    pub biome: Biome,
    pub biome_by_wave: bool,
    pub wave: u32,
    pub time: f32,
    /// 이번 틱의 공격 히트박스(디버그 표시용).
    pub last_strike: Option<Strike>,
    respawn_timer: f32,
    /// 달리며 모으기의 땅울림이 켜져 있나(0215): 켜지고 꺼지는 틱에 소리를 낸다/멈춘다.
    run_rumble: bool,
    enemy_events: Vec<enemy::Event>,
}

impl Default for Hazards {
    fn default() -> Self {
        Self {
            mines: Vec::new(),
            clouds: Vec::new(),
            rocks: Vec::new(),
            rock_cd: t::ROCK_EVERY.1,
            cocoons: Vec::new(),
            waves: Vec::new(),
            thread_lash: None,
            cocoon_bounce_id: 0,
        }
    }
}

/// 위험물의 수명(0230 R13). 충돌 규칙은 종류마다 다르게 `World`의 틱에 있고, 언제 치우는지만 여기 한 곳.
impl Hazards {
    /// 싸움이 남긴 것을 치운다: 지뢰·포자 구름·떨어지는 돌·충격파·실 가닥. 고치는 방의 일부라 남긴다.
    /// 다시 깨어날 때, 굴·쉼터·가장 깊은 곳·들판에 들어설 때, 실의 꿈에서 보스 사이에.
    pub fn clear_fight(&mut self) {
        self.mines.clear();
        self.clouds.clear();
        self.rocks.clear();
        self.waves.clear();
        self.thread_lash = None;
    }

    /// 장소가 바뀐다(어두운 한가운데, `swap_place`): 고치까지 전부.
    pub fn clear_all(&mut self) {
        self.clear_fight();
        self.cocoons.clear();
    }

    /// 아기들의 고치 둘(0165): `c`는 둘 사이 가운데. 토끼풀은 그 앞을 지킨다.
    pub fn place_cocoons(&mut self, c: Vec2) {
        self.cocoons = vec![c + vec2(-13.0, 0.0), c + vec2(13.0, 2.0)];
    }

    #[cfg(test)]
    pub fn is_clear(&self) -> bool {
        self.mines.is_empty()
            && self.clouds.is_empty()
            && self.rocks.is_empty()
            && self.cocoons.is_empty()
            && self.waves.is_empty()
            && self.thread_lash.is_none()
    }
}

impl World {
    /// 싸움이 남긴 것을 치운다(0230 R13 → 0248): 위험물 + 플레이어가 던지고 묻고 깐 것(`toss` — 도토리 탄·묻은 도토리·가시 덤불).
    /// 방에 들어갈 때·쓰러져 일어날 때·꿈의 다음 보스 앞에서. 둘을 따로 부르면 한쪽을 빠뜨린다(쓰러진 자리에 덤불이 남았다).
    pub fn clear_fight(&mut self) {
        self.hazards.clear_fight();
        self.toss.clear();
    }

    /// 장소가 바뀐다(어두운 한가운데): 고치까지 전부.
    pub fn clear_place(&mut self) {
        self.hazards.clear_all();
        self.toss.clear();
    }
}

impl Default for Juice {
    fn default() -> Self {
        Self {
            feel: Feel::default(),
            fx: Vec::new(),
            effects: 1.0,
            tells: 1.0,
            fx_checked: 0,
            fx_acc: 0.0,
            audio: sfx::Queue::default(),
            toast: None,
            deny: [0.0; 2],
            boss_card: None,
            hush: false,
            gloom: Gloom::NONE,
            petals: false,
            charge_fx_t: 0.0,
            step_dist: 0.0,
            steps: 0,
        }
    }
}

impl World {
    pub fn new() -> Self {
        let room = Room::new();
        let player = Player::new(room.center());
        let mut w = Self {
            character: crate::equipment::Character::Mole,
            room,
            player,
            inv: Inventory::default(),
            story: Story::default(),
            stats: RunStats::default(),
            hazards: Hazards::default(),
            juice: Juice::default(),
            enemies: Vec::new(),
            projectiles: Vec::new(),
            toss: Toss::default(),
            pickups: Vec::new(),
            loot: Vec::new(),
            seen: Vec::new(),
            npc: None,
            neighbors: Vec::new(),
            no_damage: false,
            assist: false,
            adv: Adventure::default(),
            debug: Debug {
                overlay: false,
                hitboxes: false,
            },
            auto_waves: true,
            knots: crate::knots::Knots::default(),
            dream: false,
            spring: None,
            deep: None,
            exit_at: None,
            exit_age: 0.0,
            biome: Biome::Meadow,
            biome_by_wave: true,
            wave: 0,
            time: 0.0,
            last_strike: None,
            respawn_timer: t::WAVE_RESPAWN_DELAY,
            run_rumble: false,
            enemy_events: Vec::new(),
        };
        w.next_wave();
        w
    }

    /// 적도 웨이브도 없는 방(시나리오·컷씬용).
    pub fn empty() -> Self {
        let mut w = Self::new();
        w.enemies.clear();
        w.wave = 0;
        w.auto_waves = false;
        w.biome = Biome::Meadow;
        w.biome_by_wave = false;
        w
    }

    pub fn spawn_enemy(&mut self, kind: Kind, pos: Vec2) {
        self.enemies.push(Enemy::new(kind, pos));
    }

    pub fn tick(&mut self, a: &Actions) {
        self.step(a);
        self.thin_fx();
    }

    /// 입자 효과 설정(0141): 지난번 뒤로 새로 생긴 장식 효과를 `effects` 비율만 남긴다. 난수를 쓰지 않고 몫을 쌓아
    /// 고르게 솎는다(0.5면 둘에 하나). 어디서 효과를 만들든(월드·모험·컷씬) 여기 한 곳에서 거른다.
    fn thin_fx(&mut self) {
        let start = self.juice.fx_checked.min(self.juice.fx.len());
        if self.juice.effects < 1.0 {
            let mut i = start;
            while i < self.juice.fx.len() {
                if matches!(self.juice.fx[i].kind, FxKind::Ring { .. }) {
                    i += 1;
                    continue;
                }
                self.juice.fx_acc += self.juice.effects.max(0.0);
                if self.juice.fx_acc >= 1.0 {
                    self.juice.fx_acc -= 1.0;
                    i += 1;
                } else {
                    self.juice.fx.remove(i);
                }
            }
        }
        self.juice.fx_checked = self.juice.fx.len();
    }

    fn step(&mut self, a: &Actions) {
        let dt = t::TICK_DT;
        self.time += dt;
        if self.adventure_pre() {
            // 장소를 바꾸는 중(어두워졌다 밝아짐): 로직은 멈춘다.
            self.freeze_interp();
            return;
        }
        let knots = self.knots;
        for e in &mut self.enemies {
            e.apply_knots(knots);
        }
        // 문(0094): 열리는 순간 문마다 보라 실이 풀려 흩어지고 흙먼지가 인다.
        if self.room.tick_doors(dt) {
            for p in self.room.door_cells() {
                if (p - self.player.pos).length() < 360.0 {
                    unravel(&mut self.juice.fx, p, 10.0);
                    puff(&mut self.juice.fx, p, 2, self.biome.dust(), 7.0, 0.5);
                }
            }
        }
        self.juice.feel.tick(dt);
        if self.juice.feel.hitstop > 0.0 {
            // 히트스톱: 로직 시계만 멈춘다. 입력은 버퍼에 쌓인다.
            self.juice.feel.hitstop -= dt;
            self.player.buffer(a);
            self.freeze_interp();
            return;
        }

        // 가장 깊은 곳(0162): 감겨 있는 동안은 입력이 닿지 않고 가운데로 끌려간다.
        let still = Actions::default();
        let a = if self.deep.as_ref().is_some_and(|d| d.wrapped > 0.0) {
            &still
        } else {
            a
        };
        // 1. 플레이어 → 아이템·줍기
        let strike = self.tick_player(a, dt);
        self.tick_items(a, dt);
        self.tick_pickups(dt);
        // 2. 적·투사체·장애물. 쓰러진 적은 모았다가 공격 판정 뒤에 떨어뜨린다. 필살기 시동 중에는 멈춘다.
        let frozen = self.player.drill_revving();
        let mut deaths: Vec<(Vec2, Kind)> = Vec::new();
        self.tick_enemies(dt);
        self.tick_deaths(&mut deaths);
        self.charge_dust(dt);
        let (summon, bury) = self.enemy_events();
        self.enemy_dirt(dt, frozen);
        let landed = self.rockfall(dt, frozen, &mut deaths);
        self.thorn_pricks(&mut deaths);
        self.summon_mice(summon);
        self.bury_mines(bury);
        let popped = self.tick_mines_and_projectiles(dt, frozen, &mut deaths);
        self.tick_toss(dt, frozen, &mut deaths);
        // 3. 플레이어 공격 → 적·투사체
        self.apply_strike(strike, &mut deaths);
        self.settle_deaths(deaths);
        // 4. 적·투사체·위험물 → 플레이어
        self.hit_player(&landed, &popped);
        // 5. 적끼리 겹침 방지
        self.separate_enemies(dt);
        // 6. 정리
        self.cleanup(dt);
        // 7. 모험(입구·출구·저장, 던전 방, 카메라)
        self.adventure_post();
        // 8. 웨이브(아레나)
        self.tick_waves(dt);
    }

    /// 받아칠 위협이 가까운가(0189): 돌진 중인 적, 다 차 가는 예고, 날아오는 탄이 `PARRY_THREAT_R` 안에.
    fn parry_threat(&self) -> bool {
        let p = self.player.pos;
        let near = |q: Vec2| q.distance_squared(p) < t::PARRY_THREAT_R * t::PARRY_THREAT_R;
        self.enemies.iter().any(|e| {
            e.alive
                && near(e.pos)
                && (e.parryable() || e.telegraph().is_some_and(|k| k >= t::PARRY_THREAT_TELE))
        }) || self
            .projectiles
            .iter()
            .any(|b| b.alive && !b.friendly && near(b.pos) && b.vel.dot(p - b.pos) > 0.0)
    }

    /// 1. 플레이어: 입력·공격·먹기, 플레이어 이벤트의 소리·효과. 이번 틱의 공격 판정을 돌려준다.
    fn tick_player(&mut self, a: &Actions, dt: f32) -> Option<Strike> {
        // 1. 플레이어. 드릴 겨누기용으로 맞을 수 있는 적 자리를 넘긴다(0139).
        self.player.targets.clear();
        self.player.targets.extend(
            self.enemies
                .iter()
                .filter(|e| e.alive && e.hittable())
                .map(|e| e.pos),
        );
        self.web_tick();
        self.player.threat = self.parry_threat();
        let strike = self.player.update(a, &self.room, dt);
        self.last_strike = strike;
        self.deep_tick(dt);
        // 도토의 던지는 손(`Toss`)은 받아치기만 — 매듭·그물·고치·부술 것에는 탄이 닿는다(0195).
        if let Some(s) = strike.filter(|s| s.kind != crate::combat::StrikeKind::Toss) {
            self.touch_world(&s);
        }
        self.take_spawns();
        self.player_rumble(dt);
        self.player_events();
        self.drill_fx();
        strike
    }

    /// 1-1. 땅이 운다: 땅속에서 힘을 모으는 동안(0158)과 달리며 멀리뛰기를 모은 동안(0215). 흔들림·흙알·소리.
    fn player_rumble(&mut self, dt: f32) {
        // 땅속에서 힘을 모으는 동안(0158) 땅이 운다: 힘에 비례해 흔들리고 흙알이 튄다.
        if self.player.is_burrowed() {
            let charge = self.player.burrow_charge();
            if charge > 0.0 {
                self.juice.feel.add_trauma(t::BURROW_RUMBLE_SHAKE * charge);
                if ((self.time / dt).round() as u32).is_multiple_of(t::BURROW_RUMBLE_EVERY) {
                    clods(
                        &mut self.juice.fx,
                        self.player.pos,
                        1 + (2.0 * charge) as usize,
                        DIRT,
                        30.0 + 50.0 * charge,
                    );
                }
            }
        }
        // 달리며 모으기(0215, 사장 "땅파기 모으기처럼 구구구 하는 효과"): 모모·도토가 멀리뛰기를 뛸 수 있게 되면(`RUN_MIN` 뒤)
        // 땅이 울기 시작한다 — 땅파기와 같은 소리·흙 고리, 달리는 동안 흔들림과 발밑 흙알. 뛰거나 멈추면 소리를 끊는다.
        let ready = self.player.jump_aim().is_some_and(|(_, r, _)| r);
        if ready != self.run_rumble {
            self.run_rumble = ready;
            if ready {
                self.juice.audio.play(sfx::Id::Rumble);
                self.juice
                    .fx
                    .push(ring(self.player.pos, 3.0, 10.0, 0.25, DIRT));
            } else {
                self.juice.audio.stop(sfx::Id::Rumble);
            }
        }
        if ready {
            self.juice
                .feel
                .add_trauma(t::BURROW_RUMBLE_SHAKE * t::RUN_RUMBLE);
            if ((self.time / dt).round() as u32).is_multiple_of(t::BURROW_RUMBLE_EVERY) {
                clods(
                    &mut self.juice.fx,
                    self.player.pos + vec2(0.0, 4.0),
                    2,
                    DIRT,
                    55.0,
                );
            }
        }
    }

    /// 1-2. 플레이어 이벤트(휘두름·박자·땅파기·먹기 …) → 소리·효과·월드 일. 다 먹었으면 효과를 건다.
    fn player_events(&mut self) {
        let mut ate = None;
        let mut rose = false;
        for ev in &self.player.events {
            match ev {
                player::Event::Swing(i) => self.juice.audio.play(sfx::Id::Claw(*i)),
                player::Event::BeatTick => {
                    // 박자가 왔다(0180): 메트로놈 '똑' + 과녁 고리에서 작은 흰 고리가 퍼진다.
                    self.juice.audio.play(sfx::Id::BeatTick);
                    self.juice.fx.push(ring(
                        self.player.pos,
                        t::BEAT_RING_END,
                        t::BEAT_RING_END + 5.0,
                        0.12,
                        Color::new(1.0, 1.0, 1.0, 0.8),
                    ));
                }
                player::Event::Beat(n) => {
                    // 박자에 맞췄다(0167 → 0174): 몸에서 퍼지는 고리 + 연속 수만큼 오르는 음 + 발톱 끝 반짝임.
                    self.juice.audio.play(sfx::Id::Beat(*n));
                    let r1 = 16.0 + 2.0 * (*n).min(4) as f32;
                    self.juice
                        .fx
                        .push(ring(self.player.pos, 6.0, r1, 0.22, WHITE));
                    let p = self.player.pos + self.player.facing * 7.0;
                    self.juice.fx.push(glint(p, WHITE, 0.18));
                }
                player::Event::Heavy => {
                    // 강공격(0161): 마무리 타 소리 + 앞으로 튀는 불똥.
                    self.juice.audio.play(sfx::Id::Claw(2));
                    let p = self.player.pos + self.player.facing * 6.0;
                    spray(&mut self.juice.fx, p, self.player.facing, 6, WHITE, 120.0);
                }
                player::Event::HeavyReady => {
                    // 다 모였다: 발톱이 번쩍 + 짧은 흰 번쩍(0168, 흔들림 설정을 따른다).
                    self.juice.audio.play(sfx::Id::Charge);
                    self.juice.feel.flash = self.juice.feel.flash.max(t::HEAVY_READY_FLASH);
                    let p = self.player.pos + self.player.facing * 7.0;
                    self.juice.fx.push(glint(p, WHITE, 0.35));
                    self.juice.fx.push(ring(p, 2.0, 10.0, 0.2, WHITE));
                }
                player::Event::ChargeStart => self.juice.audio.play(sfx::Id::ChargeUp),
                player::Event::HeavyPerfect => {
                    // 딱 맞춘 강타(0168): 박자 음(가장 높은 것) + 큰 반짝임.
                    self.juice.audio.play(sfx::Id::Beat(4));
                    let p = self.player.pos + self.player.facing * 8.0;
                    self.juice.fx.push(glint(p, WHITE, 0.3));
                    self.juice.fx.push(ring(p, 4.0, 16.0, 0.18, WHITE));
                }
                player::Event::Burrow => {
                    // 파고들기(0160): 흙이 확 튀고 화면이 살짝 — 톡 한 번이 "동작"으로 읽히게.
                    self.stats.burrows += 1;
                    self.juice.audio.play(sfx::Id::Burrow);
                    self.juice.feel.add_trauma(0.12);
                    self.juice
                        .fx
                        .push(ring(self.player.pos, 4.0, 18.0, 0.22, DIRT));
                    clods(&mut self.juice.fx, self.player.pos, 10, DIRT, 85.0);
                    puff(
                        &mut self.juice.fx,
                        self.player.pos,
                        3,
                        self.biome.dust(),
                        6.0,
                        0.4,
                    );
                }
                player::Event::Rumble => {
                    // 힘 모으기 시작(0158): 땅이 울기 시작하는 소리 + 작은 흙 고리.
                    self.juice.audio.play(sfx::Id::Rumble);
                    self.juice
                        .fx
                        .push(ring(self.player.pos, 3.0, 10.0, 0.25, DIRT));
                }
                player::Event::Emerge => {
                    // 땅이 터진다(0090): 맞는 반지름의 밝은 고리 + 밀쳐내는 반지름까지 퍼지는 흙 고리.
                    // 모은 힘(0158)만큼 고리·흙·흔들림이 크고, 다 모았으면 소리도 무겁다.
                    let charge = self.player.burrow_charge();
                    let k = 1.0 + (t::EMERGE_CHARGE_RADIUS - 1.0) * charge;
                    self.juice.audio.stop(sfx::Id::Rumble);
                    self.juice.audio.play(if charge >= 0.5 {
                        sfx::Id::EmergeBig
                    } else {
                        sfx::Id::Emerge
                    });
                    self.juice.fx.push(ring(
                        self.player.pos,
                        6.0,
                        self.player.emerge_radius_for(charge) + 4.0,
                        0.22 + 0.08 * charge,
                        WHITE,
                    ));
                    self.juice.fx.push(ring(
                        self.player.pos,
                        10.0,
                        t::EMERGE_PUSH_RADIUS * k,
                        0.32 + 0.1 * charge,
                        DIRT,
                    ));
                    self.juice
                        .feel
                        .add_trauma(t::EMERGE_BURST_SHAKE * (1.0 + charge));
                    let n = 14 + (12.0 * charge) as usize;
                    let speed = 110.0 * k;
                    burst(&mut self.juice.fx, self.player.pos, n, DIRT, speed);
                    clods(&mut self.juice.fx, self.player.pos, n, DIRT, speed);
                    puff(
                        &mut self.juice.fx,
                        self.player.pos,
                        4 + (4.0 * charge) as usize,
                        self.biome.dust(),
                        8.0 + 4.0 * charge,
                        0.45 + 0.15 * charge,
                    );
                }
                player::Event::DrillRev => {
                    // 부아아…: 소리 하나가 시동부터 돌진 끝까지 간다.
                    self.juice
                        .audio
                        .play(sfx::Id::Drill(self.player.talents.pick[2] as usize));
                    self.juice
                        .fx
                        .push(ring(self.player.pos, 22.0, 3.0, t::DRILL_REV, RAGE));
                }
                player::Event::Drill => {
                    // …아앙: 튀어 나간다.
                    self.stats.drills += 1;
                    let p = self.player.pos;
                    self.juice.fx.push(ring(p, 4.0, 22.0, 0.25, WHITE));
                    burst(&mut self.juice.fx, p, 14, DIRT, 120.0);
                    self.juice.feel.add_trauma(0.35);
                }
                player::Event::DrillWall => {
                    self.stats.slams += 1;
                    self.juice
                        .audio
                        .stop(sfx::Id::Drill(self.player.talents.pick[2] as usize));
                    self.juice.audio.play(sfx::Id::DrillWall);
                    let p = self.player.pos + self.player.facing * 6.0;
                    self.juice
                        .fx
                        .push(ring(p, 6.0, t::DRILL_WALL_RADIUS + 10.0, 0.35, WHITE));
                    self.juice
                        .fx
                        .push(ring(p, 4.0, t::DRILL_WALL_RADIUS, 0.3, RAGE));
                    self.juice
                        .fx
                        .push(ring(p, 2.0, t::DRILL_WALL_RADIUS * 0.6, 0.25, DIRT));
                    burst(&mut self.juice.fx, p, 30, DIRT, 170.0);
                    burst(&mut self.juice.fx, p, 10, RAGE, 120.0);
                    clods(&mut self.juice.fx, p, 10, DIRT, 110.0);
                    puff(&mut self.juice.fx, p, 5, self.biome.dust(), 10.0, 0.5);
                    self.juice.feel.flash = t::DRILL_WALL_FLASH;
                }
                player::Event::Ate(c) => ate = Some(*c),
                // 모모(0056): 뛰어오를 때 흙먼지, 내려앉을 때 작은 고리(이어 뛰기는 크게 + 흔들림).
                player::Event::Hop => {
                    self.stats.burrows += 1;
                    self.juice.audio.play(sfx::Id::Dodge);
                    burst(&mut self.juice.fx, self.player.pos, 6, DIRT, 50.0);
                    puff(
                        &mut self.juice.fx,
                        self.player.pos + vec2(0.0, 4.0),
                        2,
                        self.biome.dust(),
                        4.0,
                        0.35,
                    );
                }
                player::Event::Land(big) => {
                    let p = self.player.pos;
                    if *big {
                        let r = self.player.chain_radius();
                        self.juice.audio.play(sfx::Id::WallHit);
                        self.juice.feel.add_trauma(0.25);
                        self.juice.fx.push(ring(p, 4.0, r + 4.0, 0.25, PETAL));
                        self.juice.fx.push(ring(p, 2.0, r, 0.2, DIRT));
                        burst(&mut self.juice.fx, p, 12, DIRT, 110.0);
                        burst(&mut self.juice.fx, p, 6, PETAL, 80.0);
                    } else {
                        self.juice.fx.push(ring(p, 2.0, 9.0, 0.15, DIRT));
                    }
                    puff(
                        &mut self.juice.fx,
                        p + vec2(0.0, 4.0),
                        if *big { 5 } else { 2 },
                        self.biome.dust(),
                        5.0,
                        0.4,
                    );
                }
                // 회피의 밀치기(0214): 흙먼지 고리(밀치는 둘레만큼) — 맞힘 소리는 없다.
                player::Event::Shove => {
                    let p = self.player.pos;
                    self.juice
                        .fx
                        .push(ring(p, 3.0, t::SHOVE_RADIUS, 0.16, DIRT));
                    puff(
                        &mut self.juice.fx,
                        p + vec2(0.0, 4.0),
                        3,
                        self.biome.dust(),
                        6.0,
                        0.35,
                    );
                }
                player::Event::Stamp => {
                    let p = self.player.pos;
                    self.juice.audio.play(sfx::Id::Emerge);
                    self.juice.feel.add_trauma(0.1);
                    self.juice
                        .fx
                        .push(ring(p, 2.0, t::TAL_STAMP_RADIUS, 0.18, PETAL));
                    burst(&mut self.juice.fx, p, 5, DIRT, 70.0);
                }
                // 멀리뛰기 착지(0209): 모모는 밟기 고리(반지름 = 타격) + 흔들림, 도토는 흙먼지만.
                player::Event::JumpLand { stomp } => {
                    let p = self.player.pos;
                    if *stomp {
                        let r = self.player.jump_stomp_radius();
                        self.juice.audio.play(sfx::Id::WallHit);
                        self.juice.feel.add_trauma(0.18);
                        self.juice.fx.push(ring(p, 3.0, r + 3.0, 0.22, PETAL));
                        self.juice.fx.push(ring(p, 2.0, r, 0.18, DIRT));
                        burst(&mut self.juice.fx, p, 9, DIRT, 95.0);
                    } else {
                        self.juice.fx.push(ring(p, 2.0, 11.0, 0.16, DIRT));
                    }
                    puff(
                        &mut self.juice.fx,
                        p + vec2(0.0, 4.0),
                        4,
                        self.biome.dust(),
                        5.0,
                        0.4,
                    );
                }
                player::Event::Bury => {
                    // 도토리 묻기(0195): 흙이 튀고 묻는 소리.
                    self.juice.audio.play(sfx::Id::Burrow);
                    let p = self.player.pos + self.player.facing * 10.0;
                    clods(&mut self.juice.fx, p, 8, DIRT, 70.0);
                }
                player::Event::Rose => rose = true,
                player::Event::Hurt | player::Event::Die | player::Event::Parry => {}
            }
        }
        if let Some(c) = ate {
            self.finish_eating(c);
        }
        if rose {
            self.worm_rise();
        }
    }

    /// 1-3. 드릴: 시동 중엔 불꽃이 모이고, 돌진 중엔 흙이 튀고 화면이 계속 떨린다.
    fn drill_fx(&mut self) {
        // 드릴: 시동 중엔 불꽃이 모이고, 돌진 중엔 흙이 튀고 화면이 계속 떨린다.
        if let player::State::Drill { dir, time, .. } = self.player.state {
            let p = self.player.pos;
            if time < t::DRILL_REV {
                let ang = rand::gen_range(0.0, std::f32::consts::TAU);
                let from = p + Vec2::from_angle(ang) * 20.0;
                self.juice.fx.push(Fx {
                    pos: from,
                    vel: (p - from) / 0.2,
                    time: 0.0,
                    dur: 0.2,
                    kind: FxKind::Spark { color: RAGE },
                });
            } else {
                self.juice.feel.add_trauma(t::DRILL_RIDE_SHAKE);
                let back = p - dir * 6.0;
                let side = vec2(-dir.y, dir.x) * rand::gen_range(-4.0, 4.0);
                self.juice.fx.push(Fx {
                    pos: back + side,
                    vel: -dir * rand::gen_range(40.0, 110.0) + side * 8.0,
                    time: 0.0,
                    dur: 0.3,
                    kind: FxKind::Spark { color: DIRT },
                });
            }
        }
    }

    /// 1a. 퀵 아이템·거절 표시·먹기 시작·토스트 시계.
    fn tick_items(&mut self, a: &Actions, dt: f32) {
        // 1a. 퀵 아이템
        self.inv.bag.flash = (self.inv.bag.flash - t::TICK_DT).max(0.0);
        if a.item_prev || a.item_next {
            self.inv.bag.cycle(a.item_next);
        }
        for d in &mut self.juice.deny {
            *d = (*d - dt).max(0.0);
        }
        // 할 수 없는 행동(0149): 눌렀는데 아무 일도 안 일어나면 고장 난 줄 안다(0093과 같은 까닭).
        // "안 돼" 소리 + 그 HUD 칸이 붉게 떤다. 필살기는 발치에 김이 픽 빠진다.
        if self.player.is_alive() && self.player.special_denied(a) {
            self.deny_feedback(0);
            puff(
                &mut self.juice.fx,
                self.player.pos + vec2(0.0, 4.0),
                2,
                GRAY,
                4.0,
                0.3,
            );
        }
        if a.use_item
            && self.player.is_alive()
            && (self.inv.bag.count(self.inv.bag.selected) == 0 || self.player.item_cooling())
        {
            self.deny_feedback(1);
        }
        if self.player.wants_to_eat() {
            self.start_eating();
        }
        if let Some((_, time)) = self.juice.toast.as_mut() {
            *time -= dt;
            if *time <= 0.0 {
                self.juice.toast = None;
            }
        }
        if let Some((_, _, time)) = self.adv.biome_card.as_mut() {
            *time -= dt;
            if *time <= 0.0 {
                self.adv.biome_card = None;
            }
        }
    }

    /// 1b·1c. 획득물(무기)과 떨어진 것(반딧불·씨앗·가방 아이템·뿌리 심장) 줍기.
    fn tick_pickups(&mut self, dt: f32) {
        // 1b. 획득물: 떨어진 것처럼 자석 거리 안이면 끌려온다(0207).
        let ppos = self.player.pos;
        let mut got = Vec::new();
        let pull = t::LOOT_MAGNET_MAX_SPEED * dt;
        self.pickups.retain_mut(|p| {
            p.age += dt;
            // 막 떨어진 것은 잠깐 줍지 않는다(떨어지는 걸 보게, 떨어진 것과 같은 규칙).
            if p.age < t::LOOT_PICKUP_DELAY {
                return true;
            }
            let to = ppos - p.pos;
            let d = to.length();
            if d < t::LOOT_MAGNET_RADIUS && d > 0.0 {
                p.pos += to / d * pull.min(d);
            }
            if (p.pos - ppos).length() <= t::PICKUP_RADIUS {
                got.push((p.pos, p.item));
                false
            } else {
                true
            }
        });
        for (pos, item) in got {
            self.inv.gifts.push(item);
            self.juice.audio.play(sfx::Id::Pickup);
            self.juice.fx.push(ring(pos, 4.0, 18.0, 0.3, WHITE));
            burst(&mut self.juice.fx, pos, 14, PROJECTILE, 80.0);
        }

        // 1c. 반딧불·가방 아이템·씨앗. 땅속에서도 줍는다(굴파기로 지나가며 쓸어 담기).
        let alive = self.player.is_alive();
        let mut bag = std::mem::take(&mut self.inv.bag);
        let (mut bugs, mut items, mut hearts, mut seeds) = (0, 0, 0, 0);
        let mut picked_at = Vec::new();
        let room = &self.room;
        self.loot.retain_mut(|l| {
            let can = alive
                && match l.kind {
                    LootKind::Glowbug | LootKind::Seed | LootKind::RootHeart => true,
                    LootKind::Berry => bag.has_room(Consumable::Berry),
                    LootKind::Worm => bag.has_room(Consumable::Worm),
                };
            if !l.update(dt, ppos, can, room) {
                return true;
            }
            match l.kind {
                LootKind::Glowbug => bugs += 1,
                LootKind::Seed => seeds += 1,
                LootKind::Berry => {
                    bag.add(Consumable::Berry);
                    items += 1;
                }
                LootKind::Worm => {
                    bag.add(Consumable::Worm);
                    items += 1;
                }
                LootKind::RootHeart => hearts += 1,
            }
            picked_at.push((l.pos, l.kind));
            false
        });
        self.inv.bag = bag;
        self.inv.glowbugs += bugs;
        self.inv.seeds += seeds;
        if bugs > 0 || seeds > 0 {
            self.juice.audio.play(sfx::Id::Glowbug);
        }
        if items > 0 {
            self.juice.audio.play(sfx::Id::Pickup);
        }
        if hearts > 0 {
            self.inv.root_hearts += hearts;
            self.refresh_stats();
            self.player.heal(self.player.stats.max_hp);
            self.juice.audio.play(sfx::Id::Heal);
            self.juice.toast = Some((|s| s.toast_heart, 2.5));
        }
        for (pos, kind) in picked_at {
            match kind {
                LootKind::Glowbug => {
                    self.juice.fx.push(ring(pos, 1.0, 5.0, 0.15, GLOW));
                    self.juice.fx.push(glint(pos + vec2(0.0, -3.0), GLOW, 0.3));
                }
                LootKind::Seed => {
                    self.juice.fx.push(ring(pos, 1.0, 6.0, 0.18, SEED));
                    self.juice
                        .fx
                        .push(glint(pos + vec2(0.0, -3.0), WHITE, 0.25));
                }
                LootKind::Berry | LootKind::Worm => {
                    self.juice.fx.push(ring(pos, 2.0, 10.0, 0.2, WHITE));
                    self.juice.fx.push(glint(pos + vec2(0.0, -4.0), WHITE, 0.3));
                }
                LootKind::RootHeart => {
                    self.juice.fx.push(ring(pos, 4.0, 22.0, 0.4, POISON));
                    burst(&mut self.juice.fx, pos, 16, POISON, 90.0);
                }
            }
        }
    }

    /// 2. 적·이웃 한 틱(필살기 시동 중엔 멈춘다). 깨어 있는 적은 도감에 적는다.
    fn tick_enemies(&mut self, dt: f32) {
        // 2. 적·투사체
        let targetable = self.player.is_alive() && !self.player.is_burrowed();
        self.enemy_events.clear();
        // 필살기 시동 중에는 적·투사체가 멈춘다(필살기 정지 연출).
        let frozen = self.player.drill_revving();
        for e in self.enemies.iter_mut().filter(|_| !frozen) {
            // 흰 실 그물(0165): 걷는 적은 느려진다. 나는 것·보스는 그대로.
            e.webbed = (e.webbed - dt).max(0.0);
            if !self.room.webs.is_empty() && !e.flying() && !e.is_boss() {
                let rr = e.radius() + t::WEB_R - 3.0;
                if self
                    .room
                    .webs
                    .iter()
                    .any(|c| (*c - e.pos).length_squared() <= rr * rr)
                {
                    e.webbed = 0.1;
                }
            }
            e.update(
                self.player.pos,
                targetable,
                &self.room,
                dt,
                &mut self.projectiles,
                &mut self.enemy_events,
            );
            e.walked += (e.pos - e.prev_pos).length();
        }
        // 이웃: 따라갈 곳이 있으면 천천히 걸어간다(가까우면 멈춤).
        if let Some(n) = self.npc.as_mut() {
            n.prev_pos = n.pos;
            if let Some(to) = n.target {
                let d = to - n.pos;
                if d.length() > 6.0 {
                    let step = d.normalize() * t::NPC_SPEED * dt;
                    n.pos += step;
                    if step.x.abs() > 0.1 {
                        n.face_left = step.x < 0.0;
                    }
                }
            }
            let r = 5.0;
            self.room.resolve_circle(&mut n.pos, r);
        }
        // 도감: 깨어 있는 적을 처음 보면 적어 둔다.
        for e in &self.enemies {
            if !e.asleep && e.kind != Kind::Dummy && !self.seen.contains(&e.kind) {
                self.seen.push(e.kind);
            }
        }
    }

    /// 적 틱 안에서 쓰러진 적(독·벽 쿵). 떨어뜨릴 것은 `deaths`에 모았다가 `settle_deaths`에서.
    fn tick_deaths(&mut self, deaths: &mut Vec<(Vec2, Kind)>) {
        // 쓰러진 적(독·공격). 떨어뜨릴 것은 루프가 끝난 뒤에.
        for e in self.enemies.iter_mut().filter(|e| e.died_of_poison) {
            e.died_of_poison = false;
            enemy_down(
                &mut self.juice.fx,
                &mut self.juice.audio,
                &mut self.juice.feel,
                &mut self.stats.kills,
                deaths,
                e,
                POISON,
            );
        }
        // 벽 쿵(0168): 강공격에 날아가다 벽에 박힌 적.
        for e in self.enemies.iter_mut().filter(|e| e.slammed) {
            e.slammed = false;
            self.juice.audio.play(sfx::Id::WallHit);
            self.juice.feel.add_hitstop(t::WALL_SLAM_HITSTOP);
            self.juice.feel.add_trauma(t::WALL_SLAM_SHAKE);
            let back = e.vel.normalize_or_zero();
            self.juice.fx.push(ring(e.pos, 3.0, 18.0, 0.2, WHITE));
            spray(&mut self.juice.fx, e.pos, back, 6, WHITE, 160.0);
            clods(&mut self.juice.fx, e.pos - back * e.radius(), 5, DIRT, 90.0);
            if !e.alive {
                enemy_down(
                    &mut self.juice.fx,
                    &mut self.juice.audio,
                    &mut self.juice.feel,
                    &mut self.stats.kills,
                    deaths,
                    e,
                    enemy_color(e.kind),
                );
            }
        }
    }

    /// 모으는 동안(0168) 발 쪽으로 빨려 드는 먼지.
    fn charge_dust(&mut self, dt: f32) {
        // 모으는 동안(0168): 발 쪽으로 빨려 드는 먼지.
        if let player::State::Charging { .. } = self.player.state {
            self.juice.charge_fx_t += dt;
            while self.juice.charge_fx_t >= t::CHARGE_PULL_EVERY {
                self.juice.charge_fx_t -= t::CHARGE_PULL_EVERY;
                let ang = rand::gen_range(0.0, std::f32::consts::TAU);
                let off = vec2(ang.cos(), ang.sin()) * t::CHARGE_PULL_R;
                let foot = self.player.pos + vec2(0.0, 4.0);
                self.juice.fx.push(Fx {
                    pos: foot + off,
                    vel: -off / 0.2,
                    time: 0.0,
                    dur: 0.2,
                    kind: FxKind::Spark {
                        color: self.biome.dust(),
                    },
                });
            }
        } else {
            self.juice.charge_fx_t = 0.0;
        }
    }

    /// 적 이벤트의 소리·효과. 대장의 소환과 도토의 묻기는 (소환, 묻을 수)로 돌려줘 장애물 처리 뒤에 한다.
    fn enemy_events(&mut self) -> (bool, usize) {
        let mut summon = false;
        let mut bury = 0;
        let mut thumps = Vec::new();
        let mut threads = Vec::new();
        let mut rouse = Vec::new();
        for ev in &self.enemy_events {
            match ev {
                enemy::Event::Charge => self.juice.audio.play(sfx::Id::Charge),
                enemy::Event::WallHit => {
                    self.juice.audio.play(sfx::Id::WallHit);
                    self.juice.feel.add_trauma(0.2);
                }
                enemy::Event::Shoot => self.juice.audio.play(sfx::Id::Shoot),
                enemy::Event::Leap => self.juice.audio.play(sfx::Id::Charge),
                enemy::Event::Land => {
                    self.juice.audio.play(sfx::Id::WallHit);
                    self.juice.feel.add_trauma(0.12);
                }
                enemy::Event::Summon => summon = true,
                enemy::Event::Bury(n) => bury = *n,
                enemy::Event::Dig => self.juice.audio.play(sfx::Id::Burrow),
                enemy::Event::Pop => {
                    self.juice.audio.play(sfx::Id::Emerge);
                    self.juice.feel.add_trauma(0.25);
                }
                enemy::Event::Tongue => self.juice.audio.play(sfx::Id::Shoot),
                enemy::Event::Spore => self.juice.audio.play(sfx::Id::Burrow),
                enemy::Event::Thump(at) => thumps.push(*at),
                enemy::Event::Thread(at, dir) => threads.push((*at, *dir)),
                enemy::Event::Rouse(at) => rouse.push(*at),
            }
        }
        for at in rouse {
            self.rouse_fx(at);
        }
        for at in thumps {
            self.thump_at(at);
        }
        for (at, dir) in threads {
            self.thread_at(at, dir);
        }
        (summon, bury)
    }

    /// 보스가 2단계로 깨어났다(0233): 멈칫 + 흔들림 + 쿵, 보라 실 고리 둘이 퍼지고 실 부스러기가 튄다.
    fn rouse_fx(&mut self, at: Vec2) {
        self.juice.audio.play(sfx::Id::Impact);
        self.juice.audio.play(sfx::Id::Rumble);
        self.juice.feel.add_hitstop(t::BOSS_P2_HITSTOP);
        self.juice.feel.add_trauma(t::BOSS_P2_TRAUMA);
        self.juice.fx.push(ring(at, 6.0, 44.0, 0.45, THREAD));
        self.juice.fx.push(ring(at, 2.0, 26.0, 0.3, ROUSE));
        burst(&mut self.juice.fx, at, 14, THREAD, 120.0);
    }

    /// 모모가 숨는·튀어나오는 자리의 흙, 포자 구름·충격파·실 가닥의 시계.
    fn enemy_dirt(&mut self, dt: f32, frozen: bool) {
        // 모모가 숨는·튀어나오는 자리에 흙.
        for e in &self.enemies {
            match e.state {
                enemy::State::Under { time } if time <= dt => {
                    burst(&mut self.juice.fx, e.pos, 10, DIRT, 70.0);
                }
                enemy::State::Pop { time } if time <= dt => {
                    let r = e.contact_radius();
                    self.juice.fx.push(ring(e.pos, 4.0, r, 0.25, DIRT));
                    burst(&mut self.juice.fx, e.pos, 14, DIRT, 110.0);
                }
                enemy::State::Puff { time: 0.0 } => {
                    self.hazards.clouds.push(Cloud {
                        pos: e.pos,
                        time: 0.0,
                    });
                }
                _ => {}
            }
        }
        for c in self.hazards.clouds.iter_mut().filter(|_| !frozen) {
            c.time += dt;
        }
        self.hazards.clouds.retain(|c| c.time < t::SPORE_CLOUD_TIME);
        for wv in self.hazards.waves.iter_mut().filter(|_| !frozen) {
            wv.time += dt;
        }
        self.hazards
            .waves
            .retain(|w| w.time < t::CLOVER_WAVE_TIME + 0.25);
        if let Some((_, _, age)) = self.hazards.thread_lash.as_mut() {
            *age += dt;
        }
        if self
            .hazards
            .thread_lash
            .is_some_and(|(_, _, age)| age > 0.3)
        {
            self.hazards.thread_lash = None;
        }
    }

    /// 무너지는 천장(0147): 돌을 떨어뜨리고, 이번 틱에 떨어진 자리를 돌려준다(적은 여기서 맞고, 플레이어는 `hit_player`에서).
    fn rockfall(&mut self, dt: f32, frozen: bool, deaths: &mut Vec<(Vec2, Kind)>) -> Vec<Vec2> {
        // 무너지는 천장(0147): 심층 싸움방에서 문이 잠긴 동안(보스·꿈은 빼고) 몇 초마다 돌 하나.
        // 플레이어 둘레가 많고, 가끔 적 둘레에 떨어져 적도 맞는다. 땅속은 괜찮다.
        let quake = self.adv.place == Place::Dungeon
            && self.biome == Biome::Deep
            && self.room.doors_locked()
            && !self.dream
            && !self.enemies.iter().any(|e| e.is_boss());
        if !quake {
            self.hazards.rock_cd = self.hazards.rock_cd.max(t::ROCK_EVERY.0);
        } else if !frozen {
            self.hazards.rock_cd -= dt;
            if self.hazards.rock_cd <= 0.0 {
                self.hazards.rock_cd = rand::gen_range(t::ROCK_EVERY.0, t::ROCK_EVERY.1);
                let foes: Vec<Vec2> = self
                    .enemies
                    .iter()
                    .filter(|e| e.alive && !e.asleep && !e.flying())
                    .map(|e| e.pos)
                    .collect();
                let base = if !foes.is_empty() && rand::gen_range(0.0, 1.0) < 0.4 {
                    foes[rand::gen_range(0, foes.len())]
                } else {
                    self.player.pos
                };
                let a = rand::gen_range(0.0, std::f32::consts::TAU);
                let mut p = base + Vec2::from_angle(a) * rand::gen_range(0.0, t::ROCK_SPREAD);
                self.room.resolve_circle(&mut p, 6.0);
                self.hazards.rocks.push(Rockfall { pos: p, time: 0.0 });
            }
        }
        let mut landed = Vec::new();
        for r in self.hazards.rocks.iter_mut().filter(|_| !frozen) {
            r.time += dt;
            if r.time >= t::ROCK_FALL {
                landed.push(r.pos);
            }
        }
        self.hazards.rocks.retain(|r| r.time < t::ROCK_FALL);
        for &p in &landed {
            self.juice.audio.play(sfx::Id::WallHit);
            self.juice.feel.add_trauma(0.18);
            self.juice
                .fx
                .push(ring(p, 3.0, t::ROCK_R + 3.0, 0.25, DIRT));
            clods(&mut self.juice.fx, p, 8, ROCK, 100.0);
            puff(&mut self.juice.fx, p, 3, self.biome.dust(), 7.0, 0.5);
            for e in self.enemies.iter_mut().filter(|e| e.alive && e.hittable()) {
                let rr = t::ROCK_R + e.radius();
                if (e.pos - p).length_squared() > rr * rr {
                    continue;
                }
                let away = (e.pos - p).normalize_or(vec2(1.0, 0.0));
                e.hurt(t::ROCK_ENEMY_DAMAGE, away, 140.0, 0.8);
                if !e.alive {
                    enemy_down(
                        &mut self.juice.fx,
                        &mut self.juice.audio,
                        &mut self.juice.feel,
                        &mut self.stats.kills,
                        deaths,
                        e,
                        enemy_color(e.kind),
                    );
                }
            }
        }
        landed
    }

    /// 가시뿌리 덤불(0147): 돌진하던 적이 박혀 멈춘다.
    fn thorn_pricks(&mut self, deaths: &mut Vec<(Vec2, Kind)>) {
        // 가시뿌리 덤불(0147): 돌진하다 덤불에 박으면 찔려 멈춘다. 나는 박쥐·말린 고슴도치·보스는 그대로 지나간다.
        if !self.room.thorns.is_empty() {
            for e in self.enemies.iter_mut() {
                let enemy::State::Charge { dir, .. } = e.state else {
                    continue;
                };
                if e.flying() || e.guarding() || e.is_boss() {
                    continue;
                }
                let rr = e.radius() + t::THORN_R - 2.0;
                if !self
                    .room
                    .thorns
                    .iter()
                    .any(|c| (*c - e.pos).length_squared() <= rr * rr)
                {
                    continue;
                }
                e.hurt(t::THORN_ENEMY_DAMAGE, -dir, 60.0, t::THORN_STUN);
                self.juice.audio.play(sfx::Id::EnemyHit);
                self.juice.fx.push(ring(e.pos, 2.0, 10.0, 0.15, THORN));
                if !e.alive {
                    enemy_down(
                        &mut self.juice.fx,
                        &mut self.juice.audio,
                        &mut self.juice.feel,
                        &mut self.stats.kills,
                        deaths,
                        e,
                        enemy_color(e.kind),
                    );
                }
            }
        }
    }

    /// 대장이 울부짖으면 양옆에 생쥐 떼.
    fn summon_mice(&mut self, summon: bool) {
        // 대장이 울부짖으면 양옆에 생쥐 떼(셋).
        if summon {
            self.juice.audio.play(sfx::Id::Shoot);
            self.juice.feel.add_trauma(0.2);
            let chief: Vec<Vec2> = self
                .enemies
                .iter()
                .filter(|e| e.is_boss())
                .map(|e| e.pos)
                .collect();
            for c in chief {
                let side = if self.player.pos.x > c.x { -1.0 } else { 1.0 };
                let mut p = c + vec2(side * 34.0, 0.0);
                self.room.resolve_circle(&mut p, 8.0);
                for off in crate::species::pack_offsets(Kind::HouseMouse) {
                    let mut e = Enemy::new(Kind::HouseMouse, p + off);
                    e.wake_delay = 0.4;
                    self.enemies.push(e);
                }
                self.juice.fx.push(ring(p, 4.0, 18.0, 0.3, DIRT));
                burst(&mut self.juice.fx, p, 10, DIRT, 70.0);
            }
        }
    }

    /// 도토가 묻는 도토리 지뢰(0052).
    fn bury_mines(&mut self, bury: usize) {
        // 도토가 묻으면: 하나는 플레이어 발밑, 나머지는 둘레에 고르게(바닥 안으로).
        if bury > 0 {
            self.juice.audio.play(sfx::Id::Burrow);
            let fuse = t::DOTO_MINE_FUSE
                * self
                    .enemies
                    .iter()
                    .find(|e| e.kind == Kind::Doto)
                    .map_or(1.0, |e| e.windup_mul);
            let base = rand::gen_range(0.0, std::f32::consts::TAU);
            for i in 0..bury {
                let mut p = self.player.pos;
                if i > 0 {
                    let a = base + i as f32 * std::f32::consts::TAU / (bury - 1) as f32;
                    p += Vec2::from_angle(a) * t::DOTO_MINE_SPREAD;
                }
                self.room.resolve_circle(&mut p, 6.0);
                self.hazards.mines.push(Mine {
                    pos: p,
                    time: 0.0,
                    fuse,
                });
                burst(&mut self.juice.fx, p, 6, DIRT, 50.0);
            }
        }
    }

    /// 묻힌 도토리의 터짐(터진 자리를 돌려준다), 투사체 한 틱, 되돌린 투사체 → 적.
    fn tick_mines_and_projectiles(
        &mut self,
        dt: f32,
        frozen: bool,
        deaths: &mut Vec<(Vec2, Kind)>,
    ) -> Vec<Vec2> {
        // 묻힌 도토리가 터진다(필살기 시동 중엔 멈춘다). 도토가 없으면 다 사라진다.
        let mut popped = Vec::new();
        if !self.enemies.iter().any(|e| e.kind == Kind::Doto) {
            self.hazards.mines.clear();
        }
        for m in self.hazards.mines.iter_mut().filter(|_| !frozen) {
            m.time += dt;
            if m.time >= m.fuse {
                popped.push(m.pos);
            }
        }
        self.hazards.mines.retain(|m| m.time < m.fuse);
        for &p in &popped {
            self.juice.audio.play(sfx::Id::Emerge);
            self.juice.feel.add_trauma(0.15);
            self.juice
                .fx
                .push(ring(p, 3.0, t::DOTO_MINE_RADIUS + 2.0, 0.25, PROJECTILE));
            burst(&mut self.juice.fx, p, 12, DIRT, 110.0);
        }
        for p in self.projectiles.iter_mut().filter(|_| !frozen) {
            p.update(&self.room, dt);
            if !p.alive {
                self.juice.fx.push(ring(p.pos, 2.0, 6.0, 0.12, PROJECTILE));
            }
        }
        // 되돌린 투사체 → 적
        for p in self
            .projectiles
            .iter_mut()
            .filter(|p| p.alive && p.friendly)
        {
            for e in self.enemies.iter_mut().filter(|e| e.alive && e.hittable()) {
                let rr = p.radius() + e.radius();
                if (e.pos - p.pos).length_squared() <= rr * rr {
                    e.hurt(
                        t::PARRY_REFLECT_DAMAGE,
                        p.vel.normalize_or_zero(),
                        160.0,
                        0.4,
                    );
                    p.alive = false;
                    self.juice.audio.play(sfx::Id::EnemyHit);
                    self.juice.fx.push(ring(p.pos, 3.0, 12.0, 0.18, WHITE));
                    if !e.alive {
                        enemy_down(
                            &mut self.juice.fx,
                            &mut self.juice.audio,
                            &mut self.juice.feel,
                            &mut self.stats.kills,
                            deaths,
                            e,
                            enemy_color(e.kind),
                        );
                    }
                    break;
                }
            }
        }
        popped
    }

    /// 3. 플레이어 공격 → 적·투사체·지뢰. 맞힌 확인(킥·멈춤·소리), 등딱지·자세 깨기, 벽 쿵 표시, 패링.
    fn apply_strike(&mut self, strike: Option<Strike>, deaths: &mut Vec<(Vec2, Kind)>) {
        // 3. 플레이어 공격 → 적·투사체
        if let Some(s) = strike {
            let (origin, facing) = (self.player.pos, self.player.facing);
            self.resolve_strike(s, origin, facing, false, deaths);
        }
    }

    /// 타격 하나를 적에게(0195로 갈라 냄): `origin`에서 밀어낸다(원형이면 `facing`). `ranged`면 도토의 탄·터짐 —
    /// 플레이어는 튕기거나 멈추지 않고, 손 타격만의 것(받아치기·탄 베기·도토리 파내기)은 없다.
    fn resolve_strike(
        &mut self,
        s: Strike,
        origin: Vec2,
        facing: Vec2,
        ranged: bool,
        deaths: &mut Vec<(Vec2, Kind)>,
    ) {
        {
            let mut shell_from = None;
            let mut broke_at = None;
            let mut hit_any = false;
            // 마무리 타·솟구치기가 맞힌 첫 자리(0090): 한 번만 쿵 소리와 충격 고리.
            let mut impact_at = None;
            for e in &mut self.enemies {
                // 닿나·튕기나·얼마나 밀리나는 적이 정한다(0170 R10). 월드는 결과로 연출·통계만.
                let dir = match e.receive(&s, origin, facing) {
                    enemy::Struck::Miss => continue,
                    // 밀치기(0214): 흙먼지만 — 맞힘 연출·소리·게이지·통계는 없다.
                    enemy::Struck::Shoved { dir } => {
                        spray(
                            &mut self.juice.fx,
                            e.pos - dir * e.radius(),
                            dir,
                            3,
                            DIRT,
                            90.0,
                        );
                        continue;
                    }
                    enemy::Struck::Deflected => {
                        shell_from = Some((e.pos, e.kind));
                        continue;
                    }
                    enemy::Struck::Hit { dir, broke_guard } => {
                        if broke_guard {
                            broke_at = Some(e.pos);
                        }
                        dir
                    }
                };
                if matches!(
                    s.kind,
                    crate::combat::StrikeKind::Claw(_) | crate::combat::StrikeKind::Heavy
                ) {
                    self.player.gain_sp(s.finisher);
                }
                if s.kind == crate::combat::StrikeKind::Heavy {
                    self.stats.heavy_hits += 1;
                }
                if s.finisher {
                    self.stats.finisher_hits += 1;
                }
                if s.kind == crate::combat::StrikeKind::Emerge {
                    self.stats.emerge_hits += 1;
                }
                strike_feel(&mut self.juice.feel, &self.player, &s, dir);
                hit_any = true;
                let hit_point = e.pos - dir * e.radius() * 0.5;
                if (s.finisher || s.kind == crate::combat::StrikeKind::Emerge)
                    && impact_at.is_none()
                {
                    impact_at = Some((hit_point, dir));
                }
                self.juice.fx.push(ring(hit_point, 2.0, 9.0, 0.12, WHITE));
                // 맞은 쪽으로 튀는 불똥(0081): 어디로 밀었는지 보인다.
                spray(&mut self.juice.fx, hit_point, dir, 4, WHITE, 140.0);
                if e.alive {
                    self.juice.audio.play(sfx::Id::EnemyHit);
                    self.juice.audio.play(sfx::Id::HitTick);
                } else {
                    if s.kind == crate::combat::StrikeKind::Emerge {
                        self.stats.emerge_kills += 1;
                    }
                    enemy_down(
                        &mut self.juice.fx,
                        &mut self.juice.audio,
                        &mut self.juice.feel,
                        &mut self.stats.kills,
                        deaths,
                        e,
                        enemy_color(e.kind),
                    );
                }
            }
            if hit_any && !ranged {
                // 맞히면 전진이 멈춘다(0168): 빗나감과 맞힘이 손에서 다르다.
                self.player.vel = Vec2::ZERO;
            }
            if let Some((p, dir)) = impact_at {
                self.juice.audio.play(sfx::Id::Impact);
                self.juice.fx.push(ring(p, 3.0, 22.0, 0.2, WHITE));
                spray(&mut self.juice.fx, p, dir, 8, WHITE, 200.0);
                clods(&mut self.juice.fx, p, 6, DIRT, 90.0);
                puff(&mut self.juice.fx, p, 3, self.biome.dust(), 6.0, 0.4);
            }
            if s.kind == crate::combat::StrikeKind::Emerge {
                let k = 1.0 + (t::EMERGE_CHARGE_RADIUS - 1.0) * self.player.burrow_charge();
                emerge_push(
                    &mut self.enemies,
                    origin,
                    self.player.facing,
                    s.id,
                    t::EMERGE_PUSH_RADIUS * k,
                );
            }
            if let Some((from, kind)) = shell_from {
                if !ranged {
                    self.player
                        .recoil(from, t::KONGI_RECOIL, t::KONGI_RECOIL_LOCK);
                }
                self.juice.audio.play(sfx::Id::WallHit);
                self.juice.feel.add_hitstop(t::PARRY_HITSTOP * 0.6);
                self.juice.feel.add_trauma(0.2);
                let p = (self.player.pos + from) * 0.5;
                self.juice.fx.push(ring(p, 2.0, 14.0, 0.2, SHELL));
                burst(&mut self.juice.fx, p, 8, SHELL, 100.0);
                self.juice.toast = Some((
                    if kind == Kind::Hedgehog {
                        |s| s.toast_spines
                    } else {
                        self.character.def().toast_shell
                    },
                    2.0,
                ));
            }
            if let Some(at) = broke_at {
                self.juice.audio.play(sfx::Id::DrillWall);
                self.juice.feel.add_trauma(0.3);
                self.juice.fx.push(ring(at, 4.0, 24.0, 0.3, SHELL));
                burst(&mut self.juice.fx, at, 14, SHELL, 130.0);
                self.juice.toast = Some((|s| s.toast_guard_break, 2.0));
            }
            if !ranged {
                self.strike_projectiles(&s);
                self.dig_mines(&s);
            }
        }
    }

    /// 타격이 투사체에 닿으면: 스윙 첫머리에 맞받으면 되돌리고(패링), 아니면 베어 없앤다.
    fn strike_projectiles(&mut self, s: &Strike) {
        let parry = self.player.parry_dir();
        let mut parried = false;
        for p in &mut self.projectiles {
            if !p.alive || p.friendly || !s.shape.overlaps_circle(p.pos, p.radius()) {
                continue;
            }
            // 스윙 첫머리에 맞받으면 되돌린다(패링), 아니면 베어 없앤다.
            match parry {
                Some(dir) if (-p.vel.normalize_or_zero()).dot(dir) > t::PARRY_FRONT => {
                    p.vel = dir * p.vel.length() * t::PARRY_REFLECT_SPEED;
                    p.friendly = true;
                    parried = true;
                }
                _ => {
                    p.alive = false;
                    self.juice.fx.push(ring(p.pos, 2.0, 8.0, 0.12, PROJECTILE));
                }
            }
        }
        if parried {
            self.parry_feedback();
        }
    }

    /// 발톱이 닿은 묻힌 도토리는 파낸다(터지지 않는다).
    fn dig_mines(&mut self, s: &Strike) {
        let before = self.hazards.mines.len();
        let mut dug = Vec::new();
        self.hazards.mines.retain(|m| {
            let hit = s.shape.overlaps_circle(m.pos, 5.0);
            if hit {
                dug.push(m.pos);
            }
            !hit
        });
        if self.hazards.mines.len() < before {
            self.juice.audio.play(sfx::Id::Claw(0));
        }
        for p in dug {
            self.juice.fx.push(ring(p, 2.0, 8.0, 0.15, DIRT));
            burst(&mut self.juice.fx, p, 6, DIRT, 70.0);
        }
    }

    /// 이번 틱에 쓰러진 적이 떨어뜨리는 것(루프 밖에서).
    fn settle_deaths(&mut self, deaths: Vec<(Vec2, Kind)>) {
        for (pos, kind) in deaths {
            // 포자버섯은 쓰러지면서도 한 번 뿜는다.
            if kind == Kind::SporeCap {
                self.hazards.clouds.push(Cloud { pos, time: 0.0 });
            }
            self.adv.last_kill = pos;
            self.drop_loot(pos, kind);
        }
    }

    /// 4. 적·투사체·위험물(구름·돌·충격파·덤불·지뢰) → 플레이어. 받아치기 판정도 여기.
    fn hit_player(&mut self, landed: &[Vec2], popped: &[Vec2]) {
        // 4. 적·투사체 → 플레이어
        if self.player.is_alive() && !self.player.is_invulnerable() {
            let pr = self.player.radius();
            let ppos = self.player.pos;
            let mut hurt_from = None;
            let parry = self.player.parry_dir();
            let mut parried = false;
            for e in &mut self.enemies {
                if let Some(dmg) = e.contact_damage() {
                    let rr = pr + e.contact_radius();
                    if (e.pos - ppos).length_squared() <= rr * rr {
                        let from = (e.pos - ppos).normalize_or_zero();
                        // 곧게 오는 것(돌진·물기·구르기·내리꽂기)만 받아친다(0160). 내려앉기·튀어나오기 같은 둘레 공격은
                        // 고리로 예고하고 비켜야 한다 — "띠·점선은 튕겨내고 고리는 피한다".
                        if (e.parryable() || self.player.parry_heavy())
                            && parry.is_some_and(|d| d.dot(from) > t::PARRY_FRONT)
                        {
                            // 패링: 돌진을 받아쳐 크게 멈칫하게 한다. 피해 없음.
                            // 되받아치기(재주, 0047)면 피해도 준다.
                            let counter = self.player.talents.parry_counter();
                            e.hurt(counter, from, t::PARRY_KNOCKBACK, t::PARRY_STUN);
                            e.on_parried();
                            parried = true;
                            break;
                        }
                        e.on_hit_player();
                        hurt_from = Some((e.pos, dmg));
                        break;
                    }
                }
            }
            // 두꺼비 혀(0146): 내민 틱에 혀 줄에 걸리면 맞고 두꺼비 쪽으로 끌려간다. 받아치면 두꺼비가 멈칫한다.
            let mut pull = None;
            if hurt_from.is_none() && !parried {
                for e in &mut self.enemies {
                    if !e.tongue_reaches(&self.room, ppos, pr) {
                        continue;
                    }
                    let from = (e.pos - ppos).normalize_or_zero();
                    if parry.is_some_and(|d| d.dot(from) > t::PARRY_FRONT) {
                        let counter = self.player.talents.parry_counter();
                        e.hurt(counter, from, t::PARRY_KNOCKBACK, t::PARRY_STUN);
                        parried = true;
                        break;
                    }
                    hurt_from = Some((e.pos, 1));
                    pull = Some(from * t::TOAD_PULL);
                    break;
                }
            }
            if parried {
                self.parry_feedback();
            }
            if hurt_from.is_none() && !parried {
                let rr = pr + t::SPORE_CLOUD_R;
                if let Some(c) = self.hazards.clouds.iter().find(|c| {
                    c.time >= t::SPORE_CLOUD_GRACE && (c.pos - ppos).length_squared() <= rr * rr
                }) {
                    hurt_from = Some((c.pos, 1));
                }
            }
            if hurt_from.is_none() && !parried {
                let rr = pr + t::ROCK_R;
                if let Some(p) = landed
                    .iter()
                    .find(|p| (**p - ppos).length_squared() <= rr * rr)
                {
                    hurt_from = Some((*p, 1));
                }
            }
            // 토끼풀의 충격파(0165): 퍼지는 고리가 지나는 순간 한 번, 멀리 밀린다. 땅속·깡충·드릴은 안 맞는다.
            if hurt_from.is_none() && !parried && self.player.on_foot() {
                for wv in self.hazards.waves.iter_mut() {
                    if wv.hit {
                        continue;
                    }
                    let r = t::CLOVER_WAVE_R * (wv.time / t::CLOVER_WAVE_TIME).min(1.0);
                    let d = (ppos - wv.pos).length();
                    if d <= r + pr && d >= r - 10.0 {
                        wv.hit = true;
                        hurt_from = Some((wv.pos, 1));
                        pull = Some(
                            (ppos - wv.pos).normalize_or(vec2(0.0, 1.0)) * t::CLOVER_WAVE_KNOCK,
                        );
                        break;
                    }
                }
            }
            // 덤불은 걸어 들어갈 때만(땅속·드릴·깡충은 아래로·위로 지나간다).
            let mut thorned = false;
            if hurt_from.is_none() && !parried && self.player.on_foot() {
                let rr = pr + t::THORN_R;
                if let Some(c) = self
                    .room
                    .thorns
                    .iter()
                    .find(|c| (**c - ppos).length_squared() <= rr * rr)
                {
                    hurt_from = Some((*c, 1));
                    thorned = true;
                }
            }
            if hurt_from.is_none() && !parried {
                let rr = pr + t::DOTO_MINE_RADIUS;
                if let Some(p) = popped
                    .iter()
                    .find(|p| (**p - ppos).length_squared() <= rr * rr)
                {
                    hurt_from = Some((*p, 1));
                }
            }
            if hurt_from.is_none() && !parried {
                for p in &mut self.projectiles {
                    let rr = pr + p.radius();
                    if p.alive && !p.friendly && (p.pos - ppos).length_squared() <= rr * rr {
                        p.alive = false;
                        hurt_from = Some((p.pos, t::PROJECTILE_DAMAGE));
                        break;
                    }
                }
            }
            if let Some((from, dmg)) = hurt_from {
                let dmg = if self.no_damage { 0 } else { dmg };
                if self.player.take_damage(from, dmg) {
                    if let Some(v) = pull {
                        self.player.vel = v;
                    }
                    if thorned && !self.story.thorn_told {
                        self.story.thorn_told = true;
                        self.juice.toast = Some((self.character.def().toast_thorn, 2.5));
                    }
                    self.juice.feel.add_hitstop(t::PLAYER_HURT_HITSTOP);
                    self.juice.feel.add_trauma(t::PLAYER_HURT_SHAKE);
                    self.juice.feel.add_kick(ppos - from, t::PLAYER_HURT_KICK);
                    self.juice.feel.hurt = t::PLAYER_HURT_EDGE;
                    self.juice.audio.play(sfx::Id::Ouch);
                    self.juice
                        .fx
                        .push(ring(self.player.pos, 4.0, 14.0, 0.2, RED));
                    if self.player.is_alive() {
                        self.juice.audio.play(sfx::Id::Hurt);
                    } else if self.inv.bag.take(Consumable::Worm) {
                        // 쓰러졌다: 쥬시 지렁이가 있으면 저절로 먹고 일어난다(목숨). 맞은 그 틱에 — 전에는 플레이어 틱
                        // (맞기 전 단계)에서 이번 틱의 Die 이벤트를 찾아서, 다음 틱엔 이벤트가 지워져 한 번도 못 일어났다(0200).
                        // 바로 일어나면 쓰러졌는지 몰랐다(사장, 0243) → 쓰러지는 소리·멈춤과 함께 잠깐 눕고(`State::Revive`),
                        // 지렁이를 다 먹으면 `Event::Rose`로 `worm_rise`.
                        self.player.start_revive();
                        self.juice.audio.play(sfx::Id::Die);
                        self.juice.feel.add_hitstop(t::REVIVE_HITSTOP);
                        burst(&mut self.juice.fx, self.player.pos, 10, MOLE, 70.0);
                    } else {
                        self.juice.audio.play(sfx::Id::Die);
                        burst(&mut self.juice.fx, self.player.pos, 16, MOLE, 90.0);
                    }
                }
            }
        }
    }

    /// 5. 적끼리 겹치지 않게 민다.
    fn separate_enemies(&mut self, dt: f32) {
        // 5. 적끼리 겹침 방지
        for i in 0..self.enemies.len() {
            for j in (i + 1)..self.enemies.len() {
                let (a, b) = self.enemies.split_at_mut(j);
                let (ea, eb) = (&mut a[i], &mut b[0]);
                let d = eb.pos - ea.pos;
                let min = ea.radius() + eb.radius();
                let dist2 = d.length_squared();
                if dist2 < min * min {
                    let push = if dist2 > 1e-4 {
                        d / dist2.sqrt()
                    } else {
                        vec2(1.0, 0.0)
                    } * t::ENEMY_SEPARATION_SPEED
                        * dt;
                    ea.pos -= push;
                    eb.pos += push;
                    let (ra, rb) = (ea.radius(), eb.radius());
                    self.room.resolve_circle(&mut ea.pos, ra);
                    self.room.resolve_circle(&mut eb.pos, rb);
                }
            }
        }
    }

    /// 6. 정리: 쓰러진 것 치우기, 효과 시계, 출구·보스 카드 시계, 발걸음 먼지.
    fn cleanup(&mut self, dt: f32) {
        // 6. 정리
        self.enemies.retain(|e| e.alive);
        self.projectiles.retain(|p| p.alive);
        self.tick_fx(dt);
        if self.exit_at.is_some() {
            self.exit_age += dt;
        }
        if let Some((_, _, left)) = &mut self.juice.boss_card {
            *left -= dt;
            if *left <= 0.0 {
                self.juice.boss_card = None;
            }
        }
        // 발걸음 먼지(0081): 걷는 동안 12px마다 발 뒤에 작은 먼지(효과가 넘치면 건너뛴다), 두 걸음에 한 번 발소리(0234).
        let moved = (self.player.pos - self.player.prev_pos).length();
        if self.player.is_alive() && self.player.on_foot() && moved > 0.2 {
            self.juice.step_dist += moved;
            if self.juice.step_dist > STEP_DUST_EVERY {
                self.juice.step_dist = 0.0;
                self.juice.steps = self.juice.steps.wrapping_add(1);
                if self.juice.steps.is_multiple_of(2) {
                    self.juice.audio.play(sfx::Id::Step);
                }
                if self.juice.fx.len() < FX_SOFT_CAP {
                    let back = -(self.player.pos - self.player.prev_pos).normalize_or_zero();
                    let at = self.player.pos + vec2(0.0, 5.0) + back * 3.0;
                    puff(&mut self.juice.fx, at, 1, self.biome.dust(), 3.0, 0.35);
                }
            }
        } else {
            self.juice.step_dist = STEP_DUST_EVERY * 0.6;
        }
    }

    /// 8. 웨이브(아레나).
    fn tick_waves(&mut self, dt: f32) {
        // 8. 웨이브(아레나)
        if self.auto_waves
            && self.adv.place == Place::Arena
            && self.player.is_alive()
            && self.enemies.is_empty()
        {
            self.respawn_timer -= dt;
            if self.respawn_timer <= 0.0 {
                self.next_wave();
                self.respawn_timer = t::WAVE_RESPAWN_DELAY;
            }
        }
    }

    /// 패링 성공 연출: 번쩍, 히트스톱, SP.
    /// 할 수 없는 행동(0149): 소리 하나 + HUD 칸 떨기. 칸이 이미 떨고 있으면 소리를 다시 내지 않는다(연타).
    fn deny_feedback(&mut self, k: usize) {
        if self.juice.deny[k] < t::DENY_TIME * 0.5 {
            self.juice.audio.play(sfx::Id::Deny);
        }
        self.juice.deny[k] = t::DENY_TIME;
    }

    fn parry_feedback(&mut self) {
        self.stats.parries += 1;
        self.player.on_parry();
        self.juice.feel.add_hitstop(t::PARRY_HITSTOP);
        self.juice.feel.add_trauma(t::PARRY_SHAKE);
        let p = self.player.pos + self.player.facing * 8.0;
        self.juice.fx.push(ring(p, 2.0, 18.0, 0.22, WHITE));
        self.juice.fx.push(ring(p, 2.0, 10.0, 0.15, PROJECTILE));
        burst(&mut self.juice.fx, p, 10, WHITE, 120.0);
        self.juice.audio.play(sfx::Id::Parry);
        self.juice.toast = Some((|s| s.toast_parry, 0.8));
    }

    /// 장신구·씨앗으로 능력치를 다시 계산한다.
    pub fn refresh_stats(&mut self) {
        self.player.set_stats(Stats::compute(
            &self.inv.charms,
            self.inv.root_hearts,
            &self.player.talents.mods(),
        ));
    }

    /// 퀵 슬롯 아이템을 먹기 시작한다. 체력이 가득이거나 슬롯이 비었으면 먹지 않는다(아깝지 않게).
    /// 회복은 다 먹었을 때(`Event::Ate`, docs/decisions/0038).
    fn start_eating(&mut self) {
        let p = &mut self.player;
        let c = self.inv.bag.selected;
        if self.inv.bag.count(c) == 0 {
            return;
        }
        if p.hp >= p.stats.max_hp {
            // 아무 일도 안 일어나면 고장 난 줄 안다(튜토리얼에서 막혔다, 0093): 왜 안 먹는지 알린다.
            p.drop_item_press();
            self.juice.toast = Some((|s| s.toast_eat_full, 1.6));
            self.deny_feedback(1);
            return;
        }
        p.start_eat(c);
    }

    /// 다 먹었다: 가방에서 꺼내 회복. 먹는 사이 체력이 가득 찼으면(샘 등) 꺼내지 않는다.
    fn finish_eating(&mut self, c: Consumable) {
        let p = &mut self.player;
        if p.hp >= p.stats.max_hp || !self.inv.bag.take(c) {
            return;
        }
        match c {
            Consumable::Berry => p.heal(p.talents.berry_heal() + p.stats.berry_heal),
            Consumable::Worm => p.heal(p.stats.max_hp),
        }
        self.juice.audio.play(sfx::Id::Heal);
        self.stats.items_used += 1;
        let pos = p.pos;
        self.juice.fx.push(ring(pos, 3.0, 14.0, 0.3, BERRY));
        burst(&mut self.juice.fx, pos, 8, BERRY, 60.0);
    }

    /// 쥬시 지렁이를 다 먹고 일어났다(`Event::Rose`, 0243): 주변을 밀어내고 빛·소리·한마디.
    fn worm_rise(&mut self) {
        let pos = self.player.pos;
        for e in &mut self.enemies {
            let d = e.pos - pos;
            if d.length() < t::REVIVE_PUSH_RADIUS && e.hittable() {
                e.hurt(0, d.normalize_or_zero(), t::REVIVE_PUSH_KNOCKBACK, 0.5);
            }
        }
        for p in &mut self.projectiles {
            if (p.pos - pos).length() < t::REVIVE_PUSH_RADIUS {
                p.alive = false;
            }
        }
        self.juice.audio.play(sfx::Id::Heal);
        self.juice
            .fx
            .push(ring(pos, 4.0, t::REVIVE_PUSH_RADIUS, 0.4, BERRY));
        self.juice
            .fx
            .push(ring(pos, 2.0, t::REVIVE_PUSH_RADIUS * 0.6, 0.3, WHITE));
        burst(&mut self.juice.fx, pos, 20, BERRY, 120.0);
        for i in 0..6 {
            let a = i as f32 * std::f32::consts::TAU / 6.0;
            self.juice
                .fx
                .push(glint(pos + vec2(a.cos(), a.sin()) * 10.0, WHITE, 0.4));
        }
        self.juice.feel.add_trauma(0.5);
        self.juice.feel.add_hitstop(0.08);
        self.juice.toast = Some((|s| s.toast_revive, 2.5));
    }

    /// 쓰러진 적이 떨어뜨린다: 반딧불 몇 마리 + 확률로 산딸기(loot::berry_chance).
    fn drop_loot(&mut self, pos: Vec2, kind: Kind) {
        if self.dream {
            return;
        }
        let (lo, hi) = loot::glowbugs_for(kind);
        // 굴이 넓어진 만큼 잡몹의 몫만 덜어 낸다(0244). 길의 주인은 그대로.
        let keep = if crate::bosses::of(kind).is_some() {
            1.0
        } else {
            t::KILL_LOOT_KEEP
        };
        if hi > 0 {
            for _ in 0..rand::gen_range(lo, hi + 1) {
                if rand::gen_range(0.0, 1.0) < keep {
                    self.push_loot(Loot::spawn(LootKind::Glowbug, pos));
                }
            }
        }
        if rand::gen_range(0.0, 1.0) < kind.species().seed_chance * keep {
            self.push_loot(Loot::spawn(LootKind::Seed, pos));
        }
        if kind != Kind::Dummy && rand::gen_range(0.0, 1.0) < t::WORM_CHANCE * self.knots.worm_mul()
        {
            self.push_loot(Loot::spawn(LootKind::Worm, pos));
        }
        let max = self.player.stats.max_hp;
        let missing = self.player.hp < max;
        let mut p = loot::berry_chance(self.player.hp, max, self.inv.kills_since_berry, kind);
        // 실뜨기 3: 덜 떨어진다. 불운 보정(확정)은 그대로 둔다.
        if p < 1.0 {
            p *= self.knots.berry_mul();
        }
        if p > 0.0 && rand::gen_range(0.0, 1.0) < p {
            self.push_loot(Loot::spawn(LootKind::Berry, pos));
            self.inv.kills_since_berry = 0;
        } else if missing && kind != Kind::Dummy {
            self.inv.kills_since_berry += 1;
        }
    }

    /// 보간이 멈춘 화면(일시정지·히트스톱) 위에서 미끄러지지 않게 prev를 현재로 맞춘다.
    /// 입자 효과만 한 틱(컷씬에서도 돈다, 0159). 월드 틱의 6단계.
    pub fn tick_fx(&mut self, dt: f32) {
        for f in &mut self.juice.fx {
            f.time += dt;
            f.pos += f.vel * dt;
            match &mut f.kind {
                // 흙덩이는 공기 저항이 작고 중력으로 떨어진다. 바닥에 닿으면 한 번 작게 튀고 미끄러진다.
                FxKind::Clod { z, vz, .. } => {
                    f.vel *= 0.97;
                    *vz -= CLOD_GRAVITY * dt;
                    *z += *vz * dt;
                    if *z < 0.0 {
                        *z = 0.0;
                        *vz = -*vz * 0.35;
                        f.vel *= 0.5;
                    }
                }
                // 실은 천천히 떠오른다.
                FxKind::Thread { angle, .. } => {
                    f.vel *= 0.95;
                    *angle += dt * 1.5;
                }
                // 꽃잎은 팔랑이며 가라앉는다: 옆으로 흔들리고 아래로 천천히.
                FxKind::Petal { phase, .. } => {
                    *phase += dt * 5.0;
                    f.vel.x = f.vel.x * 0.96 + phase.sin() * 14.0 * dt;
                    f.vel.y = (f.vel.y + 40.0 * dt).min(16.0);
                }
                _ => f.vel *= 0.9,
            }
        }
        // 끝난 효과를 지운다. 이미 거른 것(앞쪽)이 몇 개 남는지 세어 둔다(thin_fx가 새것만 보게).
        let checked = self.juice.fx_checked;
        let (mut idx, mut kept) = (0, 0);
        self.juice.fx.retain(|f| {
            let keep = f.time < f.dur;
            if idx < checked && keep {
                kept += 1;
            }
            idx += 1;
            keep
        });
        self.juice.fx_checked = kept;
    }

    /// 컷씬 이펙트 묶음(0159)을 `at`에 낸다.
    pub fn cue(&mut self, cue: Cue, at: Vec2) {
        match cue {
            Cue::Unbind => {
                self.juice.audio.play(sfx::Id::Unbind);
                self.juice.feel.add_trauma(0.3);
                self.juice.fx.push(ring(at, 4.0, 30.0, 0.5, WHITE));
                self.juice.fx.push(ring(at, 2.0, 18.0, 0.35, THREAD));
                unravel(&mut self.juice.fx, at, 14.0);
                unravel(&mut self.juice.fx, at, 22.0);
                for k in 0..10 {
                    let ang = k as f32 / 10.0 * std::f32::consts::TAU;
                    let r = 6.0 + (k % 3) as f32 * 6.0;
                    let mut g = glint(
                        at + vec2(ang.cos() * r, ang.sin() * r - 4.0),
                        Color::new(1.0, 0.95, 0.85, 1.0),
                        0.6,
                    );
                    g.time = -0.05 * k as f32;
                    self.juice.fx.push(g);
                }
            }
            Cue::Petals(bloom) => {
                let color = if bloom {
                    Color::new(1.0, 0.70, 0.80, 1.0)
                } else {
                    Color::new(0.74, 0.56, 0.60, 1.0)
                };
                petals(&mut self.juice.fx, at, 18, color, 70.0);
            }
            Cue::Rise => {
                self.juice.audio.play(sfx::Id::Rumble);
                self.juice.feel.add_trauma(0.4);
                self.juice.fx.push(ring(at, 26.0, 4.0, 0.5, THREAD));
                self.juice
                    .fx
                    .push(ring(at, 6.0, 40.0, 0.6, Color::new(0.5, 0.3, 0.6, 1.0)));
                puff(
                    &mut self.juice.fx,
                    at,
                    6,
                    Color::new(0.30, 0.22, 0.34, 1.0),
                    10.0,
                    0.7,
                );
                clods(&mut self.juice.fx, at, 8, DIRT, 70.0);
            }
            Cue::Vanish => {
                self.juice.audio.play(sfx::Id::Burrow);
                self.juice.feel.add_trauma(0.15);
                self.juice.fx.push(ring(at, 16.0, 2.0, 0.35, THREAD));
                puff(
                    &mut self.juice.fx,
                    at,
                    5,
                    Color::new(0.30, 0.22, 0.34, 1.0),
                    8.0,
                    0.6,
                );
                clods(&mut self.juice.fx, at, 6, DIRT, 60.0);
                unravel(&mut self.juice.fx, at, 10.0);
            }
            Cue::Wrap => {
                self.juice.audio.play(sfx::Id::Rumble);
                self.juice.feel.add_trauma(0.35);
                for (k, r) in [48.0, 36.0, 24.0].into_iter().enumerate() {
                    let mut f = ring(at, r, 3.0, 0.7, THREAD);
                    f.time = -0.18 * k as f32;
                    self.juice.fx.push(f);
                }
                puff(
                    &mut self.juice.fx,
                    at,
                    8,
                    Color::new(0.24, 0.14, 0.34, 1.0),
                    12.0,
                    0.9,
                );
                unravel(&mut self.juice.fx, at, 18.0);
                unravel(&mut self.juice.fx, at, 28.0);
            }
            Cue::Eyes => {
                let violet = Color::new(0.80, 0.45, 1.0, 1.0);
                for (k, dx) in [-4.0, 4.0].into_iter().enumerate() {
                    let mut g = glint(at + vec2(dx, -6.0), violet, 1.6);
                    g.time = -0.1 * k as f32;
                    self.juice.fx.push(g);
                }
            }
            Cue::Trail => self.story.petals.push(at),
            Cue::Leave => {
                petals(
                    &mut self.juice.fx,
                    at,
                    6,
                    Color::new(0.74, 0.56, 0.60, 1.0),
                    30.0,
                );
            }
            Cue::Hatch => {
                self.juice.audio.play(sfx::Id::Unbind);
                let cocoons = std::mem::take(&mut self.hazards.cocoons);
                for (k, c) in cocoons.into_iter().enumerate() {
                    self.juice.fx.push(ring(c, 3.0, 22.0, 0.4, WHITE));
                    unravel(&mut self.juice.fx, c, 12.0);
                    petals(
                        &mut self.juice.fx,
                        c,
                        8,
                        Color::new(1.0, 0.70, 0.80, 1.0),
                        40.0,
                    );
                    for i in 0..5 {
                        let ang = i as f32 / 5.0 * std::f32::consts::TAU + k as f32;
                        let mut g =
                            glint(c + vec2(ang.cos() * 8.0, ang.sin() * 5.0 - 3.0), WHITE, 0.5);
                        g.time = -0.06 * i as f32;
                        self.juice.fx.push(g);
                    }
                }
            }
            Cue::Sparkle => {
                self.juice.audio.play(sfx::Id::Glowbug);
                for k in 0..6 {
                    let ang = k as f32 / 6.0 * std::f32::consts::TAU + 0.4;
                    let mut g = glint(
                        at + vec2(ang.cos() * 9.0, ang.sin() * 6.0 - 3.0),
                        WHITE,
                        0.5,
                    );
                    g.time = -0.06 * k as f32;
                    self.juice.fx.push(g);
                }
            }
        }
    }

    /// 길의 샘(0159): 발을 담그면 가득 회복. 한 번만. 회복했으면 true(소리는 부르는 쪽).
    /// 흰 실 그물(0165): 걸어 들어가면 느려진다(땅속·깡충·드릴은 지나간다). 처음엔 한마디. 플레이어 틱 전에.
    fn web_tick(&mut self) {
        self.player.slow = false;
        if self.room.webs.is_empty() || !self.player.on_foot() {
            return;
        }
        let pp = self.player.pos;
        let rr = self.player.radius() + t::WEB_R - 3.0;
        if !self
            .room
            .webs
            .iter()
            .any(|c| (*c - pp).length_squared() <= rr * rr)
        {
            return;
        }
        self.player.slow = true;
        if !self.story.web_told {
            self.story.web_told = true;
            self.juice.toast = Some((self.character.def().toast_web, 2.5));
        }
    }

    /// 토끼풀이 발을 굴렀다(0165): 충격파 고리 + 흙·흔들림, 플레이어 둘레에 흰 실 그물이 몇 개 떨어진다.
    fn thump_at(&mut self, at: Vec2) {
        self.juice.audio.play(sfx::Id::Impact);
        self.juice.feel.add_trauma(0.45);
        self.juice.feel.add_hitstop(0.03);
        self.hazards.waves.push(Wave {
            pos: at,
            time: 0.0,
            hit: false,
        });
        clods(&mut self.juice.fx, at, 10, DIRT, 90.0);
        self.juice.fx.push(ring(at, 6.0, 20.0, 0.2, DIRT));
        let pp = self.player.pos;
        let mut placed = 0;
        for k in 0..12 {
            if placed >= t::CLOVER_WAVE_WEBS {
                break;
            }
            let ang = rand::gen_range(0.0, std::f32::consts::TAU) + k as f32;
            let d = rand::gen_range(22.0, 56.0);
            let p = pp + vec2(ang.cos() * d, ang.sin() * d);
            if self.room.is_solid(p, t::WEB_R) || !self.room.inner.contains(p) {
                continue;
            }
            if self
                .room
                .webs
                .iter()
                .any(|c| (*c - p).length() < t::WEB_R * 2.0)
            {
                continue;
            }
            self.room.webs.push(p);
            // 천장에서 떨어진 실뭉치: 흰 보풀.
            puff(
                &mut self.juice.fx,
                p,
                4,
                Color::new(0.92, 0.90, 0.98, 1.0),
                6.0,
                0.5,
            );
            placed += 1;
        }
    }

    /// 토끼풀이 실 한 가닥을 뻗었다(0165): 줄 위에 서 있으면 받아치거나(끊김·토끼풀 멈칫) 감겨 끌려간다(피해 없음).
    /// 땅속·깡충·무적이면 안 걸린다.
    fn thread_at(&mut self, at: Vec2, dir: Vec2) {
        let dir = dir.normalize_or(vec2(0.0, 1.0));
        let reach = self.room.ray_len(at, dir, 1.0, t::CLOVER_THREAD_REACH);
        let end = at + dir * reach;
        self.hazards.thread_lash = Some((at, end, 0.0));
        self.juice.audio.play(sfx::Id::Shoot);
        let ppos = self.player.pos;
        let pr = self.player.radius();
        if !self.player.is_alive() || !self.player.on_foot() || self.player.is_invulnerable() {
            return;
        }
        if dist_to_segment(ppos, at, end) > pr + t::CLOVER_THREAD_WIDTH {
            return;
        }
        let from = (at - ppos).normalize_or_zero();
        if self
            .player
            .parry_dir()
            .is_some_and(|d| d.dot(from) > t::PARRY_FRONT)
        {
            let counter = self.player.talents.parry_counter();
            if let Some(e) = self.enemies.iter_mut().find(|e| e.kind == Kind::Clover) {
                e.hurt(counter, from, t::PARRY_KNOCKBACK, t::PARRY_STUN);
                e.on_parried();
            }
            self.hazards.thread_lash = Some((at, ppos, 0.1));
            self.player.on_parry();
            self.juice.audio.play(sfx::Id::Parry);
            self.juice.toast = Some((|s| s.toast_parry, 0.8));
            self.juice.feel.add_hitstop(t::PARRY_HITSTOP);
            self.juice.fx.push(ring(ppos, 4.0, 18.0, 0.2, WHITE));
            unravel(&mut self.juice.fx, ppos + from * 8.0, 10.0);
            return;
        }
        // 감긴다: 토끼풀 쪽으로 끌려가고 잠깐 손을 못 쓴다.
        self.player.recoil(
            ppos - from,
            0.0,
            t::CLOVER_PULL_LOCK * self.player.stats.wrap_mul,
        );
        self.player.vel = from * t::CLOVER_PULL;
        self.juice.audio.play(sfx::Id::Rumble);
        self.juice.feel.add_trauma(0.25);
        self.juice.fx.push(ring(ppos, 6.0, 18.0, 0.3, THREAD));
    }

    /// 아기들의 고치(0165): 발톱이 닿으면 튕긴다(피해 없음). 처음엔 한마디.
    fn cocoon_hit(&mut self, s: &Strike) {
        if self.hazards.cocoon_bounce_id == s.id {
            return;
        }
        let Some(c) = self
            .hazards
            .cocoons
            .iter()
            .copied()
            .find(|c| s.shape.overlaps_circle(*c, t::COCOON_R))
        else {
            return;
        };
        self.hazards.cocoon_bounce_id = s.id;
        self.juice.audio.play(sfx::Id::WallHit);
        self.juice.feel.add_hitstop(0.04);
        self.juice.fx.push(ring(c, 4.0, 14.0, 0.2, WHITE));
        puff(
            &mut self.juice.fx,
            c,
            3,
            Color::new(0.92, 0.90, 0.98, 1.0),
            5.0,
            0.4,
        );
        if !self.story.cocoon_told {
            self.story.cocoon_told = true;
            self.juice.toast = Some((|s| s.toast_cocoon, 2.5));
        }
    }

    /// 흰 실 그물(0165): 플레이어의 공격(발톱·솟구침·드릴)이 닿으면 끊긴다 — 흰 고리·보풀, 실 풀리는 소리.
    fn web_cut(&mut self, s: &Strike) {
        let mut cut = Vec::new();
        self.room.webs.retain(|c| {
            if s.shape.overlaps_circle(*c, t::WEB_R) {
                cut.push(*c);
                false
            } else {
                true
            }
        });
        for c in cut {
            self.juice.audio.play(sfx::Id::Unbind);
            self.juice.fx.push(ring(c, 3.0, 16.0, 0.3, WHITE));
            puff(
                &mut self.juice.fx,
                c,
                5,
                Color::new(0.92, 0.90, 0.98, 1.0),
                6.0,
                0.5,
            );
            unravel(&mut self.juice.fx, c, 8.0);
        }
    }

    /// 부술 수 있는 것(0175): 플레이어의 공격이 닿으면 깨진다 — 흙덩이·먼지·퍽 소리, 운에 따라 소모품(`BREAK_*`). 꿈에서는 빈손.
    fn prop_break(&mut self, s: &Strike) {
        let mut broken = Vec::new();
        let mut i = 0;
        while i < self.room.props.len() {
            let p = self.room.props[i];
            if let PropKind::Breakable(b) = p.kind {
                if s.shape.overlaps_circle(p.rect.center(), t::BREAK_R) {
                    broken.push((p.rect.center(), b));
                    self.room.remove_prop(i);
                    continue;
                }
            }
            i += 1;
        }
        for (at, b) in broken {
            self.stats.broken += 1;
            self.juice.audio.play(sfx::Id::Break);
            let (chunk, dust) = match b {
                crate::room::Breakable::Mound => (DIRT, Color::new(0.6, 0.48, 0.34, 1.0)),
                crate::room::Breakable::Gall => (
                    Color::new(0.45, 0.28, 0.2, 1.0),
                    Color::new(0.5, 0.36, 0.26, 1.0),
                ),
                crate::room::Breakable::RockPile => (ROCK, Color::new(0.55, 0.52, 0.52, 1.0)),
                crate::room::Breakable::SilkBall => (
                    Color::new(0.92, 0.90, 0.98, 1.0),
                    Color::new(0.85, 0.82, 0.92, 1.0),
                ),
            };
            clods(&mut self.juice.fx, at, 5, chunk, 70.0);
            puff(&mut self.juice.fx, at, 4, dust, 5.0, 0.45);
            self.juice.feel.add_trauma(0.08);
            if self.dream {
                continue;
            }
            let roll = rand::gen_range(0.0, 1.0);
            let kind = if roll < t::BREAK_SEED {
                Some(LootKind::Seed)
            } else if roll < t::BREAK_SEED + t::BREAK_BERRY {
                Some(LootKind::Berry)
            } else if roll < t::BREAK_SEED + t::BREAK_BERRY + t::BREAK_GLOWBUG {
                Some(LootKind::Glowbug)
            } else if roll < t::BREAK_SEED + t::BREAK_BERRY + t::BREAK_GLOWBUG + t::BREAK_WORM {
                Some(LootKind::Worm)
            } else {
                None
            };
            if let Some(k) = kind {
                self.push_loot(Loot::spawn(k, at));
            }
        }
    }

    /// 시든 꽃잎(0164): 밟으면 줍는다 — 토스트, 분홍 반짝임, 도감 '???'. 앱이 기록에 남긴다(`AdvEvent::PetalFound`).
    pub fn petal_tick(&mut self) -> bool {
        if !self.player.is_alive() || !self.player.on_foot() {
            return false;
        }
        let p = self.player.pos;
        let Some(i) = self
            .story
            .petals
            .iter()
            .position(|q| (p - *q).length() <= 10.0)
        else {
            return false;
        };
        let at = self.story.petals.remove(i);
        // 처음 주운 꽃잎은 "분홍빛 + 이 길로 갔다", 예전에 주운 적이 있으면 "또 … 이 길로"(0190 → 0194).
        let first = !self.story.petal_found;
        self.story.petal_found = true;
        self.juice.toast = Some(if first {
            (|s| s.toast_petal, 4.0)
        } else {
            (|s| s.toast_petal_trail, 3.0)
        });
        self.juice.fx.push(glint(
            at + vec2(0.0, -4.0),
            Color::new(1.0, 0.72, 0.80, 1.0),
            0.6,
        ));
        petals(
            &mut self.juice.fx,
            at,
            4,
            Color::new(0.74, 0.56, 0.60, 1.0),
            24.0,
        );
        self.adv.events.push(crate::adventure::AdvEvent::PetalFound);
        true
    }

    pub fn spring_tick(&mut self) -> bool {
        let Some(sp) = &mut self.spring else {
            return false;
        };
        if sp.drunk || !self.player.is_alive() || !self.player.on_foot() {
            return false;
        }
        if (self.player.pos - sp.pos).length() > t::SPRING_RADIUS {
            return false;
        }
        sp.drunk = true;
        let at = sp.pos;
        self.player.heal(self.player.stats.max_hp);
        self.stats.springs += 1;
        self.juice.toast = Some((|s| s.toast_spring, 2.5));
        self.juice
            .fx
            .push(ring(at, 6.0, 26.0, 0.5, Color::new(0.75, 0.92, 1.0, 1.0)));
        for k in 0..8 {
            let ang = k as f32 / 8.0 * std::f32::consts::TAU;
            let mut g = glint(
                at + vec2(ang.cos() * 12.0, ang.sin() * 6.0),
                Color::new(0.85, 0.97, 1.0, 1.0),
                0.7,
            );
            g.vel = vec2(0.0, -14.0);
            g.time = -0.07 * k as f32;
            self.juice.fx.push(g);
        }
        true
    }

    /// 가장 깊은 곳(0162) 한 틱: 당기기 예고 → 뻗기(판정 한 번) → 감김, 매듭 떨림, 셋 다 끊으면 엔딩, 세 번 감기면 잠.
    fn deep_tick(&mut self, dt: f32) {
        let Some(d) = self.deep.as_mut() else {
            return;
        };
        d.time += dt;
        for k in &mut d.knots {
            k.shake = (k.shake - dt).max(0.0);
        }
        if let Some(was) = d.done_t {
            // 놓아주기(0163): 셋 다 끊긴 뒤 할매의 남은 실이 풀리고 흰 꽃잎이 날린다, 빛이 커진다. 그 뒤 엔딩.
            let now = was + dt;
            let fire = was < t::DEEP_DONE_DELAY && now >= t::DEEP_DONE_DELAY;
            d.done_t = Some(if fire { f32::MAX } else { now });
            let c = d.center;
            self.deep_release_fx(c, was, now);
            if fire {
                self.adv.events.push(crate::adventure::AdvEvent::DeepDone);
            }
            return;
        }
        if d.asleep {
            return;
        }
        // 감겨 있다: 가운데로 끌려간다.
        if d.wrapped > 0.0 {
            d.wrapped -= dt;
            let to = d.center + vec2(0.0, 30.0) - self.player.pos;
            if to.length() > 4.0 {
                let step = to.normalize() * t::DEEP_PULL_SPEED * dt;
                self.player.pos += step.clamp_length_max(to.length());
            }
            if d.wrapped <= 0.0 {
                d.wrapped = 0.0;
                if d.wraps >= t::DEEP_WRAPS {
                    d.asleep = true;
                    self.adv.events.push(crate::adventure::AdvEvent::DeepSleep);
                }
            }
            return;
        }
        match d.pull.as_mut() {
            None => {
                d.next_pull -= dt;
                if d.next_pull <= 0.0 {
                    let to = self.player.pos - d.center;
                    let dir = to.normalize_or(vec2(0.0, 1.0));
                    d.pull = Some(Pull {
                        dir,
                        len: (to.length() + 24.0).min(t::DEEP_PULL_REACH),
                        time: 0.0,
                        judged: false,
                        snapped: false,
                    });
                }
            }
            Some(p) => {
                p.time += dt;
                if p.time >= t::DEEP_PULL_WINDUP && !p.judged {
                    p.judged = true;
                    // 뻗는 틱: 실 줄에 걸렸나. 땅속·무적이면 안 걸린다. 받아치면 끊긴다.
                    let (a, b) = (d.center, d.center + p.dir * p.len);
                    let pp = self.player.pos;
                    let caught = self.player.is_alive()
                        && !self.player.is_invulnerable()
                        && dist_to_segment(pp, a, b) <= self.player.radius() + t::DEEP_THREAD_W;
                    if caught {
                        let from = (d.center - pp).normalize_or_zero();
                        if self
                            .player
                            .parry_dir()
                            .is_some_and(|dir| dir.dot(from) > t::PARRY_FRONT)
                        {
                            p.snapped = true;
                            self.player.on_parry();
                            self.juice.audio.play(sfx::Id::Parry);
                            self.juice.toast = Some((|s| s.toast_parry, 0.8));
                            self.juice.feel.add_hitstop(t::PARRY_HITSTOP);
                            self.juice.fx.push(ring(pp, 4.0, 18.0, 0.2, WHITE));
                            unravel(&mut self.juice.fx, pp + from * 8.0, 10.0);
                        } else {
                            d.wraps += 1;
                            let wrap = t::DEEP_WRAP_TIME * self.player.stats.wrap_mul;
                            d.wrapped = wrap;
                            self.player.recoil(pp - from, 0.0, wrap);
                            self.juice.audio.play(sfx::Id::Rumble);
                            self.juice.feel.add_trauma(0.3);
                            self.juice.fx.push(ring(pp, 6.0, 20.0, 0.3, THREAD));
                            // 가장 가까운 안 끊긴 매듭이 다시 조여진다.
                            if let Some(k) = d.knots.iter_mut().filter(|k| !k.cut).min_by(|x, y| {
                                (x.pos - pp)
                                    .length_squared()
                                    .total_cmp(&(y.pos - pp).length_squared())
                            }) {
                                k.hp = t::DEEP_KNOT_HP;
                                k.shake = 0.5;
                                self.juice.fx.push(ring(k.pos, 16.0, 4.0, 0.4, THREAD));
                            }
                            if !d.hinted {
                                d.hinted = true;
                                self.juice.toast = Some((|s| s.deep_wrap, 3.5));
                            }
                        }
                    }
                }
                if p.time >= t::DEEP_PULL_WINDUP + t::DEEP_PULL_TIME {
                    d.pull = None;
                    // 2단계(0233): 매듭이 하나 남으면 한 번 더 곧바로.
                    let last = d.cut_count() + 1 >= d.knots.len();
                    d.echo = last && !d.echo;
                    d.next_pull = if d.echo {
                        t::DEEP_ECHO_GAP
                    } else {
                        d.pull_every()
                    };
                }
            }
        }
    }

    /// 놓아주기 이펙트(0163): `was`→`now` 사이에 지난 박자마다 한 번. 0.4초 실이 풀림, 1.0초 흰 꽃잎, 1.6초 더 큰 풀림,
    /// 2.3초 반짝임과 꽃잎. 그림은 `render::draw_deep`가 `done_t`로 빛·눈을 바꾼다.
    fn deep_release_fx(&mut self, c: Vec2, was: f32, now: f32) {
        let crossed = |at: f32| was < at && now >= at;
        let white = Color::new(0.98, 0.96, 1.0, 1.0);
        if crossed(0.4) {
            self.juice.audio.play(sfx::Id::Unbind);
            self.juice.feel.add_trauma(0.25);
            self.juice.fx.push(ring(c, 10.0, 60.0, 0.7, WHITE));
            self.juice.fx.push(ring(c, 6.0, 40.0, 0.5, THREAD));
            for dx in [-30.0, 0.0, 30.0] {
                unravel(&mut self.juice.fx, c + vec2(dx, 4.0), 22.0);
            }
        }
        if crossed(1.0) {
            petals(&mut self.juice.fx, c + vec2(0.0, -10.0), 24, white, 60.0);
        }
        if crossed(1.6) {
            self.juice.audio.play(sfx::Id::Unbind);
            self.juice.feel.add_trauma(0.2);
            self.juice.fx.push(ring(c, 20.0, 90.0, 0.9, WHITE));
            unravel(&mut self.juice.fx, c, 46.0);
            unravel(&mut self.juice.fx, c + vec2(0.0, 12.0), 34.0);
        }
        if crossed(2.3) {
            self.juice.audio.play(sfx::Id::Glowbug);
            for k in 0..10 {
                let ang = k as f32 / 10.0 * std::f32::consts::TAU;
                let mut g = glint(c + vec2(ang.cos() * 40.0, ang.sin() * 26.0), white, 0.7);
                g.time = -0.05 * k as f32;
                self.juice.fx.push(g);
            }
            petals(&mut self.juice.fx, c + vec2(0.0, -20.0), 16, white, 50.0);
        }
    }

    /// 플레이어의 공격이 매듭에 닿았나(0162). 평타 1·마무리 2, 강공격은 두 배. 끊기면 실이 풀린다.
    fn deep_hit(&mut self, s: &Strike) {
        let Some(d) = self.deep.as_mut() else {
            return;
        };
        if d.done_t.is_some() || d.wrapped > 0.0 {
            return;
        }
        let mut cut_now = None;
        for (i, k) in d.knots.iter_mut().enumerate() {
            if k.cut || k.last_hit_by == s.id || !s.shape.overlaps_circle(k.pos, t::DEEP_KNOT_R) {
                continue;
            }
            k.last_hit_by = s.id;
            let dmg = if s.kind == crate::combat::StrikeKind::Heavy {
                s.damage * t::DEEP_HEAVY_MUL
            } else {
                s.damage
            };
            k.hp -= dmg;
            k.shake = 0.35;
            self.juice.feel.add_hitstop(s.hitstop.min(0.08));
            self.juice.fx.push(ring(k.pos, 3.0, 12.0, 0.15, WHITE));
            spray(
                &mut self.juice.fx,
                k.pos,
                s.shape_dir_or(self.player.facing),
                4,
                THREAD,
                120.0,
            );
            self.juice.audio.play(sfx::Id::EnemyHit);
            if k.hp <= 0 {
                k.cut = true;
                cut_now = Some(i);
            }
        }
        if let Some(i) = cut_now {
            let at = d.knots[i].pos;
            let n = d.cut_count();
            self.juice.audio.play(sfx::Id::Unbind);
            self.juice.feel.add_trauma(0.35);
            self.juice.fx.push(ring(at, 4.0, 30.0, 0.5, WHITE));
            self.juice.fx.push(ring(at, 2.0, 18.0, 0.35, THREAD));
            unravel(&mut self.juice.fx, at, 14.0);
            unravel(&mut self.juice.fx, at, 22.0);
            // 할매의 한마디(0162): 첫 매듭, 둘째 매듭.
            self.juice.toast = Some((
                match n {
                    1 => |s| s.deep_k1,
                    2 => |s| s.deep_k2,
                    _ => |s| s.deep_k3,
                },
                3.5,
            ));
            if d.all_cut() {
                d.done_t = Some(0.0);
                d.pull = None;
            }
        }
    }

    pub fn freeze(&mut self) {
        self.freeze_interp();
    }

    fn freeze_interp(&mut self) {
        self.player.prev_pos = self.player.pos;
        for e in &mut self.enemies {
            e.prev_pos = e.pos;
        }
        for p in &mut self.projectiles {
            p.prev_pos = p.pos;
        }
        for l in &mut self.loot {
            l.prev_pos = l.pos;
        }
        if let Some(n) = self.npc.as_mut() {
            n.prev_pos = n.pos;
        }
        for n in &mut self.neighbors {
            n.prev_pos = n.pos;
        }
        self.adv.prev_cam = self.adv.cam;
    }

    pub fn next_wave(&mut self) {
        self.wave += 1;
        if self.biome_by_wave {
            self.biome = Biome::for_wave(self.wave);
        }
        let (chargers, shooters) = match self.wave {
            1 => (1, 1),
            2 => (2, 1),
            3 => (2, 2),
            _ => (3, 2),
        };
        let ppos = self.player.pos;
        let mut points: Vec<Vec2> = self
            .room
            .spawn_points
            .iter()
            .copied()
            .filter(|p| (*p - ppos).length() >= t::SPAWN_MIN_DIST)
            .collect();
        // 섞기 (Fisher–Yates)
        for i in (1..points.len()).rev() {
            let j = rand::gen_range(0, i + 1);
            points.swap(i, j);
        }
        let mut it = points.into_iter().cycle();
        let depth = self.wave / 3;
        for _ in 0..(chargers + shooters) {
            if let Some(p) = it.next() {
                let kind = crate::species::roll(rand::gen_range(0.0, 1.0), depth, self.biome);
                self.spawn_mob(kind, p);
            }
        }
    }

    /// 적 하나(생쥐는 떼로 둘러서).
    pub fn spawn_mob(&mut self, kind: Kind, p: Vec2) {
        for off in crate::species::pack_offsets(kind) {
            self.enemies.push(Enemy::new(kind, p + off));
        }
    }
}

// ---- 팔레트(렌더와 공유) ---------------------------------------------------------
pub const MOLE: Color = Color::new(0.90, 0.71, 0.47, 1.0);
pub const DIRT: Color = Color::new(0.55, 0.40, 0.28, 1.0);
/// 가시뿌리 덤불(0147)·무너지는 돌.
pub const THORN: Color = Color::new(0.52, 0.34, 0.58, 1.0);
pub const ROCK: Color = Color::new(0.46, 0.42, 0.44, 1.0);
pub const PROJECTILE: Color = Color::new(1.0, 0.85, 0.30, 1.0);
/// 분노 게이지·드릴 불꽃(주황).
pub const RAGE: Color = Color::new(1.0, 0.55, 0.22, 1.0);
pub const POISON: Color = Color::new(0.55, 0.90, 0.35, 1.0);
/// 반딧불 빛.
pub const GLOW: Color = Color::new(0.85, 1.0, 0.45, 1.0);
/// 씨앗(상점 화폐).
pub const SEED: Color = Color::new(0.86, 0.68, 0.40, 1.0);
/// 산딸기.
pub const BERRY: Color = Color::new(0.93, 0.30, 0.38, 1.0);
/// 모모의 꽃잎(이어 뛰기 착지, 0056).
pub const PETAL: Color = Color::new(1.0, 0.70, 0.82, 1.0);
/// 콩이의 등딱지(튕김·열림 번쩍임, 0054).
pub const SHELL: Color = Color::new(0.75, 0.90, 1.0, 1.0);

pub fn enemy_color(kind: Kind) -> Color {
    kind.species().color
}

/// 밀쳐내기(0090): 솟구치기에 안 맞은 둘레의 잡몹을 피해 없이 바깥으로 민다. 가까울수록 세게. 보스는 버틴다.
/// 같은 공격 번호(`id`)로 맞은 적은 이미 맞았으니 건너뛴다.
/// `radius`: 미는 반지름(모은 힘만큼 넓다, 0158).
impl World {
    /// 엔딩 뒤 마을 땅 위로 튀어나온다(0204, 올라오는 전환의 끝): 흙이 터지는 고리 둘 + 흙덩이 + 쿵.
    pub(crate) fn surface_burst(&mut self) {
        let at = self.player.pos;
        self.juice.audio.play(sfx::Id::EmergeBig);
        self.juice.fx.push(ring(at, 6.0, 24.0, 0.3, WHITE));
        self.juice.fx.push(ring(at, 10.0, 36.0, 0.42, DIRT));
        self.juice.feel.add_trauma(t::EMERGE_BURST_SHAKE * 2.0);
        burst(&mut self.juice.fx, at, 24, DIRT, 130.0);
        clods(&mut self.juice.fx, at, 24, DIRT, 130.0);
    }
}

fn emerge_push(
    enemies: &mut [crate::enemy::Enemy],
    origin: Vec2,
    facing: Vec2,
    id: u32,
    radius: f32,
) {
    for e in enemies {
        if e.last_hit_by == id || e.is_boss() || !e.hittable() || e.flying() {
            continue;
        }
        let d = e.pos - origin;
        let reach = radius + e.radius();
        let dist = d.length();
        if dist >= reach {
            continue;
        }
        let k = 1.0 - (1.0 - t::EMERGE_PUSH_EDGE) * (dist / reach);
        e.last_hit_by = id;
        e.hurt(
            0,
            d.normalize_or(facing),
            t::EMERGE_PUSH_KNOCKBACK * k,
            t::EMERGE_PUSH_STUN,
        );
    }
}

/// 맞혔을 때의 손맛(0168): 멈춤·흔들림, 맞힌 쪽으로 카메라 킥(박자·딱 맞춤이면 조금 더), 강공격은 번쩍.
fn strike_feel(feel: &mut Feel, player: &Player, s: &Strike, dir: Vec2) {
    feel.add_hitstop(s.hitstop);
    feel.add_trauma(s.shake);
    let beat = if player.on_beat { t::KICK_BONUS } else { 0.0 };
    let kick = match s.kind {
        crate::combat::StrikeKind::Heavy => {
            t::KICK_HEAVY
                + if matches!(player.state, player::State::Heavy { perfect: true, .. }) {
                    t::KICK_BONUS
                } else {
                    0.0
                }
        }
        crate::combat::StrikeKind::Claw(_) if s.finisher => t::KICK_FINISHER + beat,
        crate::combat::StrikeKind::Claw(_) => t::KICK_LIGHT + beat,
        crate::combat::StrikeKind::Emerge => t::KICK_FINISHER,
        _ => 0.0,
    };
    if kick > 0.0 {
        feel.add_kick(dir, kick);
    }
    if s.kind == crate::combat::StrikeKind::Heavy {
        feel.flash = feel.flash.max(t::HEAVY_HIT_FLASH);
    }
}

/// 적이 쓰러졌다(0169 R1): 수·떨어뜨릴 자리·소리·고리·터짐·실 풀림·흔들림을 한 곳에서. 어떤 원인(공격·독·돌·덤불·벽 쿵·
/// 되돌린 투사체)이든 같은 손맛이다. 자기 필드들을 따로 받는 건 `self.enemies`를 도는 중에 부르기 때문(빌림이 겹치지 않게).
/// `ring_color`는 고리 색(보통 그 적의 색, 독이면 초록).
fn enemy_down(
    fx: &mut Vec<Fx>,
    audio: &mut sfx::Queue,
    feel: &mut Feel,
    kills: &mut u32,
    deaths: &mut Vec<(Vec2, Kind)>,
    e: &Enemy,
    ring_color: Color,
) {
    *kills += 1;
    deaths.push((e.pos, e.kind));
    audio.play(sfx::Id::EnemyDie);
    feel.add_trauma(t::ENEMY_DIE_SHAKE);
    fx.push(ring(e.pos, e.radius(), e.radius() * 3.0, 0.25, ring_color));
    burst(fx, e.pos, 12, enemy_color(e.kind), 110.0);
    unravel(fx, e.pos, e.radius());
}

pub(crate) fn ring(pos: Vec2, r0: f32, r1: f32, dur: f32, color: Color) -> Fx {
    Fx {
        pos,
        vel: Vec2::ZERO,
        time: 0.0,
        dur,
        kind: FxKind::Ring { r0, r1, color },
    }
}

/// 효과가 이만큼 넘으면 먼지 같은 꾸밈 효과는 건너뛴다(타격 효과는 그대로).
const FX_SOFT_CAP: usize = 300;
const STEP_DUST_EVERY: f32 = 12.0;
const CLOD_GRAVITY: f32 = 360.0;
/// 쓰러진 이웃에서 풀려 나오는 실의 보라(물든 털의 실 색).
pub const THREAD: Color = Color::new(0.68, 0.45, 0.92, 1.0);
/// 2단계로 깨어난 보스의 붉은 기운(0233): 깨어날 때 둘째 고리, 그 뒤 발밑 고리.
pub const ROUSE: Color = Color::new(0.95, 0.35, 0.45, 1.0);

/// 먼지·연기 `n`개: `spread` 반경 안 아무 데서, 천천히 퍼진다.
pub(crate) fn puff(fx: &mut Vec<Fx>, pos: Vec2, n: usize, color: Color, size: f32, dur: f32) {
    for _ in 0..n {
        let ang = rand::gen_range(0.0, std::f32::consts::TAU);
        let off = vec2(ang.cos(), ang.sin()) * rand::gen_range(0.0, size);
        fx.push(Fx {
            pos: pos + off,
            vel: off * 2.0 + vec2(0.0, -6.0),
            time: 0.0,
            dur: dur * rand::gen_range(0.8, 1.2),
            kind: FxKind::Puff {
                r0: size * 0.5,
                r1: size * 1.3,
                color,
            },
        });
    }
}

/// 흙덩이 `n`개: 사방으로 튀어 올랐다 떨어진다.
pub(crate) fn clods(fx: &mut Vec<Fx>, pos: Vec2, n: usize, color: Color, speed: f32) {
    for _ in 0..n {
        let ang = rand::gen_range(0.0, std::f32::consts::TAU);
        let sp = rand::gen_range(0.4, 1.0) * speed;
        fx.push(Fx {
            pos,
            vel: vec2(ang.cos(), ang.sin() * 0.6) * sp,
            time: 0.0,
            dur: rand::gen_range(0.45, 0.7),
            kind: FxKind::Clod {
                color,
                z: 2.0,
                vz: rand::gen_range(70.0, 130.0),
            },
        });
    }
}

/// 맞은 쪽(`dir`)으로 퍼지는 불똥.
fn spray(fx: &mut Vec<Fx>, pos: Vec2, dir: Vec2, n: usize, color: Color, speed: f32) {
    let base = dir.y.atan2(dir.x);
    for _ in 0..n {
        let ang = base + rand::gen_range(-0.7, 0.7);
        let sp = rand::gen_range(0.5, 1.0) * speed;
        fx.push(Fx {
            pos,
            vel: vec2(ang.cos(), ang.sin()) * sp,
            time: 0.0,
            dur: rand::gen_range(0.1, 0.22),
            kind: FxKind::Spark { color },
        });
    }
}

fn glint(pos: Vec2, color: Color, dur: f32) -> Fx {
    Fx {
        pos,
        vel: Vec2::ZERO,
        time: 0.0,
        dur,
        kind: FxKind::Glint { color },
    }
}

/// 쓰러진 이웃: 연기 조금 + 보라 실이 풀려 위로 흩어진다(0081). 이웃이 실에서 풀려나는 그림.
fn unravel(fx: &mut Vec<Fx>, pos: Vec2, radius: f32) {
    puff(
        fx,
        pos,
        4,
        Color::new(0.45, 0.38, 0.50, 1.0),
        radius * 0.6,
        0.45,
    );
    for k in 0..5 {
        let ang = rand::gen_range(0.0, std::f32::consts::TAU);
        fx.push(Fx {
            pos: pos + vec2(ang.cos(), ang.sin()) * radius * 0.5,
            vel: vec2(ang.cos() * 18.0, -rand::gen_range(14.0, 30.0)),
            time: 0.0,
            dur: 0.7 + k as f32 * 0.06,
            kind: FxKind::Thread {
                angle: ang,
                len: rand::gen_range(4.0, 7.0),
            },
        });
    }
}

/// 꽃잎 흩날림(0159): 위로 튀었다가 팔랑이며 내려앉는다.
fn petals(fx: &mut Vec<Fx>, pos: Vec2, n: usize, color: Color, speed: f32) {
    for k in 0..n {
        let ang = rand::gen_range(0.0, std::f32::consts::TAU);
        let sp = rand::gen_range(speed * 0.4, speed);
        fx.push(Fx {
            pos: pos + vec2(ang.cos() * 4.0, ang.sin() * 3.0),
            vel: vec2(ang.cos() * sp * 0.6, -sp - rand::gen_range(0.0, 20.0)),
            time: -0.02 * k as f32,
            dur: rand::gen_range(1.6, 2.6),
            kind: FxKind::Petal {
                color,
                phase: rand::gen_range(0.0, 6.0),
            },
        });
    }
}

fn burst(fx: &mut Vec<Fx>, pos: Vec2, n: usize, color: Color, speed: f32) {
    for _ in 0..n {
        let ang = rand::gen_range(0.0, std::f32::consts::TAU);
        let sp = rand::gen_range(0.3, 1.0) * speed;
        fx.push(Fx {
            pos,
            vel: vec2(ang.cos(), ang.sin()) * sp,
            time: 0.0,
            dur: rand::gen_range(0.15, 0.35),
            kind: FxKind::Spark { color },
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemy::{Enemy, Kind};

    /// 토끼풀(0165): 발 구르기는 충격파 + 플레이어 둘레에 실 그물, 고치는 발톱이 튕기며 한마디, 실 한 가닥은 받아치면
    /// 쥬시 지렁이(0200): 들고 쓰러지면 그 틱에 먹고 일어난다. 없으면 쓰러진 그대로.
    #[test]
    fn juicy_worm_revives_on_death() {
        use crate::enemy::Projectile;
        let hit = |w: &mut World| {
            let at = w.player.pos;
            w.projectiles.push(Projectile {
                pos: at,
                prev_pos: at,
                vel: Vec2::ZERO,
                alive: true,
                friendly: false,
            });
        };
        let mut w = World::new();
        w.enemies.clear();
        w.player.hp = 1;
        w.player.invuln = 0.0;
        assert!(w.inv.bag.add(Consumable::Worm));
        hit(&mut w);
        w.tick(&Actions::default());
        // 바로 일어나지 않고 잠깐 눕는다(0243): 쓰러진 게 보이고, 그동안은 안 맞는다.
        assert!(
            matches!(w.player.state, player::State::Revive { .. }),
            "누워서 지렁이를 먹는다"
        );
        assert!(w.player.is_alive() && w.player.is_invulnerable());
        assert!(w.juice.audio.played(sfx::Id::Die));
        for _ in 0..((t::REVIVE_DOWN_TIME + t::REVIVE_HITSTOP + 0.2) * 60.0) as usize {
            w.tick(&Actions::default());
        }
        assert!(
            matches!(w.player.state, player::State::Free),
            "다 먹고 일어난다"
        );
        assert!(w.player.hp > 1 && w.player.invuln > 0.0);
        assert!(!w.inv.bag.take(Consumable::Worm), "지렁이는 먹었다");
        assert!(w.juice.toast.is_some());
        // 지렁이가 없으면 쓰러진다.
        let mut w = World::new();
        w.enemies.clear();
        w.player.hp = 1;
        w.player.invuln = 0.0;
        hit(&mut w);
        w.tick(&Actions::default());
        assert!(!w.player.is_alive());
    }

    /// 토끼풀이 멈칫하고 아니면 끌려간다.
    #[test]
    fn clover_thump_drops_webs_cocoons_bounce_and_thread_pulls_or_parries() {
        use crate::combat::{Shape, StrikeKind};
        let mut w = World::new();
        let at = w.player.pos + vec2(0.0, -60.0);
        w.thump_at(at);
        assert_eq!(w.hazards.waves.len(), 1);
        assert_eq!(w.room.webs.len(), t::CLOVER_WAVE_WEBS, "실 그물이 떨어진다");
        assert!(w.juice.audio.played(sfx::Id::Impact));
        // 고치: 튕기고 한마디.
        let c = w.player.pos + vec2(20.0, 0.0);
        w.hazards.cocoons.push(c);
        w.juice.toast = None;
        w.cocoon_hit(&Strike {
            id: 7,
            kind: StrikeKind::Claw(0),
            shape: Shape::Circle { center: c, r: 4.0 },
            damage: 1,
            knockback: 0.0,
            stun: 0.0,
            hitstop: 0.0,
            shake: 0.0,
            finisher: false,
            poison: 0.0,
            crush: false,
        });
        assert!(w.juice.toast.is_some(), "고치는 때릴 수 없다");
        assert!(w.juice.audio.played(sfx::Id::WallHit));
        // 실 한 가닥: 줄 위에 서 있으면 끌려간다.
        let mut e = Enemy::new(Kind::Clover, at);
        e.hp = 10;
        w.enemies.push(e);
        w.thread_at(at, vec2(0.0, 1.0));
        assert!(
            w.player.vel.y < 0.0,
            "토끼풀 쪽(위)으로 끌려간다 {:?}",
            w.player.vel
        );
        assert!(w.hazards.thread_lash.is_some());
        // 받아치면 토끼풀이 멈칫.
        w.player.vel = Vec2::ZERO;
        w.player.open_parry_for_test(vec2(0.0, -1.0));
        let i = w
            .enemies
            .iter()
            .position(|e| e.kind == Kind::Clover)
            .unwrap();
        let hp0 = w.enemies[i].hp;
        w.thread_at(at, vec2(0.0, 1.0));
        assert!(
            matches!(w.enemies[i].state, enemy::State::Stunned { .. }) || w.enemies[i].hp < hp0,
            "받아치기 {:?}",
            w.enemies[i].state
        );
        assert!(w.juice.audio.played(sfx::Id::Parry));
    }

    /// 흰 실 그물(0165): 그물 위를 걸으면 느리고(처음엔 한마디), 땅속은 안 걸리고, 발톱이 닿으면 끊긴다.
    #[test]
    fn silk_webs_slow_walking_and_are_cut_by_claws() {
        use crate::combat::{Shape, StrikeKind};
        let mut w = World::new();
        let at = w.player.pos + vec2(0.0, -8.0);
        w.room.webs.push(at);
        let walk = Actions {
            move_dir: vec2(0.0, -1.0),
            ..Default::default()
        };
        let room = Room::new();
        let y0 = w.player.pos.y;
        w.web_tick();
        assert!(w.player.slow, "그물 위에서 느리다");
        assert!(w.juice.toast.is_some(), "처음엔 한마디");
        w.player.update(&walk, &room, t::TICK_DT);
        let slowed = y0 - w.player.pos.y;
        // 땅속은 안 걸린다.
        w.player.state = crate::player::State::Burrow { time: 0.0 };
        w.web_tick();
        assert!(!w.player.slow, "땅속은 지나간다");
        w.player.state = crate::player::State::Free;
        // 발톱이 닿으면 끊긴다.
        w.web_cut(&Strike {
            id: 1,
            kind: StrikeKind::Claw(0),
            shape: Shape::Circle { center: at, r: 4.0 },
            damage: 1,
            knockback: 0.0,
            stun: 0.0,
            hitstop: 0.0,
            shake: 0.0,
            finisher: false,
            poison: 0.0,
            crush: false,
        });
        assert!(w.room.webs.is_empty(), "발톱이 그물을 끊는다");
        assert!(w.juice.audio.played(sfx::Id::Unbind));
        w.web_tick();
        let y1 = w.player.pos.y;
        w.player.update(&walk, &room, t::TICK_DT);
        let free = y1 - w.player.pos.y;
        assert!(!w.player.slow);
        assert!(
            free > slowed * 1.5,
            "끊기면 다시 빨라진다 {free} vs {slowed}"
        );
    }

    /// 할매 2단계(0233): 매듭이 하나 남으면 당기기 뒤에 곧바로 한 번 더, 그다음은 보통 사이.
    #[test]
    fn deep_last_knot_pulls_twice() {
        let mut w = World::new();
        let c = w.player.pos + vec2(0.0, -80.0);
        w.player.pos = c + vec2(0.0, t::DEEP_PULL_REACH + 60.0); // 실이 닿지 않는 곳(감기지 않게)
        w.deep = Some(Deep::new(c));
        let ends = |w: &mut World| {
            let d = w.deep.as_mut().unwrap();
            d.next_pull = 0.0;
            w.deep_tick(t::TICK_DT);
            while w.deep.as_ref().unwrap().pull.is_some() {
                w.deep_tick(t::TICK_DT);
            }
            w.deep.as_ref().unwrap().next_pull
        };
        assert!(ends(&mut w) > t::DEEP_ECHO_GAP, "매듭 셋: 보통 사이");
        for k in w.deep.as_mut().unwrap().knots.iter_mut().take(2) {
            k.cut = true;
        }
        assert_eq!(
            ends(&mut w),
            t::DEEP_ECHO_GAP,
            "하나 남으면 곧바로 한 번 더"
        );
        assert!(ends(&mut w) > t::DEEP_ECHO_GAP, "그다음은 보통 사이");
    }

    /// 가장 깊은 곳(0162): 매듭은 평타 여섯(강공격은 두 배)에 끊기고, 당기기에 세 번 감기면 실에 잠긴다.
    /// 감기면 가장 가까운 매듭이 다시 조여진다. 셋 다 끊으면 잠시 뒤 엔딩.
    #[test]
    fn deep_knots_cut_and_three_wraps_put_you_to_sleep() {
        use crate::adventure::AdvEvent;
        use crate::combat::{Shape, StrikeKind};
        let mut w = World::new();
        let c = w.player.pos + vec2(0.0, -80.0);
        w.deep = Some(Deep::new(c));
        let strike = |id: u32, kind: StrikeKind, at: Vec2, damage: i32| Strike {
            id,
            kind,
            shape: Shape::Circle { center: at, r: 4.0 },
            damage,
            knockback: 0.0,
            stun: 0.0,
            hitstop: 0.0,
            shake: 0.0,
            finisher: false,
            poison: 0.0,
            crush: false,
        };
        let k1 = w.deep.as_ref().unwrap().knots[1].pos;
        let s0 = strike(1, StrikeKind::Claw(0), k1, 1);
        w.deep_hit(&s0);
        w.deep_hit(&s0);
        assert_eq!(
            w.deep.as_ref().unwrap().knots[1].hp,
            t::DEEP_KNOT_HP - 1,
            "같은 공격 번호는 한 번만"
        );
        for id in 2..=t::DEEP_KNOT_HP as u32 {
            w.deep_hit(&strike(id, StrikeKind::Claw(0), k1, 1));
        }
        assert!(w.deep.as_ref().unwrap().knots[1].cut, "평타 여섯에 끊긴다");
        let k0 = w.deep.as_ref().unwrap().knots[0].pos;
        w.deep_hit(&strike(50, StrikeKind::Heavy, k0, 2));
        assert_eq!(
            w.deep.as_ref().unwrap().knots[0].hp,
            t::DEEP_KNOT_HP - 2 * t::DEEP_HEAVY_MUL,
            "강공격은 두 배"
        );
        // 당기기: 뻗는 틱에 줄 위에 서 있으면 감긴다. 감긴 동안 가장 가까운 매듭(0)이 다시 조여진다.
        for n in 1..=t::DEEP_WRAPS {
            // 매듭 0 쪽에 서서(1은 끊겼고 2는 반대편) 줄에 걸린다.
            w.player.pos = c + (k0 - c).normalize() * 80.0;
            let d = w.deep.as_mut().unwrap();
            let dir = (w.player.pos - c).normalize();
            d.pull = Some(Pull {
                dir,
                len: t::DEEP_PULL_REACH,
                time: t::DEEP_PULL_WINDUP - 0.001,
                judged: false,
                snapped: false,
            });
            w.deep_tick(t::TICK_DT);
            assert_eq!(w.deep.as_ref().unwrap().wraps, n, "감겼다");
            if n == 1 {
                assert_eq!(
                    w.deep.as_ref().unwrap().knots[0].hp,
                    t::DEEP_KNOT_HP,
                    "매듭이 다시 조여진다"
                );
            }
            for _ in 0..((t::DEEP_WRAP_TIME / t::TICK_DT) as usize + 2) {
                w.deep_tick(t::TICK_DT);
            }
        }
        assert!(w.deep.as_ref().unwrap().asleep, "세 번이면 실에 잠긴다");
        assert!(w.adv.events.contains(&AdvEvent::DeepSleep));
        // 셋 다 끊으면 잠시 뒤 엔딩.
        let mut w = World::new();
        w.deep = Some(Deep::new(c));
        for (i, k) in w.deep.clone().unwrap().knots.iter().enumerate() {
            for id in 0..t::DEEP_KNOT_HP as u32 {
                w.deep_hit(&strike(
                    100 + i as u32 * 10 + id,
                    StrikeKind::Claw(0),
                    k.pos,
                    1,
                ));
            }
        }
        assert!(w.deep.as_ref().unwrap().all_cut());
        for _ in 0..((t::DEEP_DONE_DELAY / t::TICK_DT) as usize + 2) {
            w.deep_tick(t::TICK_DT);
        }
        assert!(w.adv.events.contains(&AdvEvent::DeepDone));
        assert!(
            w.juice.audio.count(sfx::Id::Unbind) >= 3,
            "놓아주기: 매듭 셋 + 할매의 실이 풀리는 소리가 더 난다(0163) {:?}",
            w.juice.audio
        );
        assert!(
            w.juice
                .fx
                .iter()
                .any(|f| matches!(f.kind, FxKind::Petal { .. })),
            "흰 꽃잎"
        );
    }

    /// 길의 샘(0159): 들어서는 게 아니라 발을 담가야 가득 회복, 한 번만.
    #[test]
    fn spring_heals_once_when_stepped_in() {
        let mut w = World::new();
        let c = w.player.pos;
        w.spring = Some(Spring {
            pos: c + vec2(60.0, 0.0),
            drunk: false,
        });
        w.player.hp = 1;
        assert!(!w.spring_tick(), "멀리서는 아무 일 없다");
        assert_eq!(w.player.hp, 1);
        w.player.pos = c + vec2(60.0 + t::SPRING_RADIUS - 1.0, 0.0);
        assert!(w.spring_tick(), "발을 담그면 회복");
        assert_eq!(w.player.hp, w.player.stats.max_hp);
        assert_eq!(w.stats.springs, 1);
        assert!(w.spring.unwrap().drunk);
        w.player.hp = 1;
        assert!(!w.spring_tick(), "한 번만");
        assert_eq!(w.player.hp, 1);
    }

    /// 입자 효과 설정(0141): 새 장식 효과는 비율만큼 고르게 남고, 공격 범위 고리는 늘 남는다. 한 번 거른 것은 다시 거르지 않는다.
    #[test]
    fn effects_setting_thins_decorative_fx_but_keeps_rings() {
        let dust = |x: f32| Fx {
            pos: vec2(x, 0.0),
            vel: Vec2::ZERO,
            time: 0.0,
            dur: 1.0,
            kind: FxKind::Puff {
                r0: 1.0,
                r1: 2.0,
                color: WHITE,
            },
        };
        let ring = Fx {
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
            time: 0.0,
            dur: 1.0,
            kind: FxKind::Ring {
                r0: 1.0,
                r1: 20.0,
                color: WHITE,
            },
        };
        let mut w = World::new();
        w.juice.effects = 0.5;
        w.juice.fx.extend((0..10).map(|i| dust(i as f32)));
        w.juice.fx.push(ring);
        w.thin_fx();
        assert_eq!(w.juice.fx.len(), 6, "먼지 열 중 다섯 + 고리");
        assert!(w
            .juice
            .fx
            .iter()
            .any(|f| matches!(f.kind, FxKind::Ring { .. })));
        // 이미 거른 것은 그대로, 새로 붙은 것만 거른다.
        w.juice.fx.extend((0..4).map(|i| dust(i as f32)));
        w.thin_fx();
        assert_eq!(w.juice.fx.len(), 8);
        // 0이면 장식은 하나도 남지 않는다.
        w.juice.effects = 0.0;
        w.juice.fx.extend((0..5).map(|i| dust(i as f32)));
        w.thin_fx();
        assert_eq!(w.juice.fx.len(), 8);
    }

    /// 혀 줄 맞기(0146): 선분까지 거리.
    #[test]
    fn distance_to_tongue_segment() {
        let (a, b) = (vec2(0.0, 0.0), vec2(10.0, 0.0));
        assert_eq!(dist_to_segment(vec2(5.0, 3.0), a, b), 3.0);
        assert_eq!(dist_to_segment(vec2(-4.0, 0.0), a, b), 4.0);
        assert_eq!(dist_to_segment(vec2(13.0, 4.0), a, b), 5.0);
    }

    /// 나는 박쥐는 솟구치기에 밀리지 않는다(0146).
    #[test]
    fn emerge_push_skips_flying_bats() {
        let o = vec2(100.0, 100.0);
        let mut es = vec![Enemy::new(Kind::Bat, o + vec2(20.0, 0.0))];
        emerge_push(&mut es, o, vec2(1.0, 0.0), 3, t::EMERGE_PUSH_RADIUS);
        assert_eq!(es[0].vel, Vec2::ZERO);
    }

    /// 밀쳐내기(0090): 둘레의 잡몹은 피해 없이 바깥으로 밀리고 멈춘다. 가까울수록 세다. 이미 맞은 적·보스·멀리 있는 적은 그대로.
    #[test]
    fn emerge_pushes_nearby_mobs_outward_without_damage() {
        let o = vec2(100.0, 100.0);
        let mut es = vec![
            Enemy::new(Kind::FieldMouse, o + vec2(30.0, 0.0)),
            Enemy::new(Kind::FieldMouse, o + vec2(0.0, -44.0)),
            Enemy::new(Kind::FieldMouse, o + vec2(90.0, 0.0)),
            Enemy::new(Kind::Chief, o + vec2(-30.0, 0.0)),
            Enemy::new(Kind::FieldMouse, o + vec2(10.0, 0.0)),
        ];
        es[4].last_hit_by = 7; // 솟구치기에 이미 맞은 적
        let hp: Vec<i32> = es.iter().map(|e| e.hp).collect();
        emerge_push(&mut es, o, vec2(1.0, 0.0), 7, t::EMERGE_PUSH_RADIUS);
        assert!(
            es[0].vel.x > 0.0 && es[0].vel.y.abs() < 1e-3,
            "오른쪽 적은 오른쪽으로"
        );
        assert!(es[1].vel.y < 0.0, "위쪽 적은 위로");
        assert!(es[0].vel.length() > es[1].vel.length(), "가까울수록 세게");
        assert_eq!(es[2].vel, Vec2::ZERO, "반지름 밖은 그대로");
        assert_eq!(es[3].vel, Vec2::ZERO, "보스는 버틴다");
        assert_eq!(es[4].vel, Vec2::ZERO, "이미 맞은 적은 또 밀지 않는다");
        for (e, h) in es.iter().zip(hp) {
            assert_eq!(e.hp, h, "밀쳐내기는 피해가 없다");
        }
        assert!(matches!(es[0].state, crate::enemy::State::Stunned { .. }));
    }

    /// 부술 수 있는 것(0175): 공격이 닿으면 빠지고 막던 자리가 풀리며 소리·흙덩이, 닿지 않으면 그대로. 꿈에서는 빈손.
    #[test]
    fn breakables_break_on_a_strike_and_stop_blocking() {
        use crate::combat::{Shape, StrikeKind};
        use crate::room::{Breakable, Prop};
        let mut w = World::new();
        let c = w.player.pos + vec2(24.0, 0.0);
        let rect = Rect::new(c.x - 8.0, c.y - 8.0, 16.0, 16.0);
        w.room.add_prop(Prop {
            rect,
            kind: PropKind::Breakable(Breakable::RockPile),
        });
        assert!(w.room.solids.contains(&rect), "소품은 막는다");
        let strike = |at: Vec2| Strike {
            id: 9,
            kind: StrikeKind::Claw(0),
            shape: Shape::Circle { center: at, r: 4.0 },
            damage: 1,
            knockback: 0.0,
            stun: 0.0,
            hitstop: 0.0,
            shake: 0.0,
            finisher: false,
            poison: 0.0,
            crush: false,
        };
        let brk = |w: &World| {
            w.room
                .props
                .iter()
                .filter(|p| matches!(p.kind, PropKind::Breakable(_)))
                .count()
        };
        w.prop_break(&strike(c + vec2(40.0, 0.0)));
        assert_eq!(brk(&w), 1, "안 닿으면 그대로");
        let fx_before = w.juice.fx.len();
        w.prop_break(&strike(c + vec2(6.0, 0.0)));
        assert_eq!(brk(&w), 0, "깨졌다");
        assert!(!w.room.solids.contains(&rect), "막던 자리가 풀린다");
        assert!(w.juice.audio.played(sfx::Id::Break));
        assert!(w.juice.fx.len() > fx_before, "흙덩이·먼지");
        assert_eq!(w.stats.broken, 1);
        // 꿈: 빈손. 여러 번 깨도 떨어지는 게 없다(첫 번째 것이 운으로 떨어뜨린 건 치운다).
        w.loot.clear();
        w.dream = true;
        for _ in 0..30 {
            w.room.add_prop(Prop {
                rect,
                kind: PropKind::Breakable(Breakable::Mound),
            });
            w.prop_break(&strike(c));
        }
        assert!(w.loot.is_empty(), "꿈에서는 아무것도 안 나온다");
        // 깨어서는 운에 따라 나온다(서른 번이면 거의 확실히 하나는).
        w.dream = false;
        for _ in 0..30 {
            w.room.add_prop(Prop {
                rect,
                kind: PropKind::Breakable(Breakable::Mound),
            });
            w.prop_break(&strike(c));
        }
        assert!(!w.loot.is_empty(), "운에 따라 소모품");
    }
}

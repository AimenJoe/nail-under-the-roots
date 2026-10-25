//! 런을 넘어 남는 기록(프로필, docs/decisions/0041). 다회차의 바닥(0040).
//!
//! - 내용은 깃발(있다/없다)과 수(센 것) 두 가지뿐이다. 이름은 안정된 문자열 키라 칸을 늘려도 옛 저장을 그대로 읽는다.
//!   모르는 키도 버리지 않고 다시 쓴다(새 버전이 쓴 것을 옛 버전이 지우지 않게).
//! - 형식: 첫 줄 `nail-profile 1`, 그 뒤 한 줄에 하나 `flag <키>` / `count <키> <수>`. 사람이 읽을 수 있다.
//! - 저장소: 웹은 localStorage 한 칸(web/index.html의 nail_save 플러그인, wasm 메모리로 바이트를 주고받는다),
//!   데스크톱은 사용자 데이터 폴더의 파일, Android는 앱 전용 폴더의 파일.

use std::collections::{BTreeMap, BTreeSet};

const HEADER: &str = "nail-profile 1";

/// 기록 키. 바꾸면 옛 저장과 끊기니 이름은 바꾸지 않는다.
pub mod key {
    /// 튜토리얼을 마쳤다 → 다음 런부터 오프닝·튜토리얼을 건너뛴다.
    pub const TUTORIAL: &str = "tutorial";
    /// 들쥐 대장을 이겨 보리가 돌아왔다 → 공터는 열려 있고 대장은 없다.
    pub const BORI_BACK: &str = "bori";
    /// 새 런을 시작한 수.
    pub const RUNS: &str = "runs";
    /// 쓰러진 수(이야기 조건·도움 모드).
    pub const DEATHS: &str = "deaths";
    /// 도움 모드 "포포의 응원"이 켜져 있다(0040).
    pub const ASSIST: &str = "assist";
    /// 도움 모드를 켠 채 쓰러진 수(받는 피해가 이만큼 더 준다).
    pub const ASSIST_DEATHS: &str = "assist_deaths";
    /// 가장 멀리 간 쉼터 번호.
    pub const DEEPEST: &str = "deepest";
    /// 가장 깊은 곳에 닿아 반쪽 엔딩을 봤다 / 참 엔딩을 봤다(0042).
    pub const HALF_ENDING: &str = "half_ending";
    pub const TRUE_ENDING: &str = "true_ending";
    /// 참 엔딩 조건(0037): 모모가 돌아옴, 도토가 씨앗 자리를 기억해 냄. 아직 세우는 곳이 없다(모모·도토 보스가 생기면).
    pub const MOMO_BACK: &str = "momo";
    pub const DOTO_MEMORY: &str = "doto_memory";

    /// 실뜨기: 고를 수 있는 가장 높은 단계(캐릭터별), 마지막으로 고른 단계, 새 단계가 열렸다는 표시(0043).
    pub fn knots_open(character: impl std::fmt::Debug) -> String {
        format!("knots.{character:?}")
    }
    pub const KNOT_PICK: &str = "knot_pick";
    pub const KNOT_NEW: &str = "knot_new";

    /// 캐릭터별 클리어 수.
    pub fn clears(character: impl std::fmt::Debug) -> String {
        format!("clears.{character:?}")
    }

    pub fn mob(kind: impl std::fmt::Debug) -> String {
        format!("mob.{kind:?}")
    }
    pub fn illust(file: &str) -> String {
        format!("illust.{file}")
    }
    pub fn line(id: &str) -> String {
        format!("line.{id}")
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Profile {
    flags: BTreeSet<String>,
    counts: BTreeMap<String, u32>,
}

/// 키에 쓸 수 있는 글자: 공백·줄바꿈이 없으면 된다(한 줄 한 칸).
fn valid_key(k: &str) -> bool {
    !k.is_empty() && k.chars().all(|c| c.is_ascii_graphic())
}

impl Profile {
    pub fn flag(&self, k: &str) -> bool {
        self.flags.contains(k)
    }

    /// 깃발을 세운다. 처음 세웠으면 true(저장할 일이 생겼다).
    pub fn set(&mut self, k: &str) -> bool {
        debug_assert!(valid_key(k));
        self.flags.insert(k.to_string())
    }

    pub fn unset(&mut self, k: &str) -> bool {
        self.flags.remove(k)
    }

    pub fn count(&self, k: &str) -> u32 {
        self.counts.get(k).copied().unwrap_or(0)
    }

    pub fn add(&mut self, k: &str, n: u32) {
        debug_assert!(valid_key(k));
        let v = self.counts.entry(k.to_string()).or_insert(0);
        *v = v.saturating_add(n);
    }

    /// 수를 그대로 정한다(마지막으로 고른 것 등).
    pub fn set_count(&mut self, k: &str, n: u32) {
        debug_assert!(valid_key(k));
        self.counts.insert(k.to_string(), n);
    }

    /// 더 크면 바꾼다(최고 기록). 바뀌었으면 true.
    pub fn raise(&mut self, k: &str, n: u32) -> bool {
        if n > self.count(k) {
            self.counts.insert(k.to_string(), n);
            true
        } else {
            false
        }
    }

    pub fn encode(&self) -> String {
        let mut s = String::from(HEADER);
        s.push('\n');
        for f in &self.flags {
            s.push_str("flag ");
            s.push_str(f);
            s.push('\n');
        }
        for (k, v) in &self.counts {
            s.push_str(&format!("count {k} {v}\n"));
        }
        s
    }

    /// 머리줄이 다르면(남의 데이터, 다른 형식) None. 줄 하나가 깨졌으면 그 줄만 버린다.
    pub fn decode(s: &str) -> Option<Profile> {
        let mut lines = s.lines();
        if lines.next()?.trim() != HEADER {
            return None;
        }
        let mut p = Profile::default();
        for l in lines {
            let mut it = l.split_whitespace();
            match (it.next(), it.next(), it.next(), it.next()) {
                (Some("flag"), Some(k), None, None) if valid_key(k) => {
                    p.flags.insert(k.to_string());
                }
                (Some("count"), Some(k), Some(v), None) if valid_key(k) => {
                    if let Ok(v) = v.parse() {
                        p.counts.insert(k.to_string(), v);
                    }
                }
                _ => {}
            }
        }
        Some(p)
    }
}

// ---- 저장소 ----------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
mod store {
    extern "C" {
        fn nail_save_len() -> u32;
        fn nail_save_read(ptr: *mut u8, len: u32);
        fn nail_save_write(ptr: *const u8, len: u32);
    }

    pub fn read() -> Option<String> {
        // SAFETY: JS가 저장된 글의 UTF-8 바이트 수를 돌려주고(없으면 0, 플러그인이 없어도 0),
        // 우리가 잡은 그만큼의 버퍼에 그대로 써 준다(web/index.html nail_save).
        let n = unsafe { nail_save_len() } as usize;
        if n == 0 {
            return None;
        }
        let mut buf = vec![0u8; n];
        unsafe { nail_save_read(buf.as_mut_ptr(), n as u32) };
        String::from_utf8(buf).ok()
    }

    pub fn write(s: &str) {
        // SAFETY: JS는 이 호출 동안만 그 범위를 읽는다.
        unsafe { nail_save_write(s.as_ptr(), s.len() as u32) }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod store {
    use std::path::PathBuf;

    /// 저장 파일 자리. `NAIL_SAVE_DIR`이 있으면 그곳(테스트·휴대용).
    fn path() -> Option<PathBuf> {
        let dir = if let Some(d) = std::env::var_os("NAIL_SAVE_DIR") {
            PathBuf::from(d)
        } else if cfg!(target_os = "android") {
            // 앱 전용 내부 폴더(다른 앱은 못 본다). 패키지 이름은 Cargo.toml [package.metadata.android].
            PathBuf::from("/data/data/io.github.aimenjoe.nail/files")
        } else if cfg!(target_os = "windows") {
            PathBuf::from(std::env::var_os("APPDATA")?).join("nail")
        } else if cfg!(target_os = "macos") {
            PathBuf::from(std::env::var_os("HOME")?).join("Library/Application Support/nail")
        } else {
            match std::env::var_os("XDG_DATA_HOME") {
                Some(d) => PathBuf::from(d).join("nail"),
                None => PathBuf::from(std::env::var_os("HOME")?).join(".local/share/nail"),
            }
        };
        Some(dir.join("profile.txt"))
    }

    pub fn read() -> Option<String> {
        std::fs::read_to_string(path()?).ok()
    }

    /// 임시 파일에 쓰고 이름을 바꾼다(쓰다 꺼져도 옛 저장이 남는다).
    pub fn write(s: &str) {
        let Some(p) = path() else { return };
        if let Some(d) = p.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        let tmp = p.with_extension("tmp");
        if std::fs::write(&tmp, s).is_ok() {
            let _ = std::fs::rename(&tmp, &p);
        }
    }
}

/// miniquad가 플러그인 버전을 맞춰 보는 함수(mq_js_bundle.js의 init_plugins).
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn nail_save_crate_version() -> u32 {
    1
}

/// 저장된 기록. 없거나 읽을 수 없으면 빈 기록.
pub fn load() -> Profile {
    store::read()
        .and_then(|s| Profile::decode(&s))
        .unwrap_or_default()
}

pub fn save(p: &Profile) {
    store::write(&p.encode());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_keeps_flags_and_counts() {
        let mut p = Profile::default();
        assert!(p.set(key::TUTORIAL));
        assert!(!p.set(key::TUTORIAL), "두 번째는 새로 세운 게 아니다");
        p.add(key::DEATHS, 3);
        p.add(key::DEATHS, 2);
        p.set(&key::illust("great_tree"));
        assert!(p.raise(key::DEEPEST, 2));
        assert!(!p.raise(key::DEEPEST, 1));
        let q = Profile::decode(&p.encode()).unwrap();
        assert_eq!(p, q);
        assert_eq!(q.count(key::DEATHS), 5);
        assert!(q.flag(&key::illust("great_tree")));
    }

    #[test]
    fn rejects_foreign_data_and_skips_broken_lines() {
        assert_eq!(Profile::decode(""), None);
        assert_eq!(Profile::decode("hello\nflag x\n"), None);
        let p = Profile::decode(
            "nail-profile 1\nflag ok\ncount n notanumber\nwhat is this\ncount m 7\n",
        )
        .unwrap();
        assert!(p.flag("ok"));
        assert_eq!(p.count("n"), 0);
        assert_eq!(p.count("m"), 7);
    }

    #[test]
    fn unknown_keys_survive_a_round_trip() {
        // 새 버전이 쓴 키를 옛 버전이 읽고 다시 써도 남는다.
        let s = "nail-profile 1\nflag future.thing\ncount future.n 9\n";
        let p = Profile::decode(s).unwrap();
        let q = Profile::decode(&p.encode()).unwrap();
        assert!(q.flag("future.thing"));
        assert_eq!(q.count("future.n"), 9);
    }

    #[test]
    fn file_store_writes_and_reads_back() {
        let dir = std::env::temp_dir().join(format!("nail-save-test-{}", std::process::id()));
        std::env::set_var("NAIL_SAVE_DIR", &dir);
        let mut p = Profile::default();
        p.set(key::BORI_BACK);
        p.add(key::RUNS, 4);
        save(&p);
        assert_eq!(load(), p);
        let _ = std::fs::remove_dir_all(&dir);
        std::env::remove_var("NAIL_SAVE_DIR");
    }
}

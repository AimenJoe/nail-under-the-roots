//! Epic Games Store 연결(0276 E2). `--features epic` + Windows일 때만 EOS SDK를 부르고, 아니면 모두 빈 손이다.
//! Epic 런처로 켜지 않았거나(로그인 코드 없음) EOS 아이디가 빌드에 없으면 조용히 그냥 켜진다 — 게임은 Epic에 기대지 않는다.
//!
//! 흐름(EOS 문서 "Log In with an Epic Games Account", 2026-11-04 확인):
//! `EOS_Initialize` → `EOS_Platform_Create`(제품·샌드박스·배포·클라이언트 아이디) → 런처가 넘긴 교환 코드(`-AUTH_PASSWORD=`)로
//! `EOS_Auth_Login`(ExchangeCode) → `EOS_Auth_CopyIdToken` → `EOS_Connect_Login`(EPIC_ID_TOKEN) — 처음이면 `EOS_InvalidUser`라
//! `EOS_Connect_CreateUser`(ContinuanceToken) → 얻은 ProductUserId로 `EOS_Achievements_UnlockAchievements`.
//! 콜백은 모두 `EOS_Platform_Tick` 안에서(이 스레드에서) 불린다. 매 프레임 `tick`.
//!
//! 개발 중에는 EOS Developer Authentication Tool로: 환경 변수 `NAIL_EOS_DEV=<host:port>/<자격 이름>`이면 Developer 로그인.
//! EOS 아이디는 빌드 때 환경 변수(`NAIL_EOS_PRODUCT_ID`·`_SANDBOX_ID`·`_DEPLOYMENT_ID`·`_CLIENT_ID`·`_CLIENT_SECRET`)로 넣는다 —
//! 공개 레포에 적지 않는다(Epic 업로드 CI가 Environment `store`에서 넣는다).
//! 업적 표는 `achievements.rs` — Steam과 같은 API 이름이 EOS 업적 아이디다.

use crate::profile::Profile;

pub struct Epic {
    #[cfg(all(feature = "epic", windows))]
    eos: Option<imp::Eos>,
    /// 이번 실행에서 이미 연 업적.
    opened: Vec<&'static str>,
    /// 로그인된 뒤 옛 기록으로 이룬 업적을 한꺼번에 열었나.
    caught_up: bool,
}

impl Epic {
    pub fn start() -> Self {
        Self {
            #[cfg(all(feature = "epic", windows))]
            eos: imp::Eos::start(),
            opened: Vec::new(),
            caught_up: false,
        }
    }

    /// 매 프레임: EOS를 돌리고(로그인 콜백이 여기서 온다), 로그인이 막 끝났으면 옛 기록으로 이룬 업적도 연다.
    pub fn tick(&mut self, p: &Profile) {
        #[cfg(all(feature = "epic", windows))]
        if let Some(e) = &mut self.eos {
            e.tick();
        }
        if !self.caught_up && self.ready() {
            self.caught_up = true;
            self.sync(p);
        }
    }

    /// 기록이 바뀐 뒤: 새로 이룬 업적을 연다. 로그인 전이면 아무것도 표시하지 않고 기다린다(로그인되면 `tick`이 따라잡는다).
    pub fn sync(&mut self, p: &Profile) {
        if !self.ready() {
            return;
        }
        let fresh: Vec<&'static str> = crate::achievements::fresh(p, &self.opened)
            .map(|x| x.api)
            .collect();
        if fresh.is_empty() {
            return;
        }
        #[cfg(all(feature = "epic", windows))]
        if let Some(e) = &mut self.eos {
            e.unlock(&fresh);
        }
        self.opened.extend(fresh);
    }

    fn ready(&self) -> bool {
        #[cfg(all(feature = "epic", windows))]
        {
            self.eos.as_ref().is_some_and(|e| e.logged_in())
        }
        #[cfg(not(all(feature = "epic", windows)))]
        {
            false
        }
    }
}

#[cfg(all(feature = "epic", windows))]
mod imp {
    use eos_sys::*;
    use std::ffi::{c_void, CString};
    use std::ptr;

    // eos-sys의 바인딩은 eos_achievements.h의 함수를 싣지 않는다(타입만) — 이 하나만 직접 선언한다(EOS C SDK 서명 그대로).
    extern "C" {
        fn EOS_Achievements_UnlockAchievements(
            Handle: EOS_HAchievements,
            Options: *const EOS_Achievements_UnlockAchievementsOptions,
            ClientData: *mut c_void,
            CompletionDelegate: EOS_Achievements_OnUnlockAchievementsCompleteCallback,
        );
    }

    /// 콜백이 고쳐 쓰는 상태 — 상자에 담아 주소를 고정하고 그 주소를 ClientData로 넘긴다.
    struct State {
        auth: EOS_HAuth,
        connect: EOS_HConnect,
        user: EOS_ProductUserId,
        failed: bool,
    }

    pub struct Eos {
        platform: EOS_HPlatform,
        ach: EOS_HAchievements,
        state: Box<State>,
    }

    fn id(name: Option<&'static str>) -> Option<CString> {
        name.filter(|s| !s.is_empty())
            .and_then(|s| CString::new(s).ok())
    }

    /// 런처가 넘기는 인자에서 값 하나(`-AUTH_PASSWORD=…` 꼴).
    fn arg(key: &str) -> Option<String> {
        let pre = format!("-{key}=");
        std::env::args().find_map(|a| a.strip_prefix(&pre).map(str::to_string))
    }

    impl Eos {
        pub fn start() -> Option<Self> {
            let (Some(product), Some(sandbox), Some(deployment), Some(client), Some(secret)) = (
                id(option_env!("NAIL_EOS_PRODUCT_ID")),
                id(option_env!("NAIL_EOS_SANDBOX_ID")),
                id(option_env!("NAIL_EOS_DEPLOYMENT_ID")),
                id(option_env!("NAIL_EOS_CLIENT_ID")),
                id(option_env!("NAIL_EOS_CLIENT_SECRET")),
            ) else {
                eprintln!("epic: 빌드에 EOS 아이디가 없다 — 연결 없이 켠다");
                return None;
            };
            let name = CString::new("Nail Under the Roots").ok()?;
            let version = CString::new(env!("CARGO_PKG_VERSION")).ok()?;
            unsafe {
                let init = EOS_InitializeOptions {
                    ApiVersion: EOS_INITIALIZE_API_LATEST as i32,
                    ProductName: name.as_ptr(),
                    ProductVersion: version.as_ptr(),
                    ..Default::default()
                };
                let r = EOS_Initialize(&init);
                if r != EOS_EResult_EOS_Success && r != EOS_EResult_EOS_AlreadyConfigured {
                    eprintln!("epic: EOS_Initialize 실패 ({r}) — 연결 없이 켠다");
                    return None;
                }
                let opts = EOS_Platform_Options {
                    ApiVersion: EOS_PLATFORM_OPTIONS_API_LATEST as i32,
                    ProductId: product.as_ptr(),
                    SandboxId: sandbox.as_ptr(),
                    DeploymentId: deployment.as_ptr(),
                    ClientCredentials: EOS_Platform_ClientCredentials {
                        ClientId: client.as_ptr(),
                        ClientSecret: secret.as_ptr(),
                    },
                    ..Default::default()
                };
                let platform = EOS_Platform_Create(&opts);
                if platform.is_null() {
                    eprintln!("epic: EOS_Platform_Create 실패 — 연결 없이 켠다");
                    return None;
                }
                let eos = Self {
                    platform,
                    ach: EOS_Platform_GetAchievementsInterface(platform),
                    state: Box::new(State {
                        auth: EOS_Platform_GetAuthInterface(platform),
                        connect: EOS_Platform_GetConnectInterface(platform),
                        user: ptr::null_mut(),
                        failed: false,
                    }),
                };
                eos.login();
                Some(eos)
            }
        }

        /// Epic 런처의 교환 코드로, 없으면 개발용 Developer 로그인으로. 둘 다 없으면 로그인하지 않는다(업적 없이 그냥 논다).
        fn login(&self) {
            let (kind, idv, token) = if let Some(code) = arg("AUTH_PASSWORD") {
                (EOS_ELoginCredentialType_EOS_LCT_ExchangeCode, None, code)
            } else if let Some((host, name)) = std::env::var("NAIL_EOS_DEV").ok().and_then(|v| {
                v.split_once('/')
                    .map(|(h, n)| (h.to_string(), n.to_string()))
            }) {
                (EOS_ELoginCredentialType_EOS_LCT_Developer, Some(host), name)
            } else {
                eprintln!("epic: 런처의 로그인 코드가 없다 — 업적 없이 켠다");
                return;
            };
            let (Ok(token), idc) = (CString::new(token), idv.and_then(|h| CString::new(h).ok()))
            else {
                return;
            };
            unsafe {
                let cred = EOS_Auth_Credentials {
                    ApiVersion: EOS_AUTH_CREDENTIALS_API_LATEST as i32,
                    Id: idc.as_ref().map_or(ptr::null(), |c| c.as_ptr()),
                    Token: token.as_ptr(),
                    Type: kind,
                    ..Default::default()
                };
                let opts = EOS_Auth_LoginOptions {
                    ApiVersion: EOS_AUTH_LOGIN_API_LATEST as i32,
                    Credentials: &cred,
                    ScopeFlags: EOS_EAuthScopeFlags_EOS_AS_BasicProfile,
                    LoginFlags: 0,
                };
                let data = &*self.state as *const State as *mut c_void;
                EOS_Auth_Login(self.state.auth, &opts, data, Some(on_auth_login));
            }
        }

        pub fn tick(&mut self) {
            unsafe { EOS_Platform_Tick(self.platform) }
        }

        pub fn logged_in(&self) -> bool {
            !self.state.user.is_null() && !self.state.failed
        }

        pub fn unlock(&mut self, apis: &[&'static str]) {
            let names: Vec<CString> = apis.iter().filter_map(|a| CString::new(*a).ok()).collect();
            let mut ptrs: Vec<*const std::os::raw::c_char> =
                names.iter().map(|c| c.as_ptr()).collect();
            unsafe {
                let opts = EOS_Achievements_UnlockAchievementsOptions {
                    ApiVersion: EOS_ACHIEVEMENTS_UNLOCKACHIEVEMENTS_API_LATEST as i32,
                    UserId: self.state.user,
                    AchievementIds: ptrs.as_mut_ptr(),
                    AchievementsCount: ptrs.len() as u32,
                };
                EOS_Achievements_UnlockAchievements(
                    self.ach,
                    &opts,
                    ptr::null_mut(),
                    Some(on_unlocked),
                );
            }
        }
    }

    impl Drop for Eos {
        fn drop(&mut self) {
            unsafe {
                EOS_Platform_Release(self.platform);
                EOS_Shutdown();
            }
        }
    }

    fn done(r: EOS_EResult) -> bool {
        unsafe { EOS_EResult_IsOperationComplete(r) != 0 }
    }

    unsafe extern "C" fn on_auth_login(data: *const EOS_Auth_LoginCallbackInfo) {
        let info = &*data;
        let st = &mut *(info.ClientData as *mut State);
        if info.ResultCode != EOS_EResult_EOS_Success {
            if done(info.ResultCode) {
                eprintln!("epic: Epic 계정 로그인 실패 ({})", info.ResultCode);
                st.failed = true;
            }
            return;
        }
        let copy = EOS_Auth_CopyIdTokenOptions {
            ApiVersion: EOS_AUTH_COPYIDTOKEN_API_LATEST as i32,
            AccountId: info.LocalUserId,
        };
        let mut token: *mut EOS_Auth_IdToken = ptr::null_mut();
        if EOS_Auth_CopyIdToken(st.auth, &copy, &mut token) != EOS_EResult_EOS_Success
            || token.is_null()
        {
            eprintln!("epic: ID 토큰을 못 받았다");
            st.failed = true;
            return;
        }
        let cred = EOS_Connect_Credentials {
            ApiVersion: EOS_CONNECT_CREDENTIALS_API_LATEST as i32,
            Token: (*token).JsonWebToken,
            Type: EOS_EExternalCredentialType_EOS_ECT_EPIC_ID_TOKEN,
        };
        let opts = EOS_Connect_LoginOptions {
            ApiVersion: EOS_CONNECT_LOGIN_API_LATEST as i32,
            Credentials: &cred,
            UserLoginInfo: ptr::null(),
        };
        EOS_Connect_Login(st.connect, &opts, info.ClientData, Some(on_connect_login));
        EOS_Auth_IdToken_Release(token);
    }

    unsafe extern "C" fn on_connect_login(data: *const EOS_Connect_LoginCallbackInfo) {
        let info = &*data;
        let st = &mut *(info.ClientData as *mut State);
        if info.ResultCode == EOS_EResult_EOS_Success {
            st.user = info.LocalUserId;
        } else if info.ResultCode == EOS_EResult_EOS_InvalidUser {
            // 이 게임에 처음 — ProductUserId를 만든다.
            let opts = EOS_Connect_CreateUserOptions {
                ApiVersion: EOS_CONNECT_CREATEUSER_API_LATEST as i32,
                ContinuanceToken: info.ContinuanceToken,
            };
            EOS_Connect_CreateUser(st.connect, &opts, info.ClientData, Some(on_create_user));
        } else if done(info.ResultCode) {
            eprintln!("epic: EOS Connect 로그인 실패 ({})", info.ResultCode);
            st.failed = true;
        }
    }

    unsafe extern "C" fn on_create_user(data: *const EOS_Connect_CreateUserCallbackInfo) {
        let info = &*data;
        let st = &mut *(info.ClientData as *mut State);
        if info.ResultCode == EOS_EResult_EOS_Success {
            st.user = info.LocalUserId;
        } else if done(info.ResultCode) {
            eprintln!("epic: EOS 사용자 만들기 실패 ({})", info.ResultCode);
            st.failed = true;
        }
    }

    unsafe extern "C" fn on_unlocked(
        data: *const EOS_Achievements_OnUnlockAchievementsCompleteCallbackInfo,
    ) {
        let info = &*data;
        if info.ResultCode != EOS_EResult_EOS_Success {
            eprintln!("epic: 업적 열기 실패 ({})", info.ResultCode);
        }
    }
}

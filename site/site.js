// 공식 페이지(0069, 0155): GitHub Releases에서 최신 버전·APK 주소·소식을 읽어 채운다.
// 반딧불(.flies[data-flies])을 뿌리고, 깊이 눈금(.depth)에서 지금 층을 밝히고 나일을 스크롤만큼 내린다.
// 읽지 못하면(오프라인·요청 제한) 페이지에 적어 둔 값과 릴리스 목록 링크가 그대로 남는다.
(() => {
  const REPO = "AimenJoe/nail-under-the-roots";
  const lang = document.documentElement.lang || "ko";
  const fmt = (iso) => (iso || "").slice(0, 10);
  // 릴리스 본문 첫 줄 "# Naïl: Under the Roots v0.1.3 — 멈칫 없이 들어가기"에서 부제만.
  const subtitle = (body) => {
    const first = (body || "").split("\n").find((l) => l.startsWith("# ")) || "";
    const i = first.indexOf("—");
    return i >= 0 ? first.slice(i + 1).trim() : "";
  };
  fetch(`https://api.github.com/repos/${REPO}/releases?per_page=8`, { headers: { Accept: "application/vnd.github+json" } })
    .then((r) => (r.ok ? r.json() : Promise.reject(r.status)))
    .then((list) => {
      // 개발판(미리 보기 릴리스 `dev`, 0193)은 건너뛴다. 넉넉히 읽고 넷만.
      const rel = list.filter((r) => !r.draft && !r.prerelease).slice(0, 4);
      if (!rel.length) return;
      const latest = rel[0];
      const apk = (latest.assets || []).find((a) => a.name.endsWith(".apk"));
      document.querySelectorAll("[data-latest-version]").forEach((el) => { el.textContent = latest.tag_name; });
      document.querySelectorAll("[data-latest-date]").forEach((el) => { el.textContent = fmt(latest.published_at); });
      if (apk) document.querySelectorAll("[data-apk]").forEach((el) => { el.href = apk.browser_download_url; });
      // PC 묶음(0240): nail-<windows|macos|linux>-release.(zip|tar.gz).
      document.querySelectorAll("[data-pc]").forEach((el) => {
        const pc = (latest.assets || []).find((a) => a.name.startsWith(`nail-${el.dataset.pc}-`));
        if (pc) el.href = pc.browser_download_url;
      });
      const news = document.querySelector("[data-news]");
      if (news) {
        news.textContent = "";
        for (const r of rel) {
          const li = document.createElement("li");
          const v = document.createElement("span"); v.className = "v"; v.textContent = r.tag_name;
          const d = document.createElement("span"); d.className = "d"; d.textContent = fmt(r.published_at);
          const a = document.createElement("a"); a.href = r.html_url;
          // 영어 페이지는 본문 맨 아래 "English (short)"가 있으면 그 한 줄을 쓴다.
          let text = subtitle(r.body);
          if (lang === "en") {
            const m = (r.body || "").match(/\*\*English \(short\)\*\*:\s*(.+)/);
            if (m) text = m[1].split(". ")[0].replace(/\.$/, "");
          }
          a.textContent = text || r.name || r.tag_name;
          li.append(v, d, a);
          news.append(li);
        }
      }
    })
    .catch(() => {});
})();

(() => {
  // 반딧불: 자리·흔들림·깜빡임을 하나씩 다르게. 움직임 줄이기면 CSS가 멈춘다.
  const rnd = (a, b) => a + Math.random() * (b - a);
  document.querySelectorAll("[data-flies]").forEach((box) => {
    const n = Number(box.dataset.flies) || 8;
    for (let i = 0; i < n; i++) {
      const f = document.createElement("span");
      f.className = "fly";
      const st = f.style;
      st.left = `${rnd(4, 96)}%`;
      st.top = `${rnd(8, 88)}%`;
      st.setProperty("--x1", `${rnd(-40, 40)}px`); st.setProperty("--y1", `${rnd(-30, 30)}px`);
      st.setProperty("--x2", `${rnd(-40, 40)}px`); st.setProperty("--y2", `${rnd(-30, 30)}px`);
      st.setProperty("--d", `${rnd(7, 13)}s`); st.setProperty("--b", `${rnd(2.2, 4.5)}s`);
      st.setProperty("--delay", `${rnd(-8, 0)}s`);
      box.append(f);
    }
  });

  // 깊이 눈금: 페이지를 내려간 만큼 나일이 내려가고, 화면 가운데 있는 층의 이름이 밝아진다.
  const depth = document.querySelector(".depth");
  if (!depth) return;
  const mole = depth.querySelector(".depth-mole");
  const marks = [...depth.querySelectorAll("li")];
  const sections = [...document.querySelectorAll("[data-depth]")];
  let ticking = false;
  const update = () => {
    ticking = false;
    const max = document.documentElement.scrollHeight - innerHeight;
    const p = max > 0 ? Math.min(1, scrollY / max) : 0;
    mole.style.setProperty("--p", `${(p * 100).toFixed(1)}%`);
    const mid = innerHeight / 2;
    let cur = sections[0];
    for (const s of sections) if (s.getBoundingClientRect().top < mid) cur = s;
    const key = cur ? cur.dataset.depth : "top";
    marks.forEach((m) => m.classList.toggle("on", m.dataset.for === key));
  };
  addEventListener("scroll", () => { if (!ticking) { ticking = true; requestAnimationFrame(update); } }, { passive: true });
  addEventListener("resize", update);
  update();
})();

(() => {
  // 게임 클립: 화면 가까이 오면 그때 받아 틀고(첫 화면을 가볍게), 벗어나면 멈춘다.
  // 움직임 줄이기면 틀지 않고 재생 단추만 보인다.
  const still = matchMedia("(prefers-reduced-motion: reduce)").matches;
  document.querySelectorAll("video[data-src]").forEach((v) => {
    const load = () => { if (!v.src) { v.src = v.dataset.src; } };
    if (still) { v.controls = true; v.preload = "none"; load(); return; }
    if (!("IntersectionObserver" in window)) { load(); v.play().catch(() => {}); return; }
    new IntersectionObserver((es) => {
      for (const e of es) {
        if (e.isIntersecting) { load(); v.play().catch(() => { v.controls = true; }); }
        else if (v.src) v.pause();
      }
    }, { rootMargin: "200px 0px" }).observe(v);
  });
})();

(() => {
  // 첫 화면 아래 걷는 띠(0228 덧붙임): 들를 때마다 이웃 몇이 무작위로 나와 건너가고, 하나가 나가면 다른 이가 들어온다.
  // 칸 크기(c)의 2배로 그린다. f = 한 칸 시간, d = 건너는 시간(도토는 달리고 모모는 깡충, 보리·수수는 느릿느릿).
  const box = document.querySelector("[data-parade]");
  const SPRITE_V = "c5eb5783";   // make_images.py가 sp_*.png 판의 해시를 적는다(캐시 깨기)
  if (!box || matchMedia("(prefers-reduced-motion: reduce)").matches) return;
  const CAST = [
    { k: "nail", n: 4, c: 28, f: 0.16, d: 26 },
    { k: "popo", n: 4, c: 32, f: 0.16, d: 28 },
    { k: "momo", n: 4, c: 32, f: 0.1, d: 17 },
    { k: "doto", n: 4, c: 40, f: 0.075, d: 10 },
    { k: "susu", n: 4, c: 32, f: 0.2, d: 34 },
    { k: "bori", n: 4, c: 40, f: 0.2, d: 36 },
    { k: "kongi", n: 4, c: 32, f: 0.16, d: 27 },
    { k: "clover", n: 4, c: 40, f: 0.16, d: 22 },
  ];
  const rnd = (a, b) => a + Math.random() * (b - a);
  const out = new Set();
  const send = (slot) => {
    const free = CAST.filter((x) => !out.has(x.k));
    const who = free[Math.floor(Math.random() * free.length)];
    out.add(who.k);
    const el = document.createElement("span");
    el.className = "sprite walker";
    const st = el.style;
    st.backgroundImage = `url(${box.dataset.parade}sp_${who.k}.png?v=${SPRITE_V})`;
    st.setProperty("--n", who.n);
    st.setProperty("--s", `${who.c * 2}px`);
    st.setProperty("--f", `${who.f}s`);
    const d = who.d * rnd(0.9, 1.15);
    st.setProperty("--d", `${d.toFixed(1)}s`);
    st.setProperty("--loop", "1");
    // 처음 몇은 이미 길 위에 띄엄띄엄 있다(음수 지연 = 중간부터, 길을 칸으로 나눠 한 칸에 하나), 그다음은 잠깐 쉬었다 들어온다.
    const start = slot === null ? rnd(0.5, 5) : -d * 0.9 * ((slot + rnd(0.2, 0.8)) / count);
    st.setProperty("--delay", `${start.toFixed(1)}s`);
    el.addEventListener("animationend", (e) => {
      if (e.animationName !== "walk") return;
      el.remove();
      out.delete(who.k);
      send(null);
    });
    box.append(el);
  };
  box.textContent = "";
  const count = innerWidth < 600 ? 2 : 3;
  for (let i = 0; i < count; i++) send(i);
})();


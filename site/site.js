// 공식 페이지(0069): GitHub Releases에서 최신 버전·APK 주소·소식을 읽어 채운다.
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
  fetch(`https://api.github.com/repos/${REPO}/releases?per_page=4`, { headers: { Accept: "application/vnd.github+json" } })
    .then((r) => (r.ok ? r.json() : Promise.reject(r.status)))
    .then((list) => {
      const rel = list.filter((r) => !r.draft && !r.prerelease);
      if (!rel.length) return;
      const latest = rel[0];
      const apk = (latest.assets || []).find((a) => a.name.endsWith(".apk"));
      document.querySelectorAll("[data-latest-version]").forEach((el) => { el.textContent = latest.tag_name; });
      document.querySelectorAll("[data-latest-date]").forEach((el) => { el.textContent = fmt(latest.published_at); });
      if (apk) document.querySelectorAll("[data-apk]").forEach((el) => { el.href = apk.browser_download_url; });
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

// SPDX-License-Identifier: GPL-2.0-or-later
// Chinese text has no space-delimited words: use a small local substring index.
(() => {
  if (!document.documentElement.lang.startsWith("zh")) return;
  const nav = document.querySelector(".razers-language-nav");
  if (!nav) return;
  const form = document.createElement("form");
  form.className = "razers-search";
  form.setAttribute("role", "search");
  const label = document.createElement("label");
  label.htmlFor = "razers-search";
  label.textContent = "搜索中文手册";
  const input = document.createElement("input");
  input.id = "razers-search";
  input.type = "search";
  input.placeholder = "输入关键词，例如：语言、设备、DPI";
  input.maxLength = 160;
  const status = document.createElement("p");
  status.setAttribute("role", "status");
  const results = document.createElement("ul");
  results.id = "razers-search-results";
  input.setAttribute("aria-controls", results.id);
  form.append(label, input, status, results);
  nav.after(form);
  let index;
  let request = 0;
  async function search() {
    const sequence = ++request;
    const query = input.value.trim().toLocaleLowerCase("zh-CN");
    results.replaceChildren();
    status.textContent = "";
    if (!query) return;
    status.textContent = "正在搜索…";
    try {
      index ??= fetch("search-data.json").then(response => {
        if (!response.ok) throw new Error("search index unavailable");
        return response.json();
      });
      const pages = await index;
      if (sequence !== request) return;
      const words = query.split(/\s+/u);
      const matches = pages.filter(page => words.every(word =>
        `${page.title} ${page.text}`.toLocaleLowerCase("zh-CN").includes(word)));
      status.textContent = matches.length ? `找到 ${matches.length} 个相关章节` : "没有找到相关章节，请尝试其他关键词。";
      for (const page of matches) {
        const item = document.createElement("li");
        const link = document.createElement("a");
        link.href = page.path;
        link.textContent = page.title;
        const excerpt = document.createElement("p");
        const start = Math.max(0, page.text.toLocaleLowerCase("zh-CN").indexOf(words[0]) - 35);
        excerpt.textContent = `${start ? "…" : ""}${page.text.slice(start, start + 140)}…`;
        item.append(link, excerpt);
        results.append(item);
      }
    } catch {
      index = undefined;
      if (sequence === request) status.textContent = "搜索索引加载失败，请刷新重试或使用左侧目录。";
    }
  }
  input.addEventListener("input", search);
  form.addEventListener("submit", event => { event.preventDefault(); search(); });
  document.addEventListener("keydown", event => {
    if (event.ctrlKey || event.metaKey || event.altKey || /INPUT|TEXTAREA|SELECT/.test(event.target.tagName) || event.target.isContentEditable) return;
    if (event.key === "/" || event.key.toLowerCase() === "s") {
      event.preventDefault();
      input.focus();
    }
  });
})();

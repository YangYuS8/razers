// SPDX-License-Identifier: GPL-2.0-or-later
// Each book has identical chapter paths; preserve the chapter, not translated anchors.
(() => {
  const zh = document.documentElement.lang.toLowerCase().startsWith("zh");
  const nav = document.createElement("nav");
  nav.className = "razers-language-nav";
  nav.setAttribute("aria-label", zh ? "文档导航与语言" : "Documentation and language");
  const page = location.pathname.split("/").pop() || "index.html";
  const links = [
    [zh ? "English" : "简体中文", `../${zh ? "en" : "zh-CN"}/${page}`, zh ? "en" : "zh-CN"],
    [zh ? "Rust API 参考" : "Rust API reference", "../api/", null],
  ];
  for (const [label, href, lang] of links) {
    const link = document.createElement("a");
    link.textContent = label;
    link.href = href;
    if (lang) { link.hreflang = lang; link.lang = lang; }
    nav.appendChild(link);
  }
  document.querySelector("main")?.prepend(nav);
  if (zh) {
    // Clipboard labels are presentation-only. Do not translate mdBook's
    // hidden-line button title: upstream also uses that title as state.
    const translateClipboard = () => {
      for (const button of document.querySelectorAll(".clip-button")) {
        if (button.title === "Copy to clipboard") {
          button.title = "复制代码";
          button.setAttribute("aria-label", "复制代码");
        }
        const tip = button.firstElementChild;
        if (tip?.textContent === "Copied!") tip.textContent = "已复制";
        if (tip?.textContent === "Clipboard error!") tip.textContent = "复制失败";
      }
    };
    translateClipboard();
    new MutationObserver(translateClipboard).observe(document.querySelector("main"), {subtree: true, childList: true, characterData: true});
  }
})();

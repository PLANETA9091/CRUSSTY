/* w3schools-style guide helpers: copy buttons on code blocks + tabs */
(function () {
  "use strict";

  if (window.__wsJsLoaded) return;
  window.__wsJsLoaded = true;

  function addCopyButtons() {
    document.querySelectorAll("pre").forEach(function (pre) {
      if (pre.closest(".ws-code")) return;
      var box = document.createElement("div");
      box.className = "ws-code";
      var btn = document.createElement("button");
      btn.className = "copy-btn";
      btn.type = "button";
      btn.textContent = "Copy";
      btn.addEventListener("click", function () {
        copyText(pre.innerText, btn);
      });
      pre.parentNode.insertBefore(box, pre);
      box.appendChild(pre);
      box.appendChild(btn);
    });
  }

  function copyText(text, btn) {
    function done() {
      btn.textContent = "Copied!";
      btn.classList.add("copied");
      setTimeout(function () {
        btn.textContent = "Copy";
        btn.classList.remove("copied");
      }, 1500);
    }
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(text).then(done, function () { fallback(text, done); });
    } else {
      fallback(text, done);
    }
  }

  function fallback(text, done) {
    var ta = document.createElement("textarea");
    ta.value = text;
    ta.style.position = "fixed";
    ta.style.opacity = "0";
    document.body.appendChild(ta);
    ta.select();
    try { document.execCommand("copy"); } catch (e) { /* ignore */ }
    document.body.removeChild(ta);
    done();
  }

  function initTabs() {
    document.querySelectorAll(".ws-tabs").forEach(function (tabs) {
      var bar = tabs.querySelector(".ws-tabbar");
      if (!bar) return;
      bar.querySelectorAll("button").forEach(function (btn) {
        btn.addEventListener("click", function () {
          bar.querySelectorAll("button").forEach(function (b) { b.classList.remove("active"); });
          tabs.querySelectorAll(".ws-tab").forEach(function (t) { t.classList.remove("active"); });
          btn.classList.add("active");
          var panel = tabs.querySelector('#' + btn.dataset.tab);
          if (panel) panel.classList.add("active");
        });
      });
    });
  }

  function initAnimations() {
    var reduce = window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (reduce) return;
    document.documentElement.classList.add("ws-anim");
    var els = document.querySelectorAll(".ws-code, .ws-try, .ws-step");
    if ("IntersectionObserver" in window) {
      var io = new IntersectionObserver(function (entries) {
        entries.forEach(function (entry) {
          if (entry.isIntersecting) {
            entry.target.classList.add("ws-in");
            io.unobserve(entry.target);
          }
        });
      }, { threshold: 0.05 });
      els.forEach(function (el) { io.observe(el); });
    } else {
      els.forEach(function (el) { el.classList.add("ws-in"); });
    }
  }

  function initSoftNav() {
    var lastNav = { ts: 0 };

    function activateNav(pathname) {
      document.querySelectorAll(".site-nav .nav-list-item").forEach(function (li) {
        li.classList.remove("active");
        var a = li.querySelector(":scope > .nav-list-link");
        if (a) a.classList.remove("active");
      });
      var best = null, bestLen = -1;
      document.querySelectorAll(".site-nav .nav-list-link").forEach(function (a) {
        var href = a.getAttribute("href");
        if (!href || href.charAt(0) === "#") return;
        var abs = new URL(href, location.origin).pathname;
        if (abs === pathname || pathname.indexOf(abs) === 0) {
          if (abs.length > bestLen) { bestLen = abs.length; best = a; }
        }
      });
      if (!best) return;
      var el = best;
      while (el && el !== document && !(el.classList && el.classList.contains("site-nav"))) {
        if (el.classList && el.classList.contains("nav-list-item")) {
          el.classList.add("active");
          var a = el.querySelector(":scope > .nav-list-link");
          if (a) a.classList.add("active");
          var exp = el.querySelector(":scope > .nav-list-expander");
          if (exp) exp.setAttribute("aria-expanded", "true");
        }
        el = el.parentElement;
      }
    }

    function applyPage(html, url, addHistory) {
      var doc = new DOMParser().parseFromString(html, "text/html");
      var newMain = doc.querySelector(".main-content");
      var main = document.querySelector(".main-content");
      if (!newMain || !main) { location.href = url; return; }
      main.innerHTML = newMain.innerHTML;
      var newNav = doc.querySelector(".site-nav");
      var nav = document.querySelector(".site-nav");
      if (newNav && nav && newNav.innerHTML !== nav.innerHTML) {
        nav.innerHTML = newNav.innerHTML;
      }
      document.title = doc.title || document.title;
      var icon = doc.querySelector("style#active-nav-icon");
      var oldIcon = document.querySelector("style#active-nav-icon");
      if (oldIcon) oldIcon.remove();
      if (icon) document.head.appendChild(icon);
      var path = new URL(url, location.href).pathname;
      if (addHistory) window.history.pushState(null, "", url);
      activateNav(path);
      window.scrollTo(0, 0);
      addCopyButtons();
      initTabs();
      initAnimations();
    }

    document.addEventListener("click", function (e) {
      if (e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
      var a = e.target.closest ? e.target.closest("a") : null;
      if (!a || !a.href) return;
      if (a.target && a.target !== "_self") return;
      var href = a.getAttribute("href");
      if (!href || href.charAt(0) === "#") return;
      if (a.hasAttribute("download")) return;
      if (href.indexOf("mailto:") === 0) return;
      var url;
      try { url = new URL(a.href); } catch (err) { return; }
      if (url.origin !== location.origin) return;
      var path = url.pathname;
      if (path === location.pathname) {
        if (url.hash) {
          var t = document.querySelector(url.hash);
          if (t) t.scrollIntoView();
        }
        return;
      }
      if (Date.now() - lastNav.ts < 1200) return;
      lastNav.ts = Date.now();
      e.preventDefault();
      fetch(a.href, { headers: { Accept: "text/html" } })
        .then(function (r) { return r.text(); })
        .then(function (html) { applyPage(html, a.href, true); })
        .catch(function () { location.href = a.href; });
    });

    window.addEventListener("popstate", function () {
      fetch(location.href, { headers: { Accept: "text/html" } })
        .then(function (r) { return r.text(); })
        .then(function (html) { applyPage(html, location.href, false); })
        .catch(function () { location.reload(); });
    });
  }

  function ready(fn) {
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", fn);
    } else {
      fn();
    }
  }

  ready(function () {
    addCopyButtons();
    initTabs();
    initAnimations();
    initSoftNav();
  });

})();

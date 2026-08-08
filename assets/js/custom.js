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
  });

})();

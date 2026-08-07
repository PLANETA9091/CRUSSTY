/* ===========================================================================
 * Crussty enhancements on top of mdBook's default book.js.
 *
 * 1. Smooth navigation: internal links are fetched and swapped in place with
 *    the View Transitions API when available — no full page reload, no white
 *    flash (pattern: pjax / Turbo Drive).
 * 2. Copy buttons for every code block (with event delegation, so they keep
 *    working after in-place navigation).
 * 3. Boot sequence on the front page — played once per browser session.
 * ======================================================================== */

(function () {
    'use strict';

    /* --- Helpers -------------------------------------------------------- */
    function pagePath(href) {
        try {
            const u = new URL(href, location.href);
            if (u.origin !== location.origin) return null;
            let p = u.pathname;
            if (p.endsWith('/')) p += 'index.html';
            return p;
        } catch (err) {
            return null;
        }
    }

    function currentPath() {
        return pagePath(location.href);
    }

    /* --- Copy buttons (delegated, survive pjax) -------------------------- */
    function ensureCopyButtons() {
        document.querySelectorAll('pre code').forEach(function (code) {
            const pre = code.parentElement;
            if (!pre || pre.classList.contains('playground') || pre.querySelector('.crussty-copy-btn')) return;
            const btn = document.createElement('button');
            btn.type = 'button';
            btn.className = 'crussty-copy-btn';
            btn.title = 'Copy to clipboard';
            btn.setAttribute('aria-label', 'Copy to clipboard');
            btn.innerHTML = '<i class="tooltiptext"></i>';
            btn.appendChild(copyIcon());
            let buttons = pre.querySelector('.buttons');
            if (!buttons) {
                buttons = document.createElement('div');
                buttons.className = 'buttons';
                pre.insertBefore(buttons, pre.firstChild);
            }
            buttons.insertBefore(btn, buttons.firstChild);
        });
    }

    function copyIcon() {
        const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
        svg.setAttribute('viewBox', '0 0 16 16');
        svg.setAttribute('fill', 'currentColor');
        svg.innerHTML = '<path d="M5.5 3.5a.5.5 0 0 0-.5.5v8a.5.5 0 0 0 .5.5h6a.5.5 0 0 0 .5-.5V7.707L9.293 4.5H5.5Z"/><path d="M3 1h6.586a1 1 0 0 1 .707.293l3 3A1 1 0 0 1 13.7 5H8.5A1.5 1.5 0 0 1 7 3.5V1Z"/><path d="M5 1.5A1.5 1.5 0 0 1 6.5 0h.5v3.5H3.5v-.5A1.5 1.5 0 0 1 5 1.5Z"/>';
        return svg;
    }

    document.addEventListener('click', function (e) {
        const btn = e.target.closest('.crussty-copy-btn');
        if (!btn) return;
        const pre = btn.closest('pre');
        if (!pre) return;
        const code = pre.querySelector('code');
        if (!code) return;
        const text = code.innerText;
        const done = function () {
            btn.classList.add('copied');
            setTimeout(function () { btn.classList.remove('copied'); }, 1200);
        };
        if (navigator.clipboard && navigator.clipboard.writeText) {
            navigator.clipboard.writeText(text).then(done).catch(function () {});
        } else {
            const ta = document.createElement('textarea');
            ta.value = text;
            document.body.appendChild(ta);
            ta.select();
            try { document.execCommand('copy'); done(); } catch (err) { /* ignore */ }
            document.body.removeChild(ta);
        }
    });

    /* --- Code highlighting after in-place navigation --------------------- */
    function highlightNewBlocks(container) {
        if (!window.hljs) return;
        container.querySelectorAll('pre code').forEach(function (code) {
            if (!code.classList.contains('hljs')) {
                window.hljs.highlightBlock(code);
            }
        });
    }

    /* --- Active sidebar item --------------------------------------------- */
    function setActiveLink() {
        const target = currentPath();
        document.querySelectorAll('#mdbook-sidebar a').forEach(function (a) {
            a.classList.toggle('active', pagePath(a.href) === target);
        });
    }

    /* --- Apply a fetched page in place ------------------------------------ */
    function applyPage(doc, path) {
        const newContent = doc.querySelector('#mdbook-content');
        const newWide = doc.querySelector('.nav-wide-wrapper');
        const cur = document.querySelector('#mdbook-content');
        if (!newContent || !cur) return false;

        cur.innerHTML = newContent.innerHTML;
        const wide = document.querySelector('.nav-wide-wrapper');
        if (wide && newWide) wide.innerHTML = newWide.innerHTML;

        const title = doc.querySelector('title');
        if (title) document.title = title.textContent;

        const edit = doc.querySelector('a[rel="edit"]');
        const curEdit = document.querySelector('a[rel="edit"]');
        if (edit && curEdit) curEdit.href = edit.href;

        setActiveLink();
        ensureCopyButtons();
        highlightNewBlocks(cur);

        if (path !== currentPath()) {
            history.pushState({ path: path }, '', path);
        }
        window.scrollTo(0, 0);
        return true;
    }

    async function navigate(path, push) {
        try {
            const resp = await fetch(path, { headers: { Accept: 'text/html' } });
            if (!resp.ok) throw new Error('fetch failed: ' + resp.status);
            const html = await resp.text();
            const doc = new DOMParser().parseFromString(html, 'text/html');
            const swap = function () { return applyPage(doc, push ? path : null); };
            if (document.startViewTransition) {
                document.startViewTransition(swap);
            } else {
                swap();
            }
        } catch (err) {
            location.href = path;
        }
    }

    document.addEventListener('click', function (e) {
        if (e.defaultPrevented || e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
        const a = e.target.closest('a[href]');
        if (!a) return;
        if (a.target && a.target !== '_self') return;
        const path = pagePath(a.href);
        if (!path) return;
        e.preventDefault();
        if (path === currentPath()) return;
        navigate(path, true);
    });

    window.addEventListener('popstate', function () {
        const path = currentPath();
        if (path) navigate(path, false);
    });

    /* --- Boot sequence (once per session) --------------------------------- */
    function boot() {
        if (sessionStorage.getItem('crussty-boot') === '1') return;
        const bootEl = document.getElementById('boot');
        if (!bootEl) return;

        sessionStorage.setItem('crussty-boot', '1');

        const LINES = [
            { t: "crussty-runtime v2.0.0 (native, JVMTI)", c: "dim" },
            { t: "  options: modules=./modules;versions=./versions;kernel=purpur-1.21.10.jar", c: "dim" },
            { t: "scanning modules/ ...", c: "info" },
            { t: "  [ 1/3 ] hello    -> manifest ok, entry libhello.so", c: "ok" },
            { t: "  [ 2/3 ] dist     -> manifest ok, entry libdist.so", c: "ok" },
            { t: "  [ 3/3 ] crussty  -> manifest ok, entry libcrussty.so", c: "ok" },
            { t: "topological order: hello -> dist -> crussty", c: "dim" },
            { t: "dlopen RTLD_LOCAL ...", c: "info" },
            { t: "  plugin hello    -> cplugin_init rc=0", c: "ok" },
            { t: "  plugin dist     -> cplugin_init rc=0", c: "ok" },
            { t: "  plugin crussty  -> cplugin_init rc=0", c: "ok" },
            { t: "register class-file hook pipeline ...", c: "info" },
            { t: "  CLASS_FILE_LOAD_HOOK enabled (CAN_RETRANSFORM_CLASSES)", c: "ok" },
            { t: "spawning kernel JVM ...", c: "info" },
            { t: "  purpur-1.21.10.jar (Java 21+)", c: "dim" },
            { t: "welcome to crussty — press any key to enter", c: "warn" }
        ];

        const overlay = document.createElement('div');
        overlay.id = 'crussty-boot';
        const pre = document.createElement('pre');
        overlay.appendChild(pre);
        document.body.appendChild(overlay);

        const TYPE_MS = 11;
        const LINE_PAUSE = 35;
        const END_PAUSE = 450;

        let i = 0;
        function nextLine() {
            if (i >= LINES.length) {
                const cursor = document.createElement('span');
                cursor.className = 'cursor';
                pre.appendChild(cursor);
                setTimeout(function () {
                    overlay.remove();
                    document.getElementById('boot')?.classList.add('active');
                    document.querySelector('#mdbook-content main')?.classList.add('boot-reveal');
                }, END_PAUSE);
                return;
            }
            const line = LINES[i];
            const div = document.createElement('div');
            pre.appendChild(div);
            let pos = 0;
            (function typeChar() {
                if (pos < line.t.length) {
                    div.textContent = line.t.slice(0, pos + 1);
                    pos++;
                    setTimeout(typeChar, TYPE_MS);
                } else {
                    if (line.c) div.className = line.c;
                    i++;
                    setTimeout(nextLine, LINE_PAUSE);
                }
            })();
        }
        nextLine();
    }

    /* --- Init ------------------------------------------------------------- */
    function init() {
        ensureCopyButtons();
        setActiveLink();
        boot();
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();

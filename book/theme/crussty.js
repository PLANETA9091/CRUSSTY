/* Crussty theme - progressive enhancements layered on top of mdBook's book.js.
 *
 * Loaded via `additional-js`, so it runs after book.js has set up the sidebar,
 * theme system, search and syntax highlighting. Adds: a branded header, a
 * one-click light/dark toggle, code block cards with copy buttons, a right-hand
 * "On this page" outline with scroll-spy, a breadcrumb, and the ASCII boot
 * sequence on the front page. Ported from Glide (MIT). */

(function () {
    'use strict';

    const DARK_THEMES = ['navy', 'coal', 'ayu'];
    const html = document.documentElement;

    function isDark() {
        return DARK_THEMES.some(t => html.classList.contains(t));
    }

    const ICONS = {
        sun: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="4"></circle><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"></path></svg>',
        moon: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path></svg>',
        copy: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>',
        check: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6 9 17l-5-5"></path></svg>',
        play: '<svg viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"></path></svg>',
    };

    /* --- Logo badge + version pill ---------------------------------------- */
    function setupBranding() {
        const name = document.querySelector('.glide-logo-name');
        const badge = document.querySelector('.glide-logo-badge');
        if (badge && name) {
            const letter = (name.textContent || '').trim().charAt(0).toUpperCase();
            badge.textContent = letter || 'C';
        }
        const img = document.querySelector('.glide-logo-img');
        if (img) {
            const reveal = () => {
                if (img.naturalWidth > 0) {
                    img.hidden = false;
                    if (badge) badge.hidden = true;
                } else {
                    img.hidden = true;
                }
            };
            if (img.complete) reveal();
            else {
                img.addEventListener('load', reveal);
                img.addEventListener('error', () => { img.hidden = true; });
            }
        }
        const version = document.querySelector('meta[name="glide-version"]');
        const pill = document.querySelector('.glide-version');
        if (pill && version && version.content) {
            pill.textContent = version.content;
            pill.hidden = false;
        }
    }

    /* --- One-click light / dark toggle ------------------------------------ */
    function setupThemeToggle() {
        const btn = document.getElementById('glide-theme-btn');
        const icon = btn && btn.querySelector('.glide-theme-icon');
        if (!btn || !icon) return;

        function render() {
            icon.innerHTML = isDark() ? ICONS.sun : ICONS.moon;
            btn.setAttribute('aria-pressed', String(isDark()));
        }
        render();

        btn.addEventListener('click', function () {
            const target = isDark() ? 'mdbook-theme-light' : 'mdbook-theme-navy';
            const themeButton = document.getElementById(target);
            if (themeButton) {
                themeButton.click();
            } else {
                const next = isDark() ? 'light' : 'navy';
                DARK_THEMES.concat('light').forEach(t => html.classList.remove(t));
                html.classList.add(next);
                try { localStorage.setItem('mdbook-theme', next); } catch (err) { /* ignore */ }
            }
            render();
        });

        new MutationObserver(render).observe(html, { attributes: true, attributeFilter: ['class'] });
    }

    /* --- Header search trigger -------------------------------------------- */
    function setupSearchTrigger() {
        const btn = document.getElementById('glide-search-btn');
        const toggle = document.getElementById('mdbook-search-toggle');
        if (!btn || !toggle) return;
        btn.addEventListener('click', function () {
            toggle.click();
            const input = document.getElementById('mdbook-searchbar');
            if (input) {
                setTimeout(() => input.focus(), 0);
            }
        });
    }

    /* --- Code block cards with a copy button ------------------------------ */
    function makeCopyButton(code) {
        const copyBtn = document.createElement('button');
        copyBtn.type = 'button';
        copyBtn.className = 'glide-copy-btn';
        copyBtn.innerHTML = ICONS.copy + '<span>Copy</span>';
        copyBtn.addEventListener('click', function () {
            const text = code.textContent;
            const done = () => {
                copyBtn.classList.add('copied');
                copyBtn.innerHTML = ICONS.check + '<span>Copied</span>';
                setTimeout(() => {
                    copyBtn.classList.remove('copied');
                    copyBtn.innerHTML = ICONS.copy + '<span>Copy</span>';
                }, 1300);
            };
            if (navigator.clipboard && navigator.clipboard.writeText) {
                navigator.clipboard.writeText(text).then(done).catch(() => {});
            } else {
                const ta = document.createElement('textarea');
                ta.value = text;
                document.body.appendChild(ta);
                ta.select();
                try { document.execCommand('copy'); done(); } catch (err) { /* ignore */ }
                document.body.removeChild(ta);
            }
        });
        return copyBtn;
    }

    function setupCodeBlocks() {
        const blocks = document.querySelectorAll('#mdbook-content main pre > code');
        blocks.forEach(function (code) {
            const pre = code.parentElement;
            if (!pre) return;
            if (pre.parentElement && pre.parentElement.classList.contains('glide-code')) return;

            const langClass = Array.from(code.classList).find(c => c.startsWith('language-'));
            const label = langClass ? langClass.replace('language-', '') : 'code';

            const card = document.createElement('div');
            card.className = 'glide-code';

            const header = document.createElement('div');
            header.className = 'glide-code-header';

            const name = document.createElement('span');
            name.className = 'glide-code-name';
            name.textContent = label;

            const actions = document.createElement('div');
            actions.className = 'glide-code-actions';

            const nativeButtons = pre.querySelector(':scope > .buttons');
            if (nativeButtons) {
                const play = nativeButtons.querySelector('.play-button');
                const hide = nativeButtons.querySelector('button[title*="hidden lines"]');

                if (play) {
                    const runBtn = document.createElement('button');
                    runBtn.type = 'button';
                    runBtn.className = 'glide-run-btn';
                    runBtn.title = 'Run this code';
                    runBtn.innerHTML = ICONS.play + '<span>Run</span>';
                    runBtn.addEventListener('click', () => play.click());
                    const syncRun = () => {
                        const shown = !play.hidden && !play.classList.contains('hidden');
                        runBtn.style.display = shown ? '' : 'none';
                    };
                    syncRun();
                    new MutationObserver(syncRun).observe(play, {
                        attributes: true, attributeFilter: ['hidden', 'class'],
                    });
                    actions.appendChild(runBtn);
                }

                if (hide) {
                    hide.classList.add('glide-hide-btn');
                    actions.appendChild(hide);
                }
            }

            actions.appendChild(makeCopyButton(code));

            header.appendChild(name);
            header.appendChild(actions);

            pre.parentNode.insertBefore(card, pre);
            card.appendChild(header);
            card.appendChild(pre);
        });
    }

    /* --- Right-hand "On this page" outline + scroll-spy ------------------- */
    function setupPageToc() {
        const aside = document.getElementById('glide-page-toc');
        const main = document.querySelector('#mdbook-content main');
        if (!aside || !main) return;

        const headings = Array.from(main.querySelectorAll('h2[id], h3[id]'));
        if (headings.length < 2) {
            aside.style.display = 'none';
            return;
        }

        const title = document.createElement('div');
        title.className = 'glide-toc-title';
        title.textContent = 'On this page';

        const list = document.createElement('ul');
        const links = [];
        headings.forEach(function (h) {
            const li = document.createElement('li');
            const a = document.createElement('a');
            a.href = '#' + h.id;
            a.textContent = h.textContent.replace(/[#¶]+$/, '').trim();
            if (h.tagName === 'H3') a.classList.add('glide-toc-h3');
            li.appendChild(a);
            list.appendChild(li);
            links.push(a);
        });

        aside.appendChild(title);
        aside.appendChild(list);

        let current = null;
        function setActive(id) {
            if (current === id) return;
            current = id;
            links.forEach(a => a.classList.toggle('active', a.getAttribute('href').slice(1) === id));
        }

        const observer = new IntersectionObserver(function (entries) {
            if (atBottom()) {
                setActive(headings[headings.length - 1].id);
                return;
            }
            const visible = entries
                .filter(e => e.isIntersecting)
                .sort((a, b) => a.boundingClientRect.top - b.boundingClientRect.top);
            if (visible.length) {
                setActive(visible[0].target.id);
            }
        }, { rootMargin: '-72px 0px -70% 0px', threshold: 0 });

        function atBottom() {
            const scrollEl = document.documentElement;
            return window.innerHeight + window.scrollY >= scrollEl.scrollHeight - 2;
        }

        function onScroll() {
            if (atBottom()) setActive(headings[headings.length - 1].id);
        }
        window.addEventListener('scroll', onScroll, { passive: true });

        headings.forEach(h => observer.observe(h));
        setActive(headings[0].id);
    }

    /* --- Breadcrumb ------------------------------------------------------- */
    function setupBreadcrumb() {
        const main = document.querySelector('#mdbook-content main');
        const active = document.querySelector('#mdbook-sidebar a.active');
        if (!main || !active) return;

        const chapter = active.textContent.trim();
        let section = null;

        let li = active.closest('li');
        while (li) {
            let sib = li.previousElementSibling;
            while (sib) {
                if (sib.classList && sib.classList.contains('part-title')) {
                    section = sib.textContent.trim();
                    break;
                }
                sib = sib.previousElementSibling;
            }
            if (section) break;
            li = li.parentElement ? li.parentElement.closest('li') : null;
        }

        if (!section && !chapter) return;

        const crumb = document.createElement('div');
        crumb.className = 'glide-breadcrumb';
        if (section) {
            const s = document.createElement('span');
            s.textContent = section;
            crumb.appendChild(s);
            const sep = document.createElement('span');
            sep.className = 'glide-sep';
            sep.textContent = '/';
            crumb.appendChild(sep);
        }
        const c = document.createElement('span');
        c.className = 'glide-crumb-current';
        c.textContent = chapter;
        crumb.appendChild(c);

        main.insertBefore(crumb, main.firstChild);
    }

    /* --- Front-page ASCII boot sequence ----------------------------------- */
    function setupBoot() {
        const boot = document.getElementById('boot');
        if (!boot) return;
        const terminal = boot.querySelector('pre');
        if (!terminal) return;

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

        const TYPE_MS = 12;
        const LINE_PAUSE = 40;
        const END_PAUSE = 550;

        document.body.classList.add('boot');

        const cursor = document.createElement('span');
        cursor.className = 'cursor';

        let i = 0;
        function nextLine() {
            if (i >= LINES.length) {
                finish();
                return;
            }
            const line = LINES[i];
            const div = document.createElement('div');
            terminal.appendChild(div);
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

        function finish() {
            terminal.appendChild(cursor);
            setTimeout(function () {
                document.body.classList.remove('boot');
                if (terminal.contains(cursor)) terminal.removeChild(cursor);
                // Re-run the page reveal animation after the boot clears.
                const main = document.querySelector('#mdbook-content main');
                if (main) {
                    main.style.animation = 'none';
                    void main.offsetWidth;
                    main.style.animation = 'glideUp 0.3s ease';
                }
            }, END_PAUSE);
        }

        nextLine();
    }

    /* --- Init ------------------------------------------------------------- */
    function init() {
        setupBranding();
        setupThemeToggle();
        setupSearchTrigger();
        setupCodeBlocks();
        setupPageToc();
        setupBreadcrumb();
        setupBoot();
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();

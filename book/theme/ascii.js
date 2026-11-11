/* Crussty ASCII boot animation — plays on the intro page only */
(function () {
    "use strict";

    function start() {
    var boot = document.getElementById("boot");
    if (!boot) return;

    var terminal = boot.querySelector("pre");

    var LINES = [
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

    var TYPE_MS = 14;      // per-character typing speed
    var LINE_PAUSE = 45;   // pause after each line
    var END_PAUSE = 600;   // pause before revealing content

    document.body.classList.add("boot");

    var holder = document.createElement("div");
    holder.className = "boot-lines";
    var cursor = document.createElement("span");
    cursor.className = "cursor";

    var i = 0;
    function nextLine() {
        if (i >= LINES.length) {
            finish();
            return;
        }
        var line = LINES[i];
        var div = document.createElement("div");
        terminal.appendChild(div);
        var pos = 0;
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
            document.body.classList.remove("boot");
            terminal.removeChild(cursor);
            setTimeout(function () {
                var main = document.querySelector(".content main");
                if (main) main.style.animation = "crussty-fadein 0.6s ease both";
            }, 30);
        }, END_PAUSE);
    }

    nextLine();
    }

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", start);
    } else {
        start();
    }
})();

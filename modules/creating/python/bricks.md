---
title: Platform bricks in Python
parent: Python
nav_order: 2
---

# <img class="page-icon" src="/CRUSSTY/assets/images/icons/mcf.svg" alt=""> Platform bricks in Python

Like the JS path, Python reaches the whole platform through the C shim:
`cplugin_init` receives the full `CPluginApi` (v3), so **every brick is
available** — the shim decides which ones to forward into Python.

Practical mapping:

| Brick | From Python | Notes |
|---|---|---|
| events | via shim | subscribe with a Python callback; events arrive as objects |
| scheduler | via shim | main-thread/tick dispatch into Python |
| network | via shim | registry, counters |
| storage | via shim | key/value, Python dict as store |
| threads | via shim | CPython GIL: one interpreter, one worker is simplest |
| hot_reload | via shim | re-import the module body on library swap |

Keep in mind: **all of it enters the interpreter from the hook thread** — the
GIL serializes access, so concurrency-sensitive bricks (events, threads)
work best with a worker-thread pattern inside Python.

Version-guard contract (`CPAPI_VERSION=3`, `CPB_VERSION=1`) is on
[Platform bricks](../../../platform/bricks.html).
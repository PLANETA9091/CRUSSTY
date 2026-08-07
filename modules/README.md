# Плагины

Плагины (модули) живут в отдельных репозиториях `c-<имя>`:

- [c-hello](https://github.com/PLANETA9091/c-hello) — минимальный proof-модуль (хук + GetLoadedClasses + JNI)
- [c-dist](https://github.com/PLANETA9091/c-dist) — движок dist (UDP-аренды, fencing) как модуль
- [c-crussty](https://github.com/PLANETA9091/c-crussty) — Crussty CE native surface как модуль

Установка: клонируй в `modules/<имя>` и собери (`cargo build && cp target/debug/lib<имя>.so .`),
либо упакуй директорию плагина в `.zip` и положи рядом — агент сам распакует
и загрузит.

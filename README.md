# CRUSSTY — native c-plugin платформа (v2)

Инжектит нативные Rust-модули в любой Paper-совместимый ядро: JVMTI-агент с
ClassFileLoadHook hot-patch конвейером. Модуль = плагин: директория (или
`.zip`/`.jar` архив) с `cplugin.json` и entry library.

## Структура

- `cplug-abi/` — единственный контракт агент ↔ модуль
- `cplug-sdk/` — SDK для авторов модулей (хуки, JNI, главный поток, ASM-вейвинг)
- `agent/` — JVMTI-агент: рекурсивный скан, топологическая загрузка, хук-конвейер
- `launcher/` — лаунчер (спавн ядра с `-agentpath`)
- `modules/` — плагины живут в своих репозиториях `c-<имя>`; клонируй их сюда
- `docs/V2-DESIGN.md` — дизайн платформы

## Сборка

```bash
cargo build --manifest-path agent/Cargo.toml
cp agent/target/debug/libdist_agent.so libdist_agent.so
javac -d launcher/out launcher/src/main/java/dev/dist/launcher/Main.java && \
  jar cfe launcher/launcher.jar dev.dist.launcher.Main -C launcher/out .
./run.sh
```

Нужен `versions/purpur-1.21.10.jar` (в репозиторий не входит).

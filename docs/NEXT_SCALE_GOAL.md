# Next Scale Goal

Дата: 2026-05-19 CEST

Status: historical/superseded. The superseding execution target moved to
`docs/CURRENT_2026-05-23_REAL_GOAL_P500_TO_NEAR_UNBOUNDED.md`.

Рабочая цель: near-unbounded measured scaling. Не literal unlimited claim.
Каждый новый шаг должен закрываться tier gate, soak, restart/recovery и
self-contained evidence bundle.

## Current Status

- [x] Исторический 500-bot claim существовал для артефакта
  `4064700022a879d83b16323cfbd0a769caf4551fdd8ed21dc7332afdd39d6b47`.
- [x] Soak wrapper теперь режет короткие прогоны ниже floor и не даёт снова
  запустить `1800s` там, где нужен full-load window.
- [x] Soak smoke больше не завязан на active-artifact binding.
- [x] Cold+warm soak на историческом артефакте `d4b27d49...` прошёл с
  `2400s` floor.
- [x] Historical claim bundle и публикация были обновлены:
  `reports/production-500-readiness-bundle-20260519-040502` и
  `reports/production-500-claim-current.{txt,md,json}`.
- [x] Fresh repeat quorum на historical artifact прошёл:
  `repeat_passes=3`, `repeat_failures=0`, `repeat_dir_count=1`.
- [ ] Superseded 2026-05-23 artifact snapshot
  `ece63dbd93423ac5797e439b54680c4d0a08b3f34f95d3de505cd375940b9ecc`
  still had no fresh green P500 claim or regenerated bundle; it is not
  current-artifact evidence now.
- [x] Readiness harness больше не смешивает old-artifact repeat dirs с
  fresh-artifact claim при fresh repeat refresh.
- [ ] Дойти до следующего измеренного tier: P500 mixed gameplay.
- [ ] Поднять mob tier до M10k mixed.
- [ ] Поднять chunk tier до C10k mixed.
- [ ] Получить 24h soak без watchdog/sync-load/queue blow-up.
- [ ] Проверить restart/recovery under load на следующем tier.
- [ ] Экспортировать и валидировать bundle для каждого принятого tier.

## Non-Claims

- [x] Не обещать literal unlimited.
- [x] Не обещать full Rust Paper runtime.
- [x] Не обещать real-player parity без отдельного измеренного gate.
- [x] Не обещать multi-hour soak без отдельного soak evidence.
- [x] Не обещать support for arbitrary plugins without a matrix gate.

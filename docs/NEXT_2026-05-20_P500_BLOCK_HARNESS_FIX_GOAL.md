# P500 Block Harness Fix Goal

Дата: 2026-05-20 CEST

Эта цель закрывает следующий реальный блокер перед новым claim по текущему
артефакту. Цель не про literal unlimited и не про полный Rust Paper runtime.
Цель: получить чистый, свежий и повторяемый `500 bots / 32 view / 32
simulation / creative block` gate на текущем artifact без загрязнения ранним
движением ботов.

## Текущий артефакт

- [x] `optimized-paper-1.21.10-mojmap.jar`
  `d2b676b0d533d31d58c3b9b366aaf944197e5e26d1b7db57405b41c1463a6ae9`
- [x] `artifacts/optimized-runtime/run.sh`
  `b047f4e266d6e518ec44f882486cfe212306e9fdf6e6896323fa89bef132f69a`
- [x] Старый зелёный claim для `d4b27d49...` больше нельзя выдавать за
  текущий artifact.

## Красный факт

- [x] Прогон `production-500-cold-soak-current-artifact-20260520-174258`
  достиг `500 created / 500 connected / 500 ready`.
- [x] Тот же прогон был остановлен красным: `kicked=5`, `errors=20`,
  финально `active=0`.
- [x] В серверном логе есть `616` строк `moved too quickly`.
- [x] Bot harness стартовал с `blockMovementMode=walk`, хотя block-profile
  должен ждать телепорта в подготовленную arena.
- [x] Thread sample красного прогона показывает горячий путь block destroy
  events и broadcast, но это нельзя считать финальным серверным bottleneck,
  пока стартовая фаза прогона загрязнена.

## Immediate Fix Ladder

- [x] Сделать block/mixed block-action profile default:
  `BOT_BLOCK_MOVEMENT_MODE=wait-for-teleport`.
- [x] Сделать direct `mc_bot_swarm.cjs --mode=block` default:
  `block-movement-mode=wait-for-teleport`.
- [x] Проверить shell syntax для production runners.
- [ ] Запустить короткий P500 diagnostic на текущем artifact с
  `wait-for-teleport`.
- [ ] Diagnostic должен иметь `moved_too_quickly_warnings=0`.
- [ ] Diagnostic должен удержать `500 ready / 500 active` без kicks и bot
  errors после action gate.
- [ ] Если diagnostic чистый, запустить полный cold soak на текущем artifact.
- [ ] Если cold soak зелёный, запустить warm soak на том же artifact.
- [ ] После cold+warm вернуть repeat quorum, plugin matrix,
  restart/recovery, forced-ticket persistence и fresh evidence bundle.

## Claim Rule

- [ ] Не писать `production-ready` для текущего artifact, пока свежий bundle
  не содержит green cold+warm soak, repeat quorum, plugin matrix,
  restart/recovery, forced-ticket persistence и artifact hashes.
- [ ] Не писать `unlimited players/mobs/chunks/ticks`.
- [ ] Не смешивать старые bundle/claim файлы `d4b27d49...` с текущим
  artifact `d2b676b0...`.

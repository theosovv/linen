# Backlog linen

Здесь фиксируются обоснованные будущие улучшения, которые не входят в текущий production-этап. Запись не является разрешением на реализацию: для работы нужна отдельная задача с allowlist, а изменение контрактов сначала получает `SPEC.md`/ADR по обычному процессу.

## BUF-001. Буферизация PCM16-записи при рендере

- Статус: предложено, не начато.
- Источник: benchmark baseline этапа 3 от 2026-11-21 в [`stages/stage-03-ir-bytecode-vm.md`](stages/stage-03-ir-bytecode-vm.md).
- Программа для повторного измерения: `fixtures/valid/stage-03-benchmark.ln`.

### Наблюдение

Renderer записывает каждый PCM16 sample отдельным `write_all`. На актуальном локальном sample-major baseline после review-исправлений для 60 секунд звука при block size 128 медианное wall time составило 5.381 с, включая примерно 4.664 с system time и 0.705 с user time. Block sizes `1`, `3`, `64`, `128`, `256` не меняли порядок результата; исторический instruction-major baseline сохранён в stage document.

Это делает буферизацию записи обоснованным кандидатом на оптимизацию, но не доказывает её эффект заранее.

### Возможный объём отдельной задачи

- Рассмотреть буферизацию на library renderer boundary, чтобы улучшение работало для любого `Write`, а не только для CLI `File`.
- Сохранить streaming: память ограничена выбранным output buffer и не растёт от длительности WAV.
- Сохранить PCM16 bytes, sample indices, diagnostics, `Write` error propagation и atomic CLI publish.
- Не менять VM/DSP, block semantics, WAV format или numeric tolerances попутно.

### Критерии приёмки

1. Все существующие audio/WAV/CLI/differential/compatibility tests проходят без изменения expected.
2. Runtime/I/O failure по-прежнему не публикует частичный итоговый WAV; buffered data корректно flushed только в рамках успешного render.
3. Повторный benchmark использует тот же fixture, release profile, sample rate и block sizes; документируются before/after median wall, user/system time и realtime factor при том же окружении.
4. Отсутствие ускорения — допустимый результат измерения, а не повод скрыть данные или добавить неоправданную сложность.
5. Новые numeric tolerances, performance gates на нестабильном CI и оптимизации VM не входят в эту задачу.

# Этап 5D — Явные PolyBLEP-осцилляторы

- Статус: спецификация принята и подготовительная test-facing задача завершена 2026-11-22; production-алгоритм и финальная приёмка завершены 2026-11-24.
- Источник требований: `SPEC.md` 0.7.0, разделы 0, 3–5, 6.2, 7.4–7.11, 8, 9 (этап 5D), 10–14.
- Связанные ADR: `0016`; сохраняются `0001`–`0007`, `0009`, `0011`–`0015`.
- Базовый этап 5 завершён; исторический отчёт `stage-05-basic-oscillators-stereo.md` не переписывается.
- Блокирует: завершение anti-aliasing подэтапа; не разрешает начинать этап 6 в том же diff.

## 1. Контекст

Этап 5 завершён по `SPEC.md` 0.6.0: naive phase oscillators, `Stereo`/`pan`, interleaved VM/WAV и deterministic noise проходят 239 integration tests и allocation harness. ADR 0013 прямо запрещает скрытую замену naive waveform на anti-aliased алгоритм.

5D — отдельный корректирующий подэтап. Он добавляет два явных PolyBLEP-осциллятора, сохраняет весь прежний звук и фиксирует границу между низкоуровневыми builtins и будущей явно импортируемой stdlib. Пользователь выбрал подготовительную схему: агент пишет SDD, representations/mechanical plumbing, independent reference/tests и три временные заглушки; человек реализует основной lowering и DSP transitions.

## 2. Ожидаемый результат после production-завершения

- `saw_polyblep(freq)` и `square_polyblep(freq)` работают через source → Typed AST → IR → bytecode → VM → WAV.
- 2-point PolyBLEP соответствует точным формулам ADR 0016, включая initial samples, negative/zero/Nyquist/above-Nyquist frequency и nonfinite diagnostics.
- Naive `saw`/`square` и все прежние fixtures сохраняют samples и WAV bytes.
- Новые ops получают descriptors, origins, state/reset, verifiers, resources/capability, independent reference, differential/property и allocation coverage.
- Spectral acceptance подтверждает снижение alias energy, а посемпловые vectors доказывают выбранный алгоритм.
- Будущая stdlib может добавлять обёртки/композиции, но modules/imports и stdlib в этом подэтапе не реализуются.

## 3. Объём и allowlist подготовительной задачи

### Документация

- `SPEC.md`, `CHANGELOG.md`, `AGENTS.md`;
- `docs/**`.

### Production-каркас

- `src/sema/types.rs`, `src/sema/builtins.rs`;
- `src/ir/graph.rs`, `src/ir/lower.rs`, `src/ir/verify.rs`;
- `src/bytecode/instruction.rs`, `src/bytecode/codegen.rs`, `src/bytecode/verify.rs`;
- `src/dsp/mod.rs`, `src/dsp/oscillator.rs`;
- `src/vm/state.rs`, `src/vm/execute.rs`.

Production агента ограничен descriptors, enum/representation variants, mechanical codegen/verifier/VM plumbing и тремя заглушками из §6. Основные lowering/DSP algorithms принадлежат человеку.

### Tests и oracles

- `tests/**`, `fixtures/**`, `test-support/reference/**`.

`README.md`, `Cargo.toml`, `Cargo.lock`, toolchain, dependencies, CLI, renderer, WAV и module/package infrastructure не меняются: до production-завершения нельзя рекламировать новые builtins как работающие.

## 4. Нецели

- Изменение naive `saw`/`square`, `sine`, `phasor` или `triangle` и их samples.
- `triangle_polyblep`, `phasor_polyblep`, PolyBLAMP, DPW, minBLEP, oversampling, wavetables и общий algorithm selector.
- Низкоуровневый `polyblep()` builtin, новые phase/select/sample-rate языковые операции или глобальный AA config.
- Modules/imports/source stdlib, plugins, stable bytecode ABI, filters/feedback, этап 6 и последующие возможности.
- Новые dependencies/toolchain, CLI flags, diagnostic codes, WAV/channel/layout changes.
- Placeholder success, тишина, фиктивный IR/output или зелёная приёмка production 5D до удаления заглушек.

## 5. Контракты реализации

1. Добавляются `BuiltinId::{SAW_POLYBLEP,SQUARE_POLYBLEP}`, два полных phase descriptors и algorithm-specific names. Порядок IDs детерминирован, descriptors остаются в существующей статической таблице.
2. `NodeOp::{SawPolyblep,SquarePolyblep}` и одноимённые `Instruction` принимают один `Sig` frequency, возвращают `Sig`, используют один audio register/StateId и `f64` phase. Capability — `CoreAudio`.
3. Frequency expression lower’ится один раз; origin узла — frequency expression с текущим inner-to-outer call chain. Runtime nonfinite error сохраняет `E4001`, exact span и zero-based frame index.
4. Начальная phase `0`; output предшествует signed increment. Коррекция использует `abs(phase_step)`, все промежуточные values `f64`, результат приводится к `f32` после коррекции.
5. При `dt == 0` или `dt > 0.5` correction равна нулю; `dt == 0.5` использует PolyBLEP. Это явный числовой контракт, а не скрытый fallback.
6. Sharing/fresh state, reset, eager roots, block-independent error order, debug stripping и zero-allocation steady-state сохраняются.
7. Будущие stdlib names/wrappers не резервируются и не создаются; два новых algorithm-specific builtin не вводят универсальный AA API.

## 6. Три разрешённые временные заглушки

Только следующие entry points могут содержать `todo!("stage 5D: ...")`:

1. `ir::lower::lower_stage5d_builtin` — one-time frequency lowering, выбор нового `NodeOp` и origin plumbing в конечном вызове.
2. `dsp::SawPolyblep::next_sample` — проверка frequency, saw PolyBLEP transition и phase update.
3. `dsp::SquarePolyblep::next_sample` — проверка frequency, square PolyBLEP transition и phase update.

Constructors/reset/accessors, descriptors, IR/bytecode representations, codegen, verifiers и VM state/dispatch не должны быть placeholder success. Прежние builtins и все старые programs не проходят через эти заглушки. `unimplemented!()`, заглушки вне списка и user-reachable fallback запрещены. Все три заглушки удаляются до завершения 5D.

## 7. Порядок production-реализации человеком

1. Реализовать `lower_stage5d_builtin`; проверить exact IR nodes/origins, one-time frequency и function fresh state через `polyblep`, IR/bytecode suites.
2. Реализовать оба DSP transitions по ADR 0016 без общего кода с independent reference; прогнать analytic vectors и edge cases.
3. Удалить все три `todo!`, прогнать generated differential/properties, spectral/reset/partition/debug-strip и allocation scenarios.
4. Выполнить real CLI `check`/`run` для двух новых builtins, runtime/atomic error и cross-process determinism; полный общий набор зелёный.
5. Записать фактические результаты отдельным разделом, обновить README только с реально работающими examples и отметить 5D завершённым после полного DoD.

## 8. Матрица проверки

| Требование | Positive/negative/boundary cases | Независимый oracle |
|---|---|---|
| ADR 0016 — descriptors/API | names, signatures, IDs, function-name conflicts, value binding allowed | Exact descriptors, `E2000`, SourceId/spans |
| Saw PolyBLEP | wrap, обе зоны `B`, outside, initial/zero/negative | Явные formula values и fixed arrays |
| Square PolyBLEP | wrap + half-cycle, shifted phase, boundaries `t=dt`, `t=1-dt`, `t=0.5` | Явные formula values и fixed arrays |
| Frequency edges | `dt=0`, `0<dt<0.5`, `dt=0.5`, `dt>0.5`, NaN/Inf | Analytic naive/correction branches и exact phase |
| State/runtime | partitions, variable blocks, reset, long render, sharing/fresh calls | Independent reference и exact repeated samples |
| IR/bytecode/VM | types, state uniqueness/coverage, origins, dispatch, debug stripping | Explicit graphs/instructions и typed corruption errors |
| Spectral acceptance | fixed 1001 Hz/48 kHz corpus и 12 alias bins для обеих forms | Direct DFT, analytic bin list, минимум 20 dB reduction |
| Compatibility | 239 прежних integration tests, mono/stereo/noise/WAV/CLI | Старые expected без ослабления |

Audio tolerance — `2e-5 + 2e-5 × abs(expected)`; exact IDs, resources, spans, state/frame indices и discrete instruction/layout values сравниваются точно. Spectral threshold не заменяет посемпловые comparisons и не повышает audio tolerance.

## 9. Definition of Done production 5D

- [x] Оба builtins работают через production source → WAV и соответствуют каждой формуле/branch ADR 0016.
- [x] Positive/negative/boundary vectors, zero/negative/Nyquist/high/nonfinite frequencies и initial/reset state проверены.
- [x] Naive waveform samples и все прежние compatibility contracts не изменены.
- [x] Origins, descriptors, resources/capability, verifiers, state ownership и debug stripping покрыты.
- [x] Independent reference, generated differential/properties, variable partitions и spectral acceptance проходят.
- [x] Steady-state `render_into`/`reset` остаются allocation-free.
- [x] Все три временные заглушки удалены; нет ignored tests, новых dependencies или future-stage infrastructure.
- [x] Real CLI check/run/WAV, runtime/atomic error и repeated-process determinism проверены.
- [x] Узкие и общие проверки запущены и прошли; README/docs описывают только фактическую функциональность.

## 10. Критерии приёмки подготовительной задачи

- [x] SPEC/ADR/stage/task зафиксировали контракт до production-алгоритма.
- [x] Diff ограничен allowlist; ровно три разрешённые заглушки, никаких других placeholders.
- [x] Все targets компилируются; новые tests имеют independent expected и не используют ignored.
- [x] Все 239 прежних integration tests и прежний allocation scenario проходят.
- [x] Ожидаемые failures достигают только документированных заглушек и учтены честно.
- [x] Статический набор, doctest и rustdoc запущены; production 5D не объявлен завершённым.

## 11. Обязательные команды

Подготовительный diff:

```bash
cargo test --test polyblep --locked --no-run
cargo test --all-targets --locked --no-run
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked --no-fail-fast
cargo test --doc --locked
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked
git diff --check
```

Финальная приёмка после production-реализации использует обычный `cargo test --all-targets --locked` без ожидаемых failures, отдельные `polyblep`/`differential`/`properties`/`compatibility`/`allocations`, real CLI scenarios и полный DoD.

## 12. Фактические результаты подготовительной задачи — 2026-11-22

- Спецификация обновлена до 0.7.0, принят ADR 0016, созданы отдельные stage/task documents. Зафиксированы два algorithm-specific builtins, точный 2-point PolyBLEP, shifted-phase rounding contract, zero/Nyquist/above-Nyquist branches и граница будущей stdlib.
- Добавлены `BuiltinId::{SAW_POLYBLEP,SQUARE_POLYBLEP}` с сохранением прежних numeric IDs, полные descriptors, отдельные NodeOp/Instruction variants, codegen/verifiers и VM state/reset/dispatch. Constructors/accessors/reset работают; основной lowering и numeric transitions не реализованы.
- В production остаются ровно три разрешённые заглушки: `lower_stage5d_builtin`, `SawPolyblep::next_sample`, `SquarePolyblep::next_sample`. Поиск не нашёл `unimplemented!()` или других `todo!()` в `src`; прежние paths не проходят через новые заглушки.
- Independent reference содержит отдельные formulas и прошёл fixed vectors для обеих forms, strict boundaries, zero/negative/Nyquist/above-Nyquist cases. Fixed-seed generator и 15 новых tests покрывают production vectors, diagnostics, scalar/audio operands, origins/state, verifier corruption, direct DFT, differential/properties/compatibility и allocations. Добавлены три fixtures.
- `polyblep`: 4 passed, 7 ожидаемых failed; новые differential/properties/compatibility/allocation tests — ещё четыре ожидаемых failed. Два failures вызывают DSP stub, девять — lowering stub. Других причин failures нет.
- Финальный aggregate: **253 integration + 2 allocation tests = 255; 244 passed, 11 expected failed, 0 ignored**. Все 239 прежних integration tests и прежний allocation scenario проходят. Старые audio/WAV expected не изменены; два descriptor count assertions механически обновлены с 10 на 12.
- `cargo test --all-targets --locked --no-run`, `cargo fmt --check`, `git diff --check`, clippy `-D warnings`, doctest (1 passed) и rustdoc `-D warnings` успешны. Полный `cargo test --all-targets --locked --no-fail-fast` завершился только с перечисленными ожидаемыми failures.
- При подготовке исправлены test-only точность decimal literals (заменены exact rational formulas) и два descriptor counts. Это не production-дефекты. До исправления shifted-phase contract независимый расчёт выявил rounding edge у `(t + 0.5).rem_euclid(1)`; спецификация и ADR уточнены до production-реализации.
- Production lowering/DSP, real CLI/WAV, фактический spectral/allocation/reset/differential success и полный DoD 5D не проверены и не завершены. Следующий шаг — ручная реализация по §7, удаление трёх заглушек и отдельная финальная приёмка без переписывания этого отчёта.

## 13. Фактические результаты production 5D — 2026-11-24

- Реализованы one-time frequency lowering с origin текущего вызова и оба DSP transition по ADR 0016. Все три подготовительные заглушки удалены; `todo!()`/`unimplemented!()` в `src` отсутствуют. Naive oscillator code и прежние audio/WAV expected не изменены.
- Review-исправление добавило regression для `t < 0.5` с округлением `t + 0.5` до `1.0`: правая ветвь `B(1.0, dt)` проверена отдельно от modulo-реализации. Также удалён оставшийся после реализации лишний комментарий. Исторический отчёт §12 не переписывался.
- `polyblep`: **12 passed, 0 failed**; отдельные differential/properties/compatibility/allocation suites проходят. Финальный aggregate: **254 integration + 2 allocation tests = 256; 256 passed, 0 failed, 0 ignored**.
- `cargo fmt --check`, clippy `-D warnings`, `cargo test --all-targets --locked`, doctest (1 passed), rustdoc `-D warnings` и `git diff --check` успешны. Spectral acceptance, generated differential/properties, variable partitions, reset, debug stripping и zero-allocation checks входят в эти зелёные suites.
- Реальные CLI `check` и `run` для fixture с обоими builtins создают WAV на 96 frames при 48 000 Hz. Два независимых процесса дали идентичные WAV bytes. Runtime `E4001` с `--force` сохранил существующий output без изменений; временные проверочные artifacts удалены.
- Все пункты §9 выполнены. Новых dependencies, ignored tests, placeholders и future-stage infrastructure нет. Подэтап 5D завершён; этап 6 и другие AA algorithms не начинались.

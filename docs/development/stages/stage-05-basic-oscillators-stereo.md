# Этап 5 — Базовые осцилляторы и стерео

- Статус: завершён 2026-11-22; production-реализация, review-исправления и полный DoD этапа 5 выполнены. Исторический отчёт подготовительной задачи сохранён в §11.
- Источник требований: `SPEC.md` 0.6.0, разделы 0, 2.2, 3–5, 6.2–6.5, 7.4–7.11, 8, 9 (этап 5), 10–14.
- Связанные ADR: `0013`, `0014`, `0015`; сохраняются `0001`–`0007`, `0009`, `0011`, `0012`.
- Блокирует: anti-aliasing подэтап, этап 6 и последующие DSP/языковые этапы.

## 1. Контекст

Этап 4 завершён: все 208 integration tests и общий набор проходят. Production path использует Typed AST → IR → bytecode → block VM → mono PCM16 WAV. Этап 5 расширяет существующий graph/runtime базовыми осцилляторами, arithmetic builtins, явным `Stereo` и детерминированным noise без событий, GUI и общего multichannel.

Пользователь выбрал подготовительную схему этапа 4: агент фиксирует контракты, representations, механическую plumbing, independent reference/tests; человек реализует основные sema/lowering/DSP algorithms и удаляет временные заглушки.

## 2. Ожидаемый результат после завершения

- `phasor`, naive `saw`/`square`/`triangle` имеют общий per-sample `f64` phase contract, analytic vectors, reset и block-invariance.
- `gain` и `mix` работают как descriptor-driven arithmetic sugar без дублирования stateful arguments.
- `Stereo`, `stereo` и equal-power `pan` поддерживают functions/scopes/origins; `out` производит mono или stereo WAV.
- VM и WAV используют явный channel layout и interleaved `L,R`; block/sample index считаются frames.
- `noise(seed)` использует explicit `RandomStreamId` и SplitMix64, не зависит от compiler layout и повторяется после reset.
- Все прежние programs, mono PCM bytes, diagnostics, atomic CLI, allocation и compatibility contracts сохраняются.

## 3. Объём и allowlist подготовительной задачи

### Документация

- `SPEC.md`, `CHANGELOG.md`, `README.md`, `AGENTS.md`;
- `docs/**`.

### Production-каркас

- `src/api.rs`, `src/diagnostic.rs`, `src/lib.rs`, `src/main.rs`;
- `src/syntax/**`, `src/sema/**`, `src/ir/**`, `src/bytecode/**`;
- `src/vm/**`, `src/dsp/**`, `src/render/**`.

Production агента ограничен representations, descriptors, mechanical codegen/verifier/channel plumbing и восемью явными заглушками ниже. Основные алгоритмы принадлежат человеку.

### Tests и oracles

- `tests/**`, `fixtures/**`, `test-support/reference/**`.

`Cargo.toml`, `Cargo.lock`, toolchain, dependencies и CI/tooling upgrades запрещены.

## 4. Нецели

- PolyBLEP, band-limited oscillators и скрытая замена naive formulas. Это отдельный последующий подэтап с собственной numeric acceptance.
- Filters, delay, feedback, events, envelopes, MIDI, polyphony, probes и оптимизации.
- Generic multichannel, dynamic channel count, automatic mono duplication/downmix и Stereo arithmetic.
- `RenderConfig`-owned random, system random, random UUID или NodeId-based streams.
- Persisted/stable bytecode ABI, `inspect`, modules/packages/registry/plugins, dependencies/toolchain changes.
- Зелёная приёмка этапа 5 в подготовительном diff или объявление новых builtins работающими до удаления заглушек.

## 5. Контракты реализации

### 5.1. Builtins и phase

Сигнатуры и formulas определены ADR 0013. Все frequency-based oscillators, включая прежний `sine`, проверяют runtime finite frequency до phase update и возвращают `E4001` с exact argument origin/frame index. Начальная phase `0`; output предшествует increment; increment использует `f64(freq)/sample_rate` и `rem_euclid(1.0)`.

`gain(input, amount)` и `mix(a, b, balance)` lower’ятся в существующую арифметику. Args вычисляются один раз слева направо; повторное использование balance — shared NodeId, не duplicate state. `balance` не clamp’ится.

### 5.2. Stereo и pan

`TypeName::Stereo`, `Type::Stereo` и `ValueType::Stereo` представляют ровно два канала. Разрешены annotations, bindings, params/results и `out`; implicit conversions кроме прежнего `Float → Sig` отсутствуют. `stereo(left,right)` сохраняет left-before-right evaluation.

`pan` использует equal-power formula ADR 0014, clamp position и position-primary `E4001` для NaN/Inf. Stereo node занимает пару audio registers; output layout явный. VM slice и PCM16 interleaved; `block_size`, sample index и report samples выражены в frames.

WAV channel count только `1` или `2`; header sizes и data bytes вычисляются checked из frames/channels. Старый `write_wav_header` mono contract может сохраняться как wrapper, но stereo не подменяет его неявно.

### 5.3. Noise

`noise(seed: Float): Sig` принимает finite integer `0..=2^53−1`. Finite invalid seed — `E2107`; ошибки вычисления Float имеют прежний приоритет. Generic parameter-dependent seed проверяется на конкретизации с definition primary/call chain.

`RandomStreamId` содержит только explicit `u64` seed. IR хранит constant seed input и stream ID; bytecode сохраняет stream ID. SplitMix64 и conversion верхних 24 бит определены ADR 0015. Одинаковые seeds дают одинаковые samples, но states продвигаются и сбрасываются независимо.

### 5.4. Origins, resources и execution order

- Phase origins — frequency expression; pan origin — position expression для parameter error; stereo/sugar synthetic nodes — полный call value span, если отдельный parameter origin не требуется.
- Inner-to-outer `CallOrigin` сохраняется для всех новых nodes и `E2107`/`E4001`.
- `RuntimeCapability::Multichannel` добавляется только фактическим stereo ops; остальные используют `CoreAudio`.
- Resource metadata отражает registers/state текущих ops. Sugar resources учитываются фактически созданными arithmetic nodes.
- Sample-major instruction order, eager roots, left-to-right errors и debug stripping сохраняются. Stereo output errors проверяют left перед right.

## 6. Восемь разрешённых временных заглушек

Только следующие entry points могут содержать `todo!("stage 5: ...")`:

1. `sema::check::validate_noise_seed` — concrete/generic seed validation и `E2107`.
2. `ir::lower::lower_stage5_builtin` — arithmetic sugar, phase/stereo/pan/noise lowering и origins.
3. `dsp::Phasor::next_sample`.
4. `dsp::Saw::next_sample`.
5. `dsp::Square::next_sample`.
6. `dsp::Triangle::next_sample`.
7. `dsp::pan` — equal-power numeric transition.
8. `dsp::Noise::next_sample` — SplitMix64 и output conversion.

`new`/`reset`/accessors, descriptors, representations, structural codegen/verifiers, VM dispatch и mono/stereo WAV plumbing не должны быть placeholder success. Старый sine/mono path не проходит через эти заглушки. `unimplemented!()`, тишина, фиктивный IR/output и user-reachable `unreachable!()` запрещены. Все восемь заглушек удаляются до завершения этапа.

## 7. Порядок production-реализации человеком

1. **5A — phase и arithmetic:** реализовать четыре DSP transitions и `lower_stage5_builtin` для oscillators/gain/mix. Прогнать `oscillators`, verifier/reference suites.
2. **5B — Stereo:** завершить stereo/pan lowering и `dsp::pan`, проверить explicit layouts, channel diagnostics, VM partitions/reset и WAV bytes через `stereo`.
3. **5C — Noise:** реализовать seed validation/`E2107`, noise lowering и SplitMix64; прогнать `noise`, independent vectors, stream identity и function origin cases.
4. **Общая приёмка:** удалить все `todo!`, расширить generated differential/properties и allocation harness, выполнить real CLI mono/stereo/noise/atomic scenarios и полный набор.

Разрешено делить `lower_stage5_builtin` на внутренние helpers во время реализации без изменения независимых expected. Если нестабильные Rust signatures уточняются, tests мигрируют механически, а task/stage docs обновляются без ослабления contracts.

## 8. Матрица проверки

| Требование | Positive/negative/boundary cases | Независимый oracle |
|---|---|---|
| ADR 0013 — waveforms | phase vectors, discontinuities, zero/negative/high/nonfinite freq | Явные phase/formula values |
| Reset/block/state | все oscillators, sharing/fresh function calls | Посемпловый independent reference |
| gain/mix | exact gains, balance `-1/0/0.5/1/2`, stateful args | Явная f32/f64 арифметика и node counts |
| Stereo typing | annotations/functions/lifts/out, forbidden arithmetic/conversions | Exact Type/BuiltinId/code/SourceId/spans |
| ADR 0014 — pan | `-2/-1/0/1/2`, nonfinite position/input | Независимые cos/sin vectors и tolerance |
| Channel layout | L/R independence, odd/oversized slices, left-before-right errors | Явные interleaved arrays и structured errors |
| Stereo WAV | channels/byte rate/block align/data size/interleaving | Вручную рассчитанные header/PCM bytes |
| ADR 0015 — seed | min/max/fraction/negative/nonfinite/per-call | Exact `E2107`/`E2105`, spans/call chain |
| SplitMix64 | fixed seeds/wrapping/reset/same/different streams | Независимо рассчитанные `u64`/audio vectors |
| Compatibility | 208 прежних tests, mono WAV, CLI/atomic/determinism/allocations | Старые expected без ослабления |

Audio tolerance остаётся `2e-5 + 2e-5 × abs(expected)`; exact arithmetic, IDs, layouts, spans, frame indices и PCM bytes сравниваются точно там, где представление точное. Naive oscillators не проверяются как band-limited.

## 9. Definition of Done этапа

- [x] Все новые builtins и `Stereo` работают через production source → WAV.
- [x] Для каждого stateful node начальное состояние, edge cases, reset и block-invariance проверены.
- [x] Naive waveform и будущий anti-aliasing не смешиваются.
- [x] gain/mix сохраняют one-time args и sharing.
- [x] Stereo layout, pan law, channel diagnostics и WAV interleaving соответствуют ADR 0014.
- [x] Noise seed boundaries, stable stream identity, independent state и SplitMix64 vectors проходят.
- [x] IR/bytecode verifiers, origins, capabilities/resources и debug stripping покрывают новые ops.
- [x] Differential/properties/generated corpus расширены всеми новыми типами/узлами.
- [x] Прежние 208 tests и mono compatibility проходят без изменения expected.
- [x] Steady-state mono/stereo `render_into`/`reset` остаются allocation-free.
- [x] Нет временных заглушек, ignored tests, новых dependencies или future-stage infrastructure.
- [x] Real CLI check/run, stereo/noise WAV, runtime/atomic error scenarios проверены.
- [x] Узкие и общие проверки запущены и прошли; документация описывает фактическое состояние.

## 10. Обязательные команды

```bash
cargo test --test oscillators --locked --no-run
cargo test --test stereo --locked --no-run
cargo test --test noise --locked --no-run
cargo test --all-targets --locked --no-run
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
cargo test --doc --locked
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked
```

Для подготовительного diff `--no-run` доказывает компиляцию; новые runtime tests ожидаемо красные на документированных заглушках. Полный `--no-fail-fast` используется для учёта. Финальная приёмка требует обычные команды без `--no-run`, все suites зелёные и ноль ignored.

## 11. Фактические результаты

### Подготовительная задача агента — 2026-11-22

- Обновлены `SPEC.md` 0.6.0, ADR 0013/0014/0015 и SDD/status documents. Добавлены contextual `Stereo`, десять builtin descriptors, phase/stereo/noise IR ops, `RandomStreamId`, явный `OutputLayout`, stereo codegen/verifiers, typed VM states, interleaved VM/WAV и source-aware frequency/position diagnostics plumbing.
- Оставлены ровно восемь разрешённых `todo!()`: noise seed validation, stage-5 builtin lowering, четыре waveform transition, pan и SplitMix64. `unimplemented!()` и ignored tests отсутствуют. Прежний `sine`/mono/functions path не проходит через заглушки.
- Independent reference расширен всеми новыми builtins, Stereo frames и собственными phase/pan/SplitMix64 algorithms. Добавлены fixed-seed mono/stereo/noise generators, 28 integration tests и девять stage-5 fixtures; stereo allocation scenario включён последовательно в существующий harness.
- Ручной stereo path `TypedProgram + Graph → codegen → VM → render` создал 56-byte PCM16 WAV: 3 frames, 2 channels, data size 12, interleaved exact bytes. VM odd/oversized slice checks, reset, debug stripping и 0 steady-state allocations проходят.
- `cargo test --all-targets --locked --no-run` успешен без warnings. `cargo fmt --check`, `git diff --check`, clippy `-D warnings`, doctest (1 passed) и rustdoc `-D warnings` успешны.
- Все 208 прежних integration tests и отдельный allocation harness проходят. Новые suites: `oscillators` 5 passed/5 expected failed, `stereo` 7/3, `noise` 4/4. Общий `--no-fail-fast`: 236 integration tests + 1 allocation test, aggregate 225 passed, 12 expected failed, 0 ignored; failures достигают только документированных заглушек.
- Сознательный compatibility break согласован: function declarations с именами новых builtins отклоняются `E2000`; stage-4 `mix` fixture переименован в `mix_signals` без изменения audio expected. Старый unknown-function fixture использует `noize` вместо ставшего builtin `noise`, сохраняя code/span проверки.
- Production phase/pan/noise algorithms, общий builtin lowering и seed diagnostics не реализованы. Generated source corpus и real stage-5 CLI source → WAV не выполнены до ручной реализации; full green DoD, anti-aliasing и этап 6 не начаты. Этап 5 не завершён.

## 12. Production-реализация и финальная приёмка — 2026-11-22

Этот раздел добавлен после production-реализации и review. Он не изменяет исторические результаты подготовительной задачи из §11.

- Все восемь временных заглушек удалены. Реализованы phase transitions, descriptor-driven `gain`/`mix`, stage-5 lowering, `Stereo`/`pan`, seed validation/`E2107` и SplitMix64 `noise`. Naive oscillators не заменяются PolyBLEP или другим anti-aliasing.
- Review выявил и регрессионно закрыл два дефекта: computed/negative-zero noise seeds теперь согласованно оцениваются lowering и IR verifier; `MakeStereo` читает оба источника до записи, а одинаковые stereo destinations отклоняются verifier. Добавлен `fixtures/regression/stage-05-noise-computed-seed.ln` и register-alias corruption tests.
- Production waveforms проходят block sizes `1/3/64/128/256`, zero/negative/above-Nyquist/nonfinite frequency cases, reset и 100 000-sample phase checks. `gain`/`mix` и `pan` проверены на всех документированных boundaries; noise — на fixed vectors, computed/per-call seeds, partitions, reset и debug stripping.
- Generated mono/stereo/noise differential suites и stage-5 property test проверяют все новые типы/узлы, variable frame partitions, reset и verifiers. Allocation harness прогоняет реальные oscillator/stereo/pan/noise graphs и подтверждает 0 steady-state heap allocations на `render_into`/`reset`.
- Финальный corpus: **239 integration tests + 1 allocation test, 0 failed, 0 ignored**. Из них 31 test относится к этапу 5; добавлены десять stage-5 fixtures с учётом regression fixture. Все 208 прежних integration tests и mono compatibility проходят без ослабления expected.
- `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --all-targets --locked`, `cargo test --doc --locked` (1 passed), `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked` и `git diff --check` успешны.
- Реальные CLI `run` успешно создали mono Beep, oscillator, stereo, pan, noise и computed-seed WAV. Два независимых noise process с block sizes `1` и `256` дали побайтово одинаковый WAV. Runtime-ошибка nonfinite `pan` position вернула `E4001`/exit `1` и сохранила существующий destination без частичной записи. Временные файлы удалены.
- Этап 5 завершён. Anti-aliasing, filters/feedback, этап 6 и module/package infrastructure не начинались.

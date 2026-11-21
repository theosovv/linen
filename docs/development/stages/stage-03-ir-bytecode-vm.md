# Этап 3 — IR, bytecode и блочная VM

- Статус: завершён 2026-11-21; production IR/bytecode/VM, renderer, allocation harness и benchmark baseline реализованы и задокументированы.
- Источник требований: `SPEC.md` 0.4.0, разделы 0, 2.2, 3, 4.3–4.4, 4.8, 5.1–5.11, 6.1–6.5, 7.1–7.11, 8, 9 (этап 3), 10–14.
- Связанные ADR: `0004`, `0005`; сохраняются `0001`, `0002`, `0003`, `0006`, `0009`, `0011`, `0012`.
- Экосистемные инварианты: source-aware `NodeOrigin`, resolved IDs, deterministic graph/codegen, capability/resource metadata, отсутствие скрытого I/O и отделение runtime от host products.
- Блокирует: этап 4 — функции, блоки и pipeline.

## 1. Контекст

Этап 2 завершён и фактически проверен: 121 integration test, общий набор и ручной Beep/atomic/determinism сценарии проходят. Production path пока использует `Typed AST → NaiveInterpreter → WAV`; независимый reference находится только в `test-support`.

Этап 3 должен заменить внутренний execution path:

```text
Typed AST → signal IR → IR verifier → bytecode → bytecode verifier → block VM → WAV
```

Язык, diagnostic codes/spans/sample indices, CLI и PCM16 WAV остаются совместимыми. Подготовительный агентский diff создаёт SDD-документы, representations, test-facing seams и независимые tests; человек реализует lowering, verifiers, codegen и VM.

## 2. Ожидаемый результат после завершения

- `compile` после semantic checks строит canonical typed IR, проверяет его, создаёт typed register bytecode и проверяет structural/origin invariants.
- IR сохраняет все top-level values как evaluation roots, sharing resolved bindings, explicit lifts, checked sample count и валидный `NodeOrigin` у каждого node.
- Каждый IR op и builtin `sine` имеют согласованные `CoreAudio` и resource metadata.
- Bytecode различает scalar/audio registers и `StateId`; optional debug origins можно удалить без изменения звука.
- `create_vm` создаёт полностью предвыделенную VM. `render_into` принимает empty/variable blocks в пределах configured maximum и не выделяет память в steady-state.
- `reset()` восстанавливает registers, oscillator states и absolute sample index.
- Production WAV renderer использует VM; наивный production interpreter удалён только после полного parity, independent reference сохранён.
- Все прежние Beep fixtures, diagnostics, CLI atomicity и WAV expected сохраняются.
- Измерен baseline наивной VM с документированной конфигурацией; никаких универсальных performance promises.

## 3. Объём и allowlist подготовительной задачи

Пользователь явно выбрал подготовительную задачу и разрешил механический production-каркас с временными `todo!()` в обозначенных entry points. Основной lowering/codegen/verifiers/VM пишет человек.

### Документация

- `SPEC.md`, `CHANGELOG.md`, `README.md`, `AGENTS.md`;
- `docs/adr/**`, `docs/development/**`.

### Production-каркас

- `src/api.rs`, `src/diagnostic.rs`, `src/lib.rs`, `src/main.rs`;
- `src/sema/builtins.rs`, `src/sema/mod.rs`, `src/dsp/**`, `src/render/**`;
- новые `src/ir/**`, `src/bytecode/**`, `src/vm/**`.

### Tests и oracles

- `tests/**`, `fixtures/**`, `test-support/reference/**`.

`Cargo.toml`, `Cargo.lock`, toolchain, dependencies, `benches/**`, module/package infrastructure и другие production modules не входят в задачу. Расширение allowlist требует отдельного согласования.

## 4. Нецели

- Изменение Beep syntax/types, diagnostic codes/spans, CLI flags, WAV/PCM16 или render limits.
- Optimization passes, constant folding, CSE, dead-node elimination, buffer reuse и rate inference.
- Пользовательские функции, `FunctionId`, nested scopes, inlining и call-origin chains.
- Новые DSP nodes, stereo, filters, events, feedback, noise и probes.
- `inspect`, serialized/persisted bytecode, stable ABI, versioned device packages и plugin interfaces.
- Modules/imports/exports, manifest, registry, network и package CLI.
- Real-time backend, SIMD, JIT и новые dependencies/toolchain.
- Объявление этапа завершённым при временных заглушках, красных tests или невыполненном DoD.

## 5. Принятые контракты реализации

### 5.1. IR и lowering

1. `Graph` содержит `nodes`, `roots`, `output`, `render_samples`. `NodeId` — Vec index; node IDs совпадают с indices, inputs строго предшествуют consumer.
2. Roots содержат values всех bindings в declaration order, включая не используемые в `out`; output входит в roots и имеет `Sig`. Duplicate NodeId среди roots означает sharing.
3. Identifier reference повторно использует lowered node binding; `Group` прозрачен. Обход left-to-right, оптимизации отсутствуют.
4. `ConstFloat`/scalar ops сохраняют `f64`; signal runtime использует `f32`. Binary IR inputs имеют одинаковый тип благодаря явным lifts.
5. `NodeOrigin` содержит primary span. Для `Sine` primary — frequency expression для стабильной `E4001`; полный out span сохраняется для output diagnostics/SetOutput origin.
6. IR verifier проверяет IDs, inputs/ordering, типы, finite constants, roots/output, reachability и origins через исходную `SourceMap`.

Подробности: ADR 0004.

### 5.2. Capabilities и resources

1. Все существующие ops относятся к `RuntimeCapability::CoreAudio`; пустой enum будущих capabilities не создаётся.
2. `ResourceModel` содержит scalar registers, audio registers, state slots и state bytes на node. `Sine` требует один audio register, один state slot и одну `f64` phase.
3. Builtin descriptor и IR op descriptor для `sine` согласованы. Buffer bytes зависят от maximum block size и проверяются на overflow до allocation.

### 5.3. Bytecode и verification

1. Scalar registers содержат `f64`, audio registers — предвыделенные `Vec<f32>` maximum block length. Index spaces и типы различны.
2. Codegen выделяет один register на node, последовательные independent `StateId` для `Sine` и отдельный output register. Ровно одна финальная `SetOutput` копирует output node; отдельный аргумент codegen передаёт ей полный out value origin.
3. `BinaryAudio` и `Sine` принимают typed `Operand::{Scalar,Audio}`; ordinary codegen сохраняет explicit lifts и использует audio operands.
4. Structural verifier проверяет ranges, initialization, operand types, finite constants, state ownership/coverage и output protocol до VM creation, храня только фактически встреченные indices без allocation размером в declared counts.
5. `debug_origins = Some` содержит один валидный origin на instruction; `None` допустим. Origin verification использует исходную `SourceMap`. Stripping не меняет execution.

Подробности: ADR 0005.

### 5.4. VM, diagnostics и WAV

1. VM creation проверяет arithmetic overflow и Vec capacity limits до verifier metadata/runtime allocations, проверяет config/structural bytecode и выделяет registers/buffers/state. Maximum block size равен `RenderConfig::block_size`.
2. `render_into([])` — no-op; `1..=block_size` допустимо; больший block — typed error до dispatch. Absolute sample index сохраняется между calls.
3. Dispatch выполняет instructions в Vec order для каждого sample, затем переходит к следующему sample. `Sine` использует ADR 0006 и production `dsp::Sine`; nonfinite frequency не меняет его phase.
4. VM errors содержат structured instruction/state/sample metadata, но не source `Diagnostic`. Renderer с retained origins создаёт прежние `E4001`; structured `NonFiniteOutput` на `SetOutput` локализуется полным out value span до перехода к следующему sample.
5. Reset сбрасывает registers/states/index. Steady-state не делает allocation/I/O/logging.
6. Renderer использует один scratch buffer вне loop и переменные final chunks. Protective limit, streaming header, PCM16, sample count и atomic publish не меняются.
7. Internal IR/bytecode invariant возвращается как typed compile/render error и даёт CLI exit `3`, не source diagnostic и не panic/101.

### 5.5. Compatibility и parity

1. До удаления `NaiveInterpreter` VM сравнивается с ним и independent reference на одном corpus. Основные expected остаются аналитическими.
2. Top-level unused binding с nonfinite frequency по-прежнему даёт `E4001` в declaration order.
3. Все block sizes `1`, `3`, `64`, `128`, `256`, empty calls и fixed-seed variable partitions дают эквивалентный output и одинаковые первые runtime diagnostics с code/span/sample index; чистая арифметика сравнивается точно, sine — с допуском ADR 0009.
4. Reset повторяет samples и первый ошибочный sample index. Debug-stripped/regular programs совпадают посемплово.
5. Физический путь, hash iteration, SourceId и debug info не влияют на graph/codegen order или звук.

## 6. Порядок production-реализации человеком

1. **IR lowering:** canonical nodes/roots/sharing/origins. Узкая проверка: `cargo test --test ir_verify --locked`.
2. **IR verifier:** structural/type/root/reachability/origin checks и descriptors. Проверка: `cargo test --test ir_verify --locked`.
3. **Codegen и bytecode verifier:** register/state/output layout, initialization и debug origins. Проверка: `cargo test --test bytecode_verify --locked`.
4. **Block VM:** preallocation, scalar/audio dispatch, state, errors, variable blocks и reset. Проверки: `cargo test --test vm --locked`, `cargo test --test differential --locked`.
5. **Facade и renderer:** compile pipeline, structured internal errors, VM scratch loop, source mapping, удаление naive production interpreter после parity. Проверки: `cargo test --test audio --locked`, `cargo test --test wav --locked`, `cargo test --test cli --locked`, `cargo test --test compatibility --locked`.
6. **Properties и ресурсы:** fixed-seed corpus, block partitions, origins/capabilities, allocation-counting harness. Проверка: `cargo test --test properties --locked`.
7. **Приёмка:** удалить все временные `todo!()`, пройти узкие/общие проверки, выполнить real Beep/atomic/determinism/debug-strip сценарии, измерить documented baseline и удалить generated artifacts.

Человек может менять нестабильные Rust-сигнатуры только с одновременной адаптацией tests без ослабления assertions и с обновлением task document.

## 7. Матрица проверки

| Требование | Positive/negative/boundary cases | Независимый oracle |
|---|---|---|
| §5.6, ADR 0004 — lowering | Explicit node graphs, aliases, shared state, groups, lifts, roots | Явно ожидаемые nodes/IDs/spans |
| IR verifier | Unknown/forward/cyclic inputs, wrong ID/type, nonfinite const, roots/output/reachability/origin corruption | Вручную повреждённые graphs и exact error variants |
| Capabilities/resources | Каждый op и `sine`, zero/one state slots | Явная таблица ADR 0004 |
| §5.7, ADR 0005 — codegen | Register counts/types, state IDs, output, optional origins, полный out span у alias | Явный instruction layout и exact offsets известной program |
| Bytecode verifier | Unknown/uninitialized/mistyped regs, state/output/origin corruption, экстремальные declared counts | Вручную повреждённые bytecode и exact variants |
| VM scalar/audio | Const, negation, four ops, scalar/audio operands, lift, structured nonfinite output | Аналитические `f64`/`f32` vectors и sample indices |
| Sine/state | Analytic quarter period, two calls, nonfinite frequency, reset | ADR 0006 и independent reference |
| §7.4 — blocks | 1/3/64/128/256, empty calls, fixed variable partitions, конкурирующие frequency/output errors | Посемпловое сравнение и exact code/span/sample index первого diagnostic |
| §7.5 — differential | Valid expressions, sharing, unused roots, runtime errors | `test-support/reference`, analytic cases |
| Debug origins | Valid source spans, exact SetOutput span, strip regular/compact parity | SourceMap offsets и sample comparison |
| §10.1 — allocations | Repeated non-empty/empty steady-state calls; block/register/state size overflow | Allocation counter и typed `SizeOverflow` до allocation |
| Compatibility | Все прежние fixtures, CLI, diagnostics, WAV, review regressions | Старые expected без ослабления и новые explicit diagnostics |
| Determinism | Repeat compile/VM render/process, different logical paths | Structural graph/bytecode equality и samples |
| Benchmark | Naive sample-major VM baseline с fixed graph/config/environment | Измеренные measurements, не production-derived correctness |

Audio comparison использует `abs_tol = rel_tol = 2e-5`, первый mismatch включает sample index. Counts, register/state IDs, spans, enum errors и PCM bytes сравниваются точно.

## 8. Definition of Done этапа

- [x] Все Beep fixtures проходят без изменения expected; production interpreter удалён после parity.
- [x] VM совпадает с independent reference на согласованном corpus; analytic tests остаются зелёными.
- [x] Block sizes `1`, `3`, `64`, `128`, `256`, empty calls и fixed variable partitions эквивалентны по samples и первому runtime diagnostic.
- [x] Reset повторяет исходный render и runtime-error index.
- [x] IR/bytecode verifiers отклоняют все повреждённые invariants до dispatch без panic; экстремальные sizes возвращают typed error.
- [x] Каждый source IR node имеет валидный origin; `SetOutput` сохраняет полный out value span; stripping debug info не меняет звук.
- [x] Каждый IR op имеет capability/resource classification, согласованную с `sine` descriptor.
- [x] Steady-state `render_into` имеет доказанные 0 heap allocations; runtime sizes проверены до allocation.
- [x] Unused bindings и их runtime-error ordering сохранены через roots и sample-major dispatch.
- [x] Beep CLI, diagnostics codes/spans/sample index и WAV bytes/conversion не изменились.
- [x] Нет временных заглушек, ignored tests, скрытых fallback, persisted/stable ABI или future-stage infrastructure.
- [x] Повторный compile/render детерминирован в одном и разных processes; physical path/hash iteration не влияют.
- [x] Benchmark baseline измерен с commit/toolchain/CPU/OS/profile/sample rate/block sizes/graph/duration/allocations/realtime factor.
- [x] Узкие suites и общий набор действительно запущены и прошли.
- [x] README/changelog/development docs описывают только фактически работающее состояние.

## 9. Обязательные команды приёмки

После production-реализации и удаления заглушек:

```bash
cargo test --test ir_verify --locked
cargo test --test bytecode_verify --locked
cargo test --test vm --locked
cargo test --test differential --locked
cargo test --test properties --locked
cargo test --test compatibility --locked
cargo test --test audio --locked
cargo test --test wav --locked
cargo test --test cli --locked
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
cargo test --doc --locked
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked
```

Дополнительно выполняются real Beep через VM, repeat/reset/block partition/debug stripping, runtime errors, atomic output, independent processes и allocation-counting harness. Для benchmark фиксируется вся конфигурация из §10.2. Сгенерированные WAV/temp/log artifacts удаляются после проверки.

## 10. Фактические результаты

### Подготовительная задача агента — 2026-11-20

- Спецификация обновлена до 0.4.0; приняты ADR 0004/0005. Evaluation roots всех top-level bindings, canonical IDs/order, typed registers/state/output, optional origins, structured internal errors и block/reset contracts зафиксированы до production-реализации.
- Добавлены механические IR/bytecode/VM representations, capability/resource metadata, config validation, debug stripping, compile/facade seams и ровно семь разрешённых `todo!()`: `lower`, IR verify, codegen, два bytecode verify entry points, `Vm::new` и `Vm::render_into`.
- Добавлены 52 integration tests: `ir_verify` 13, `bytecode_verify` 9, `vm` 14, `differential` 6, `properties` 5, `compatibility` 5. Общий integration corpus теперь содержит 173 tests. Добавлены три stage-3 fixtures и независимый fixed-seed source generator без новых dependencies.
- Все новые `--no-run` команды и `cargo test --all-targets --locked --no-run` успешны. `cargo fmt --check`, `git diff --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --doc --locked` (1 passed) и rustdoc с `-D warnings` успешны.
- Обычный `cargo test --all-targets --locked` ожидаемо останавливается на `audio`: 3 passed, 6 failed. Полный `--no-fail-fast` дал 99 passed, 74 failed, 0 ignored. 71 failure непосредственно достигает разрешённых заглушек; три CLI parent assertions вызваны таким же lowering panic в child process. Это не зелёная приёмка и не подтверждение production correctness.
- Production lowering/verifiers/codegen/VM, VM renderer, real Beep через VM, allocation-free proof и benchmark не реализованы и не проверены. Allocation-counting harness требует отдельного test-only unsafe/dependency решения. Наивный production interpreter пока сохранён, но valid compile до него не доходит из-за явной lowering заглушки.

Следующий шаг — ручная реализация по разделу 6 и task-facing API, удаление всех семи заглушек, зелёный 173-test corpus и полный DoD. Финальная production-приёмка добавляется отдельным разделом без переписывания этих результатов.


### Production-реализация и приёмка без benchmark — 2026-11-21

- После ручной реализации lowering/verifiers/codegen/VM все 173 исходных integration tests стали зелёными; three-way parity naive/VM/reference подтверждён до удаления naive production interpreter.
- Renderer переведён на VM с одним preallocated scratch buffer и variable final chunks. `E4001` frequency сохраняет code/span/sample index через retained instruction origins; nonfinite output использует полный out span. VM creation/runtime invariants возвращают structured internal errors и CLI exit `3`.
- Добавлены tests для missing debug origin после stripping и VM creation error до записи. `src/render/interpreter.rs` удалён; audio/reference/differential tests используют production VM и independent reference.
- Properties, required block partitions, reset, origins, capabilities/resources и size overflow проходят. Разрешённый пользователем test-only `GlobalAlloc` counter в `tests/allocations.rs` доказал 0 heap allocations для steady-state `render_into` с variable/empty blocks и `reset`.
- Для harness `Cargo.toml` использует `unsafe_code = "deny"`; production library и binary явно сохраняют `#![forbid(unsafe_code)]`. Новых dependencies нет.
- Узкие `audio`, `wav`, `cli`, `compatibility`, `differential`, `properties`, `vm`, `ir_verify` и `allocations` проверки прошли. Общий `cargo test --all-targets --locked` прошёл: 176 integration tests, 0 failed, 0 ignored. `cargo fmt --check`, clippy `-D warnings`, doctest и rustdoc `-D warnings` успешны; `git diff --check` успешен.
- Реальный `cargo run -- run fixtures/valid/stage-01-beep.ln` через production VM создал mono PCM16 WAV на 96 000 samples и 192 044 байта с правильными RIFF sizes. Сгенерированный WAV удалён.
- Benchmark baseline с полной конфигурацией из §10.2 не измерен. Поэтому этап 3 не объявляется завершённым, несмотря на зелёный функциональный и allocation набор.


### Benchmark baseline — 2026-11-21

Измерена наивная VM без оптимизаций. Это локальный baseline, а не универсальная метрика или performance gate.

Программа сохранена как `fixtures/valid/stage-03-benchmark.ln`:

```linen
let modulator = sine(5);
let out = sine(440 + modulator * 20) * 0.1;
render 60s;
```

Конфигурация и окружение:

- базовый commit: `ffd7e302cc2fb21d0a8f9d72a50dead9c2b21c54`; рабочее дерево содержало незакоммиченные изменения этапа 3 (`dirty`), отдельный benchmark commit не создавался;
- toolchain: `rustc`/`cargo` 1.99.1, `1.99.1-x86_64-unknown-linux-gnu` из `rust-toolchain.toml`;
- CPU: AMD Ryzen 9 7945HX, 16 cores/32 threads; OS: Linux `7.3.3-1-cachyos`, x86_64;
- build: `cargo build --release --locked`, стандартный optimized release profile;
- sample rate: 48 000 Гц; duration: 60 с; output: 2 880 000 samples, mono PCM16 WAV на 5 760 044 байта;
- graph: 13 IR nodes, 14 bytecode instructions, 4 scalar registers, 10 audio registers, 2 state slots;
- allocations: 0 heap allocations в steady-state `render_into` после initialization по отдельному allocation-counting test; это не утверждение о нулевых allocations всего compile/CLI процесса;
- `/usr/bin/time` отсутствовал. Измеритель использовал Python standard library `time.perf_counter()` для wall time и `resource.getrusage(RUSAGE_CHILDREN)` для user/system time;
- для каждого block size выполнен один warm-up, затем пять measured runs с atomic overwrite одного WAV. Ниже — median wall time, min–max и `60 / median_seconds`.

| Block size | Median wall | Min–max wall | Offline realtime factor |
|---:|---:|---:|---:|
| 1 | 5.592 с | 5.516–5.933 с | 10.73× |
| 3 | 5.440 с | 5.310–5.566 с | 11.03× |
| 64 | 5.236 с | 5.205–5.357 с | 11.46× |
| 128 | 5.203 с | 5.132–5.239 с | 11.53× |
| 256 | 5.173 с | 5.017–5.240 с | 11.60× |

Для default block size 128 медианные user/system components составили примерно 0.695 с и 4.505 с соответственно. Максимальный RSS, наблюдаемый через `RUSAGE_CHILDREN`, — 31 428 KiB.

System time существенно превышает user time на всех конфигурациях, а block size почти не влияет на wall time. Это согласуется с текущей записью каждого PCM16 sample отдельным `write_all`, но точный call-level профиль этим baseline не измерялся. Буферизация PCM-записи вынесена в [`../backlog.md`](../backlog.md) как BUF-001 и требует отдельной задачи с повторным differential/functional набором и before/after измерением.

Все пункты DoD этапа 3 теперь выполнены. Этап 4 не начат и требует отдельной явной задачи.

### Исправления после read-only review — 2026-11-21

- Instruction-major dispatch изменял первый runtime diagnostic при смене block size. Dispatch переведён на sample-major порядок instructions; `SetOutput` возвращает structured `NonFiniteOutput { sample_index }` до перехода к следующему sample. Renderer преобразует его в прежнюю source diagnostic с полным out span. Frequency errors текущего sample сохраняют приоритет над output error, как в посемпловом interpreter/reference.
- Добавлены regression fixtures `stage-03-runtime-frequency-order.ln` и `stage-03-runtime-output-order.ln`. Для block sizes `1`, `3`, `64`, `128`, `256` точно проверяются code, SourceId, primary byte span, message и sample index первого diagnostic. Реальные release CLI runs для block sizes `1` и `128` дали побайтово одинаковый stderr в обоих сценариях и не опубликовали WAV.
- Structural bytecode verifier больше не выделяет `Vec` размером в declared register/state counts; initialization/state tracking использует только фактически встреченные indices. `Vm::new` проверяет arithmetic overflow и Vec capacity limits до verifier/runtime allocations. Ручные corruption cases для всех count-fields возвращают `VmCreateError::SizeOverflow` без capacity-overflow panic.
- Codegen получает отдельный `NodeOrigin` полного out value для `SetOutput`. Regression fixture `stage-03-set-output-origin.ln` проверяет exact span `32..36` у alias; frequency origin `Sine` и прежняя `E4001` локализация сохранены.
- Добавлены четыре integration tests: два runtime-order regressions, structured nonfinite output и exact SetOutput origin. Текущий corpus — **180 integration tests, 0 failed, 0 ignored**. Allocation harness по-прежнему подтверждает 0 heap allocations в steady-state `render_into`/`reset`.
- Узкие `compatibility`, `vm`, `bytecode_verify`, `audio` suites и общий `cargo test --all-targets --locked` прошли. `cargo fmt --check`, `git diff --check`, clippy `-D warnings`, doctest и rustdoc `-D warnings` успешны. SPEC, ADR 0005, task-facing API, README и changelog приведены в соответствие с реализацией.

#### Актуальный benchmark после review-исправлений

Предыдущая таблица сохранена как исторический instruction-major baseline. Повторно измерена исправленная sample-major VM; эта таблица является актуальным локальным baseline, а не performance gate.

Конфигурация совпадает с разделом выше: базовый commit `ffd7e302cc2fb21d0a8f9d72a50dead9c2b21c54` (`dirty`), rustc/cargo 1.99.1, AMD Ryzen 9 7945HX, Linux `7.3.3-1-cachyos`, x86_64, `cargo build --release --locked`, 48 000 Гц, 60 с, 2 880 000 samples, 13 IR nodes, 14 instructions, 4 scalar/10 audio registers и 2 states. Для каждого block size выполнен один warm-up и пять measured runs; каждый run создал проверенный WAV на 5 760 044 байта. Измеритель использует `time.perf_counter()` и `RUSAGE_CHILDREN`.

| Block size | Median wall | Min–max wall | Offline realtime factor |
|---:|---:|---:|---:|
| 1 | 5.681 с | 5.516–5.755 с | 10.56× |
| 3 | 5.531 с | 5.432–5.618 с | 10.85× |
| 64 | 5.255 с | 5.195–5.273 с | 11.42× |
| 128 | 5.381 с | 5.189–5.579 с | 11.15× |
| 256 | 5.105 с | 5.086–5.271 с | 11.75× |

Для block size 128 медианные user/system components составили примерно 0.705 с и 4.664 с. Максимальный RSS через `RUSAGE_CHILDREN` — 15 304 KiB. System time остаётся доминирующим; PCM16 buffering сохранён как отдельная задача BUF-001. Различия с историческим instruction-major измерением не используются как statistically significant performance claim.

Все generated WAV/temp artifacts удалены. Этап 3 остаётся завершённым после review-исправлений; этап 4 не начат.

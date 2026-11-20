# Этап 2 — Типы, наивный интерпретатор и Beep WAV

- Статус: завершён 2026-11-20; production-реализация, regression-исправления и полная приёмка выполнены. Исторический отчёт подготовительной задачи сохранён в §10, итоговая приёмка находится в §11.
- Источник требований: `SPEC.md` 0.3.0, разделы 0, 2.1, 3–4.4, 4.8, 5.1–5.11, 6.1–6.5, 7.1–7.11, 8, 9 (этап 2), 10–14.
- Связанные ADR: `0001`, `0002`, `0003`, `0006`, `0009`, `0011`, `0012`.
- Экосистемные инварианты: `SourceId`/`SourceSpan`, `SourceMap`, явная session/symbol storage, resolved IDs, builtin descriptors, отсутствие скрытого I/O и детерминированный render.
- Блокирует: этап 3 — IR, bytecode и block VM.

## 1. Контекст

Этапы 0, 1 и 1A завершены. На момент начала задачи рабочее дерево чистое, `cargo test --all-targets --locked` успешно выполняет 69 integration tests. Lexer/parser создают syntax AST с source-aware spans, а `CompileSession` владеет `SourceMap` и `CompileConfig`.

Этап 2 должен впервые дать законченную вертикаль:

```linen
let out = sine(440) * 0.1;
render 2s;
```

```bash
linen run beep.ln --output beep.wav
```

Результат — mono PCM16 WAV, 48 000 Гц, ровно 96 000 семплов. До полной реализации и зелёной приёмки этот сценарий не считается работающим.

## 2. Ожидаемый результат после завершения

- `check` выполняет lexer/parser, name resolution, type checking и статическую проверку `render` с sample rate 48 000 Гц; успешный результат по-прежнему содержит syntax AST.
- `compile` получает явный `RenderConfig`, проверяет его и возвращает Typed AST с `BindingId`, `BuiltinId`, типами, явными lifts, `SourceSpan` и checked sample count.
- `CompileSession` и source-level compile outcome явно владеют `SymbolStorage`; повторный строковый lookup не используется как resolved identity.
- Статический `BuiltinDescriptor` для `sine` является единственным источником сигнатуры и содержит ID, имя, parameters/result, purity/determinism, statefulness и documentation name.
- Наивный interpreter вычисляет Typed AST по одному семплу. Каждый `sine` имеет независимую `f64` phase и resettable state.
- `render` проверяет protective duration limit, выявляет NaN/Inf с фактическим sample index и потоково пишет RIFF/WAVE PCM16 через `Write`, не храня весь output.
- CLI `run` поддерживает `--output`, `--sample-rate`, `--block-size`, `--max-seconds`, `--force` и atomic publish через temporary file в той же директории.
- Ни invalid source, ни runtime error, ни I/O error не оставляют видимого частичного итогового WAV.
- Все обязательные tests из `SPEC.md` §7.3 проходят; предыдущие syntax/source contracts сохранены, кроме документированного расширения `check` семантикой этапа 2.

## 3. Объём и allowlist подготовительной задачи

Пользователь явно разрешил агенту подготовить SDD-документы, механический test-facing каркас, временные `todo!()` в обозначенных entry points, contract tests, fixtures и test-support reference. Основной resolver/type checker/interpreter/DSP/WAV пишет человек.

### Документация

- `SPEC.md`, `CHANGELOG.md`, `README.md`, `AGENTS.md`;
- `docs/adr/**`;
- `docs/development/**`.

### Production-каркас

- `src/api.rs`, `src/diagnostic.rs`, `src/lib.rs`, `src/main.rs`;
- `src/syntax/**` только при необходимости сохранить уже существующий span contract;
- `src/sema/**`, `src/dsp/**`, `src/render/**`.

### Tests и oracles

- `tests/**`, `fixtures/**`, `test-support/reference/**`.

Дополнительное явное согласование разрешает только §13 `docs/linen — подготовка к экосистеме.md` для синхронизации переноса FunctionId/nested scopes на этап 4; остальная roadmap не изменяется.

`Cargo.toml`, `Cargo.lock`, toolchain и dependencies не входят в задачу. `src/ir/**`, `src/bytecode/**`, `src/vm/**` и module/package infrastructure не изменяются. Дальнейшее расширение allowlist требует отдельного согласования.

## 4. Нецели

- IR, `NodeOrigin`, bytecode, registers, block VM, verifiers и stable bytecode ABI.
- Пользовательские функции, `FunctionId`, nested scopes, shadowing, блоки-выражения и pipeline.
- Осцилляторы кроме `sine`, фильтры, stereo, noise, events, feedback и probes.
- Parser recovery и несколько diagnostics за один `check`.
- `inspect`, real-time, allocations-free VM steady-state и benchmark promises.
- Modules/imports/exports, packages, manifest, registry, network, plugins и device artifacts.
- Новые dependencies, WAV/DSP libraries, повышение toolchain.
- Объявление этапа завершённым при наличии заглушек, failing tests или невыполненного DoD.

## 5. Принятые контракты реализации

### 5.1. Name resolution и Typed AST

1. Top-level declarations обрабатываются слева направо. Duplicate name проверяется среди предыдущих declarations до разбора значения текущего binding; текущий binding не входит в scope собственного value. Поэтому self/forward reference — `E2001`.
2. Вызов сначала разрешает callee, затем проверяет arity, затем аргументы слева направо. После value проверяется аннотация binding; `render` проверяется после всех bindings.
3. `BindingId` выдаются в порядке declarations начиная с нуля; `BuiltinId` индексирует статическую таблицу. Одинаковое имя не является глобальной identity.
4. На этапе 2 один плоский scope. FunctionId/nested-scope tests перенесены в этап 4 вместе с реальным синтаксисом.
5. Typed AST содержит тип каждого expression, resolved IDs и все `SourceSpan`. `Group` может сохраняться для точного source/runtime origin и не меняет тип.
6. Каждый Float→Sig conversion представлен `LiftFloatToSig`; interpreter не восстанавливает implicit conversion по имени или контексту.

Подробности: ADR 0003.

### 5.2. Float, duration и render limits

1. Literal и compile-time arithmetic обязаны давать конечный `f64`, иначе `E2105`. Конечный `f64` может превышать `f32` до момента lift.
2. При lift finite `f64` переводится в `f32`; бесконечный результат conversion — `E2105`. IEEE rounding и underflow к нулю допустимы.
3. `Float / Float` с нулевым divisor — `E2104` до общей проверки результата. Dynamic signal division не отклоняется при компиляции.
4. `render` после учёта unit и optional minus обязан быть конечным и строго положительным. Нулевой/negative/nonfinite и rounded-to-zero результат — `E2103`.
5. `round(duration_seconds × sample_rate)` использует округление половин от нуля. Mathematical result должен помещаться в `u64`, а PCM data и RIFF chunk sizes — в их 32-bit fields; overflow — `E4002`.
6. `check` использует 48 000 Гц для sample-count validation. `run` использует выбранный `--sample-rate`.
7. `max_output_seconds` — только protective render limit. Фактическая positive duration больше лимита даёт `E4003`; `check` лимит не применяет. Лимит `0` валиден и отклоняет любую программу перед записью.
8. Sample rate находится в `8000..=384000`. На этапе 2 любой nonzero `block_size`, представимый как `usize`, допустим; посемпловый interpreter не меняет алгоритм от его значения.

### 5.3. Diagnostics и exit codes

1. Существующие lexer/parser/structural codes и spans не меняются. Semantic fail-fast порядок определяется §5.1 и `SPEC.md` §6.2.
2. `E2000` указывает на второй identifier и первый secondary; `E2001`/`E2102` — на unresolved identifier/callee; `E2101` — на callee; `E2104` — на zero divisor.
3. Type mismatch в аргументе указывает на argument. Обычная несовместимая аннотация — primary value + secondary annotation. Запрещённая `out: Float` аннотация — primary annotation.
4. `E2103`, `E4002`, `E4003` указывают на duration literal без optional minus. `E2105` указывает на literal/operation/lifted expression, значение которого отклонено.
5. `E4001` на output указывает на value `out`; nonfinite sine frequency — на frequency argument. `Diagnostic` содержит optional structured `sample_index`, обязательный для `E4001` и отсутствующий у остальных текущих codes.
6. Source/type/render errors возвращают exit `1`. CLI usage, invalid numeric render configuration (`E4000`) и host I/O (`E5000`) — exit `2`. Internal invariant — exit `3`, не panic/101.
7. Source-free config/I/O errors не получают искусственный `SourceSpan`. Source diagnostics продолжают разрешаться через исходную `SourceMap` и logical `file_name`.

### 5.4. Runtime и WAV

1. `sine` следует ADR 0006: phase `f64`, начальный `0`, output до phase increment, Euclidean modulo `[0,1)`, state на каждый вызов.
2. Нечисловая frequency даёт `E4001` до изменения phase. Нечисловой output также даёт `E4001`; оба содержат фактический zero-based sample index.
3. `reset()` сбрасывает все oscillator states. Повторный рендер после reset воспроизводит последовательность.
4. WAV — базовый 44-byte RIFF/WAVE header, PCM tag 1, mono, 16-bit, little-endian, sample rate из config.
5. Каждый finite sample clamp в `[-1,1]`, затем `(value × 32767).round()` с половинами от нуля и conversion в `i16`. Dithering отсутствует.
6. Запись использует `Write` и не накапливает все samples. CLI сначала создаёт temporary file в output directory, удаляет его при ошибке и делает rename только после полного успеха. `--force` необходим для перезаписи существующего destination.
7. Успешный `run` печатает output path, samples, sample rate и duration. Успешный `check` молчит; diagnostics — stderr.

## 6. Порядок production-реализации человеком

1. **Resolver и symbol storage:** declarations/IDs/scopes и `E2000`–`E2001`. Узкая проверка: `cargo test --test sema --locked`.
2. **Types и builtins:** descriptor-driven `sine`, Typed AST, lifts, annotations, operators, constant checks и duration. Проверка: `cargo test --test sema --locked`.
3. **Sine и interpreter:** per-sample evaluation, state ownership, reset и runtime diagnostics. Проверки: `cargo test --test audio --locked`, `cargo test --test reference --locked` при наличии отдельного target.
4. **WAV и render:** checked sizes, streaming header/samples, protective limit, I/O propagation. Проверка: `cargo test --test wav --locked`.
5. **Facade и CLI:** подключить sema к `check`, `compile`/`render` к `run`, аргументы, atomic output и отчёты. Проверка: `cargo test --test cli --locked`.
6. **Приёмка:** удалить все временные `todo!()`, пройти узкие и общие проверки, выполнить реальный Beep, сравнить independent renders/processes, удалить generated WAV и только затем обновить статусы/DoD.

Агентский test-support reference не подключается к production. Человек может менять нестабильные Rust-сигнатуры только с одновременной адаптацией tests без ослабления assertions и с обновлением task document.

## 7. Матрица проверки

| Требование | Positive/negative/boundary cases | Независимый oracle |
|---|---|---|
| Resolved IDs | Несколько bindings, repeated references, deterministic IDs | Явные indices и structured Typed AST |
| Unknown/duplicate/forward/self | Источник с каждым случаем и комбинированный fail-fast case | Код + вручную заданный `(SourceId,start,end)` |
| `Float`/`Sig` inference | Literal, `sine`, annotations, mixed binary/unary | Таблица правил §4.4 |
| Explicit lifts | `sine(1)`, `Sig = Float`, `out = Float`, оба направления mixed ops | Явно ожидаемые `LiftFloatToSig` nodes |
| Arity/unknown function/type mismatch | Ноль/два аргумента, unknown callee, `Float = Sig`, `out: Float` | Сигнатура descriptor + exact spans |
| Constant arithmetic | Finite f64, nonfinite literal/result, f32 overflow, underflow, zero division | Независимые `f64` вычисления и IEEE conversion rules |
| Duration | Seconds/ms, `0`, negative, infinity, rounded zero, u64/RIFF overflow | Формула round-half-away-from-zero и явные bounds |
| Builtin metadata | Единственный `sine`, все обязательные fields, signature reuse | Явная descriptor table из спецификации |
| Typed AST spans | Recursive traversal всех nodes/IDs/lifts/render | Исходные byte offsets и один `SourceId` |
| Sine | Индексы `0,25,50,75,100` при 480 Hz/48 kHz | Значения `0,1,0,-1,0`; допуск ADR 0009 |
| Другие frequency cases | Zero, negative, above Nyquist, long render | Независимая phase recurrence/аналитическая формула |
| Gain/reset | `×0.1`, два рендера после reset | Посемпловые values; без RMS-only assertion |
| Runtime nonfinite | Dynamic division и nonfinite sine frequency | Первый invalid index, exact code/span/sample_index |
| Reference | Те же аналитические vectors и supported Typed AST | Отдельная test-support реализация без production DSP |
| WAV header | Известные sample rate/count | Полный вручную записанный 44-byte array |
| PCM16 | `-1,-0.5,0,0.5,1`, clipping, NaN/Inf | Вручную рассчитанные signed values и LE bytes |
| Beep WAV | 2 seconds, 48 kHz, 96 000 samples | Размер `44 + 2×96000`, independent sample vectors |
| CLI run | Required/unknown options, valid report, force, config errors | Exit/stdout/stderr и файловое состояние |
| Atomic output | Syntax/type/runtime/I/O failure | Destination отсутствует или прежнее содержимое сохранено |
| Determinism | Повторный compile/render и independent processes | Structural equality и посемпловое/byte equality там, где применимо |
| Compatibility | Все прежние tests и fixtures | Старые expected; `check` теперь намеренно отклоняет deferred semantics |

## 8. Definition of Done этапа

- [x] Beep CLI создаёт mono PCM16 WAV ровно на 96 000 семплов; частота и усиление проверены численно.
- [x] Все обязательные Beep tests из `SPEC.md` §7.3 реализованы и проходят.
- [x] Typed AST сохраняет resolved `BindingId`/`BuiltinId`, типы, lifts и валидные `SourceSpan`.
- [x] `sine` использует полный descriptor без повторного перечисления сигнатуры в unrelated matches.
- [x] Name/type/duration/Float diagnostics имеют документированные code/span/order; runtime errors содержат точный sample index.
- [x] `sine` state, reset, negative/zero/high frequency и длинный рендер проверены.
- [x] WAV streaming, PCM conversion, RIFF sizes и atomic CLI output проверены.
- [x] Нет временных заглушек, `todo!()`, `unimplemented!()`, ignored tests, скрытых fallback и production/test общего oracle.
- [x] Нет FunctionId/nested scopes/IR/VM или других future-stage возможностей.
- [x] Повторный render детерминирован и не зависит от physical path/hash iteration/block size.
- [x] Все прежние fixtures сохраняют контракт, кроме документированного включения semantics в `check`.
- [x] Узкие suites и общий набор действительно запущены и прошли.
- [x] README/AGENTS/changelog описывают только фактически работающее состояние.

## 9. Обязательные команды приёмки

После production-реализации и удаления заглушек:

```bash
cargo test --test sema --locked
cargo test --test audio --locked
cargo test --test wav --locked
cargo test --test cli --locked
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
cargo test --doc --locked
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked
```

Дополнительно вручную выполняются `linen check` для valid/invalid programs, `linen run beep.ln --output beep.wav`, повторный run в другой directory с тем же logical input, проверка header/sample count и сценарии `--force`, runtime error и max-seconds. Сгенерированные WAV и temporary files удаляются после проверки.

## 10. Фактические результаты

Подготовительная задача агента завершена 2026-11-17. Это не приёмка production-этапа.

- Спецификация обновлена до 0.3.0; приняты ADR 0003/0006/0009. Semantic spans/order, Float/duration/RIFF limits, `E2105`/`E4003`, structured sample index, плоские stage-2 IDs и CLI atomic contract зафиксированы до реализации. §13 ecosystem roadmap синхронизирован без добавления module/package functionality.
- Созданы Typed AST/descriptors/storage/config/facade и CLI host boilerplate. Ровно семь разрешённых временных заглушек находятся в `sema::check_program`, `Sine::next_sample`, `NaiveInterpreter::{new,next_sample}`, `render`, `write_wav_header` и `quantize_pcm16`. Других `todo!()`, `unimplemented!()`, ignored tests и фиктивных успехов нет.
- Добавлены 11 fixtures и независимый test-support reference. Всего в integration suites 116 tests: `audio` 9, `cli` 23, `diagnostics` 8, `lexer` 13, `parser` 23, `reference` 3, `sema` 17, `source` 14, `wav` 6.
- `cargo test --all-targets --locked --no-run` — успешно. Узкие `source`, `lexer`, `parser`, `diagnostics` suites зелёные: 14 + 13 + 23 + 8 passed, 0 failed/ignored. Reference analytic/reset tests — 2 passed; production differential ожидаемо достигает sema-заглушку.
- `cargo fmt --check`, `git diff --check` и `cargo clippy --all-targets --locked -- -D warnings` — успешно. `cargo test --doc --locked` — 1 passed; `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked` — успешно.
- Обычный `cargo test --all-targets --locked` ожидаемо останавливается на `audio`. Полный дополнительный запуск с `--no-fail-fast` дал 77 passed, 39 failed, 0 ignored: `audio` 0/9, `cli` 14/23, `reference` 2/3, `sema` 3/17, `wav` 0/6; остальные suites полностью зелёные. 38 failures достигают явных заглушек, один показывает ещё не подключённую семантику `check`.
- Успешный production Beep и WAV, корректность красных expected на реальной реализации и полный DoD не проверены. `check` пока сохраняет прежний syntax-only behavior; `run` имеет argument/config/host boilerplate, но не создаёт звук. Все generated test/log artifacts удалены.

Следующий шаг — ручная production-реализация по разделу 6 и task-facing API. После неё нужно удалить все семь заглушек, пройти 116 tests и общий набор без `--no-fail-fast`, выполнить ручной Beep/atomic/determinism сценарии и только затем отметить DoD и этап завершённым.


## 11. Финальная production-приёмка — 2026-11-20

Этот раздел добавлен после реализации и ревью. Он не изменяет исторические результаты подготовительной задачи из §10.

- Реализованы resolver/type checker, Typed AST с resolved IDs/lifts/spans, descriptor-driven `sine`, checked Float/duration semantics, наивный interpreter, resettable DSP state, streaming PCM16 WAV, semantic `check`, `compile`/`render` и CLI `run` с atomic publish.
- По read-only review исправлены два дефекта: повторная semantic compilation теперь очищает `SymbolStorage` и выдаёт `BindingId` от нуля; publish без `--force` использует hard link на завершённый temporary file, поэтому broken symlink или destination, появившийся во время render, не перезаписывается.
- Добавлены постоянные regression fixtures `fixtures/regression/stage-02-repeated-session.ln` и `stage-02-atomic-output.ln` с tests, которые воспроизводили старые дефекты.
- Переработаны тексты lexer/parser/sema/runtime/CLI diagnostics: добавлены конкретные names, types, arity, numeric values и limits, устранены повторы message/primary label. Renderer теперь выводит secondary source locations и notes. Стабильные codes, spans, fail-fast order и sample index сохранены.
- Финальный integration-набор: 121 passed, 0 failed, 0 ignored. Breakdown: `audio` 9, `cli` 24, `diagnostics` 8, `lexer` 14, `parser` 23, `reference` 3, `sema` 20, `source` 14, `wav` 6.
- Узкие проверки `cargo test --test sema --locked`, `cargo test --test audio --locked`, `cargo test --test wav --locked` и `cargo test --test cli --locked` выполнены: 20 + 9 + 6 + 24 passed.
- Общий набор выполнен успешно: `cargo fmt --check`; `cargo clippy --all-targets --locked -- -D warnings`; `cargo test --all-targets --locked`; `cargo test --doc --locked` — 1 passed; `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked`.
- Ручной Beep создан через `linen run`: mono PCM16, 48 000 Гц, ровно 96 000 samples и 192 044 байта. Два independent processes в разных directories с одинаковым logical input дали одинаковые WAV bytes.
- Вручную проверены valid/invalid `check`, отказ overwrite без `--force`, успешный `--force`, runtime `E4001` и protective limit `E4003`. Ошибки не опубликовали итоговый WAV, temporary/generated files удалены.
- Временные заглушки, ignored tests и production/test общий oracle отсутствуют. FunctionId, nested scopes, `inspect`, IR, bytecode и VM не реализованы; этап 3 этим diff не начат.

Все пункты DoD из §8 выполнены. Следующая работа над IR/bytecode/block VM требует отдельной явной задачи по `SPEC.md` этапа 3.

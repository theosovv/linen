# Этап 2 — SDD, test-facing каркас и независимые contract tests

- Статус: историческая подготовительная задача агента завершена; последующая production-реализация и приёмка этапа 2 завершены 2026-11-20. Итоговый отчёт находится в основном stage document; результаты ниже относятся только к подготовительному diff.
- Исполнитель подготовительной задачи: агент.
- Основной production resolver/type checker/interpreter/DSP/WAV: человек.
- Основной stage document: `docs/development/stages/stage-02-types-interpreter-wav.md`.
- Связанные ADR: `0003`, `0006`, `0009`; базовые ограничения — `0001`, `0002`, `0011`, `0012`.

## Контекст

- Этап: 2, типы, наивный интерпретатор и Beep WAV.
- Пункты `SPEC.md` 0.3.0: 2.1, 3–4.4, 4.8, 5.1–5.11, 6.1–6.5, 7.1–7.11, 9 (этап 2), 10–14.
- Экосистемные инварианты: `SourceSpan`/`SourceMap`, session-owned symbol storage, resolved IDs, deterministic diagnostics/render, отсутствие скрытого I/O.
- Baseline: этап 1A завершён, дерево чистое, 69 integration tests зелёные.
- Пользователь прямо разрешил временные заглушки в этом подготовительном diff. Это исключение не распространяется на завершение этапа.

## Задача

1. Уточнить в `SPEC.md` semantic spans/order, Float/duration/render-limit contracts, новые `E2105`/`E4003` и stage-2 границы IDs/scopes.
2. Создать ADR 0003/0006/0009, stage document и этот проверяемый task.
3. Добавить механические типы Typed AST/descriptors/config/report и явные entry-point заглушки, сохранив существующие lexer/parser/source implementations.
4. Подготовить независимые sema/audio/WAV/CLI tests, fixtures и test-support reference.
5. Запустить проверки, честно записать expected failures и не объявлять этап завершённым.

## Разрешено менять

- `SPEC.md`, `CHANGELOG.md`, `README.md`, `AGENTS.md`;
- `docs/adr/**`, `docs/development/**`;
- `src/api.rs`, `src/diagnostic.rs`, `src/lib.rs`, `src/main.rs`, `src/syntax/**`;
- `src/sema/**`, `src/dsp/**`, `src/render/**`;
- `tests/**`, `fixtures/**`, `test-support/reference/**`.

Production-код разрешён только в перечисленных путях и только как механический каркас/CLI boilerplate, не основная реализация ядра. `Cargo.toml`, `Cargo.lock`, toolchain, dependencies и IR/bytecode/VM не входят в allowlist. Отдельно разрешена только §13 `docs/linen — подготовка к экосистеме.md` для синхронизации FunctionId/nested scopes с этапом 4; остальная экосистемная roadmap не изменяется.

## Нецели

- Реализация resolver/type checking/evaluation/DSP/WAV semantics агентом.
- FunctionId, nested scopes, пользовательские функции, блоки и pipeline.
- IR/bytecode/VM, capability/resource analysis и `inspect`.
- Скрытые fallback, placeholder success, `#[ignore]` и удаление failing tests.
- Зелёная приёмка этапа 2 в этом diff.

## Test-facing API

Это нестабильный механический Rust seam, а не новый языковой контракт. Он конкретизирует принятую архитектуру и позволяет писать integration tests до реализации. При изменении сигнатур человек адаптирует вызовы tests и этот документ, сохраняя независимые expected.

### Sema

```text
Type::{Float,Sig}
BindingId::index() -> usize
BuiltinId::{SINE,index()}

TypedProgram {
    bindings: Vec<TypedBinding>,
    output: BindingId,
    render: TypedRender,
    span: SourceSpan,
}
TypedBinding { id, name, annotation, value, span }
TypedExpr { kind, ty, span }
TypedExprKind::{
    Number,
    Identifier,
    Call,
    Unary,
    Binary,
    Group,
    LiftFloatToSig,
}
TypedRender { duration_seconds, render_samples, duration, span }

SymbolStorage::{get,len,is_empty}
BindingSymbol { id, name, ty, span }

BuiltinDescriptor {
    id, name, signature,
    purity, determinism, statefulness, documentation_name,
}
BUILTINS
builtin_descriptor(BuiltinId) -> Option<&'static BuiltinDescriptor>
check_program(&Program, &mut SymbolStorage, sample_rate)
    -> Result<TypedProgram, Diagnostic>
```

`BindingId` соответствует declaration index. `BuiltinId::SINE` равен нулю. `TypedRender::duration` сохраняет исходный literal/span, а checked `duration_seconds` и `render_samples` не пересчитываются tests production-функцией.

### Render config, facade и execution

```text
RenderConfig {
    sample_rate: u32,          // default 48000
    block_size: usize,         // default 128, stage 2: любое > 0
    max_output_seconds: u64,   // default 600
}
RenderConfig::validate() -> Result<(), RenderConfigError>

CompiledProgram { typed: TypedProgram, config: RenderConfig }
CompileOutcome {
    sources: SourceMap,
    symbols: SymbolStorage,
    result: Result<CompiledProgram, Diagnostic>,
}

compile(source, render_config)
    -> Result<CompileOutcome, RenderConfigError>
compile_with_config(logical_name, source, compile_config, render_config)
    -> Result<CompileOutcome, RenderConfigError>

CompileSession::check(&mut self, SourceId)
    -> Result<Result<Program, Diagnostic>, SessionError>
CompileSession::compile(&mut self, SourceId, RenderConfig)
    -> Result<Result<CompiledProgram, Diagnostic>, SessionError>
CompileSession::symbols() -> &SymbolStorage

render(&CompiledProgram, impl Write) -> Result<RenderReport, RenderError>
RenderReport { samples, sample_rate, duration_seconds }
RenderError::{Diagnostic,Io}

NaiveInterpreter::new(&TypedProgram)
NaiveInterpreter::reset()
NaiveInterpreter::next_sample(sample_index, sample_rate)
    -> Result<f32, Diagnostic>
```

`check` использует default `RenderConfig` только для duration/sample-count semantics и не применяет max-seconds. `compile` получает фактический config выбранного рендера. `RenderConfigError` остаётся source-free и не подделывает `Diagnostic`.

### DSP и WAV

```text
Sine::new()
Sine::phase() -> f64
Sine::reset()
Sine::next_sample(freq: f64, sample_rate: u32)
    -> Result<f32, NonFiniteFrequency>

write_wav_header(impl Write, sample_rate, samples)
    -> Result<(), WavError>
quantize_pcm16(value: f32) -> Result<i16, NonFiniteSample>
```

`Sine` не знает об AST/diagnostics. Interpreter добавляет source span/sample index. `quantize_pcm16` не выполняет clamp для NaN/Inf: nonfinite input — ошибка до conversion.

### Runtime diagnostic metadata

```text
Diagnostic::sample_index() -> Option<u64>
Diagnostic::with_sample_index(u64) -> Diagnostic
```

Поле обязательно для `E4001` и отсутствует у других текущих diagnostics. Renderer может показывать его как note; structured assertions не зависят от форматирования.

## Явно разрешённые временные заглушки

Только следующие entry points могут содержать `todo!("stage 2: ...")` до ручной реализации:

- `sema::check_program` — resolution/type/duration pipeline;
- `render::interpreter::NaiveInterpreter::{new,next_sample}` — execution;
- `dsp::oscillator::Sine::next_sample` — числовой state transition;
- `render::render` — orchestration/streaming;
- `render::wav::{write_wav_header,quantize_pcm16}` — WAV format/quantization;
- CLI `run` — только если аргумент/host boilerplate не реализован в подготовительном diff.

Механические constructors/getters/reset/config validation, descriptors, IDs и data representations должны быть реальными. Никакая заглушка не возвращает фиктивный успех, тишину или пустой WAV. `unimplemented!()` не используется. Все заглушки удаляются до завершения этапа.

## Контракт и независимые проверки

| Требование | Проверка | Независимый oracle |
|---|---|---|
| §5.5, §9 — resolved IDs/Typed AST | `tests/sema.rs`: declarations, references, types, lifts, recursive spans | Явные indices/tree/offsets |
| §4.3–4.4, §6.2 — names/types/durations | Positive/negative/boundary fixtures и inline cases | Exact code/SourceId/primary span; descriptor signature |
| §4.4, ADR 0003 — Float | Nonfinite f64, f32 overflow, finite intermediate, underflow | Независимые IEEE/f64 cases |
| §4.8, ADR 0006 — sine | `tests/audio.rs`: analytic quarter-period, gain, zero/negative/high frequency, long phase, reset | Аналитические values и формула, не production Sine |
| ADR 0009 — reference | `test-support/reference` и `tests/reference.rs` | Reference сам проходит analytic vectors; никаких production DSP calls |
| §4.4, §6.2 — runtime errors | Dynamic division/nonfinite frequency | Exact first sample index, code/span/structured sample_index |
| §6.3, ADR 0009 — WAV | `tests/wav.rs`: full header, quantization, clipping, LE, Beep length | Вручную рассчитанные byte arrays и counts |
| §6.1, §6.4 — CLI | Args, config, report, force, limit, atomic failures | Exit/stdout/stderr и filesystem assertions |
| §5.11, §7.3 — determinism/compatibility | Повторный compile/render/process; все старые tests | Structural equality и независимые prior expected |

Audio helper использует `abs_tol = rel_tol = 2e-5` и показывает первый mismatch. Expected не вычисляется production `Sine`, interpreter, duration или WAV-writer. `sample_index`, spans, counts и PCM bytes сравниваются точно. `#[ignore]` не добавляется.

## Критерии приёмки подготовительной задачи

- [x] Diff ограничен allowlist; основная семантика не реализована агентом.
- [x] SPEC/ADR/stage/task устраняют согласованные неоднозначности до tests.
- [x] Test-facing каркас компилируется; заглушки находятся только в разрешённых entry points.
- [x] Sema/audio/WAV/CLI/reference tests покрывают positive, negative и boundary cases с независимыми expected.
- [x] Для diagnostics проверяются code, SourceId, primary span и sample_index там, где требуется.
- [x] Нет ignored tests, новых dependencies, production-derived expected и скрытых fallback.
- [x] Прежние lexer/parser/source contracts не ослаблены; старый deferred-semantics test заменён ожиданием stage-2 `check`.
- [x] Все проверки запущены; ожидаемые failures и непроверенные сценарии явно перечислены.
- [x] Этап 2 не отмечен завершённым, README не обещает работающий звук.

Окончательный DoD находится в stage document и этим подготовительным чек-листом не заменяется.

## Обязательные команды подготовительной задачи

```bash
cargo test --test sema --locked --no-run
cargo test --test audio --locked --no-run
cargo test --test wav --locked --no-run
cargo test --test cli --locked --no-run
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
cargo test --doc --locked
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked
```

До production-реализации tests, достигающие явные `todo!()`, ожидаемо падают; `--no-run` должен доказать компиляцию всех новых targets. Полный test/clippy набор не может считаться пройденным при ожидаемых failures или warnings. Их результат фиксируется, но не скрывается.

## Отчёт об исполнении

- Подготовлены `SPEC.md` 0.3.0, ADR 0003/0006/0009, stage/task documents и точечная синхронизация §13 ecosystem roadmap. Дополнительно актуализированы индекс ADR, ADR 0012, README, AGENTS и development README.
- Добавлены реальные data representations и boilerplate: Typed AST, `BindingId`/`BuiltinId`, `SymbolStorage`, полный descriptor `sine`, `RenderConfig` с validation, compile/session outcomes, structured `sample_index`, CLI argument/host boundary и семь явно разрешённых `todo!()`. `unimplemented!()`, placeholder success и ignored tests отсутствуют.
- Добавлены `tests/sema.rs` (17 tests), `tests/audio.rs` (9), `tests/wav.rs` (6), `tests/reference.rs` (3), один новый diagnostic metadata test и 11 новых CLI tests; всего 116 integration tests вместе с прежними suites. Есть 11 stage-2 fixtures. `test-support/reference` не вызывает production DSP/interpreter и сам проходит analytic/reset vectors.
- `cargo test --all-targets --locked --no-run` — успешно, все targets компилируются.
- `cargo fmt --check` и `git diff --check` — успешно; `cargo clippy --all-targets --locked -- -D warnings` — успешно без предупреждений.
- `cargo test --doc --locked` — 1 passed; `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked` — успешно.
- `cargo test --all-targets --locked` ожидаемо останавливается на `audio`. Дополнительный `cargo test --all-targets --locked --no-fail-fast` выполнил все targets: 77 passed, 39 failed, 0 ignored из 116 integration tests. Breakdown: `audio` 0/9, `cli` 14/23, `diagnostics` 8/8, `lexer` 13/13, `parser` 23/23, `reference` 2/3, `sema` 3/17, `source` 14/14, `wav` 0/6.
- Все 39 failures ожидаемы: 38 достигают явные `todo!()` в sema/Sine/WAV/render, а `spec_9_stage_2_check_runs_name_and_type_semantics` показывает, что прежний `check` пока выполняет только parser. Признаков неправильных independent expected в зелёных частях не обнаружено; правильность red tests окончательно проверяется после production-реализации.
- End-to-end Beep, numerical production samples, actual WAV bytes и atomic successful render не проверены, поскольку ядро ещё не реализовано. CLI negative paths для аргументов/config/existing output работают; temporary test directories и log artifacts удалены.
- Подготовительная задача завершена. Основной resolver/type checker, production `check` semantics, interpreter/Sine, streaming WAV/render и последующая зелёная приёмка остаются за человеком. Этап 2 не завершён; финальный отчёт должен быть добавлен в основной stage document после удаления всех заглушек.

# linen

[![CI](https://github.com/theosovv/linen/actions/workflows/ci.yml/badge.svg)](https://github.com/theosovv/linen/actions/workflows/ci.yml)

**linen** — исследовательский некоммерческий проект статически типизированного функционального языка для описания аудиосигналов. Цель разработки — собственные компилятор, блочная виртуальная машина, DSP-узлы и инструменты исследования звука на Rust.

> Этапы 0, 1, корректирующий этап 1A, 2 и 3 завершены. По `SPEC.md` 0.4.0 работает production path `source → Typed AST → IR → bytecode → block VM → WAV`; naive interpreter удалён после parity. Все 180 integration tests и allocation-counting harness проходят. Benchmark baseline зафиксирован в stage document; этап 4 не начат.

## Что уже работает

- Rust library и binary в одном package без внешних зависимостей.
- `SourceId`, `SourceFile`, `SourceMap` и `SourceSpan` с полуинтервальными byte offsets и Source identity через tokens, AST, Typed AST и diagnostics.
- Предвычисленный line index с Unicode scalar columns, LF/CRLF, lone CR, tab и EOF boundaries.
- Lexer, AST и recursive descent parser для Beep-синтаксиса, включая лимиты глубины и числа узлов и обработку смешанных synthetic token streams.
- Name resolution в плоском top-level scope, детерминированные `BindingId`/`BuiltinId`, session-owned `SymbolStorage` и статический descriptor `sine`.
- Типы `Float`/`Sig`, checked compile-time arithmetic, явные `LiftFloatToSig`, аннотации и проверка `out`/`render`.
- Typed IR с evaluation roots/sharing/origins, graph verifier, typed register bytecode, bytecode verifier и capability/resource metadata.
- Блочная VM с preallocated scalar/audio registers, independent sine state, variable blocks, reset и structured runtime errors. Steady-state `render_into` и `reset` проверены allocation counter: 0 heap allocations.
- `sine` с независимой `f64` phase, Euclidean modulo и reset.
- Потоковая запись mono PCM16 RIFF/WAVE через `Write`, checked sizes, clipping и runtime diagnostics NaN/Inf с structured sample index.
- Структурированные diagnostics с кодами, primary/secondary labels, notes и logical source names.
- `linen check` с name/type/duration checks и `linen run` с `--output`, `--sample-rate`, `--block-size`, `--max-seconds` и `--force`.
- Atomic publish через temporary file в output directory; существующий destination не перезаписывается без `--force`.
- Integration/differential/property/compatibility tests, независимый reference interpreter, doctest, rustfmt, clippy и GitHub Actions CI.

`inspect`, пользовательские функции и блоки, stereo, фильтры и пользовательские multi-file projects ещё не реализованы. Несколько источников поддерживаются на уровне synthetic in-memory compiler tests; модульная и пакетная функциональность отсутствует.

## Требования

- [`rustup`](https://rustup.rs/) для автоматического выбора зафиксированного toolchain.
- Внешние Rust-зависимости на данный момент отсутствуют.

Версия Rust и компоненты указаны в [`rust-toolchain.toml`](rust-toolchain.toml). Не обновляйте их независимо от задачи на изменение инфраструктуры.

## Сборка и проверка

```bash
rustup show
cargo build --locked
cargo test --all-targets --locked
cargo test --doc --locked
```

Полный проверочный набор:

```bash
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
cargo test --doc --locked
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked
```

Полный набор должен проходить без ignored tests перед завершением каждого этапа.

## Текущий CLI

Проверка программы и создание Beep WAV:

```bash
cargo run -- check fixtures/valid/stage-01-beep.ln
cargo run -- run fixtures/valid/stage-01-beep.ln --output beep.wav
```

После установки binary доступны эквивалентные команды `linen check input.ln` и `linen run input.ln --output output.wav`. Успешный `check` молчит. Успешный `run` печатает output path, число samples, sample rate и duration. Beep из примера создаёт файл размером `44 + 2 × 96000 = 192044` байта.

Дополнительные опции `run`:

```bash
linen run input.ln --output output.wav --sample-rate 48000 --block-size 128
linen run input.ln --output output.wav --max-seconds 600 --force
```

`--force` обязателен для перезаписи существующего destination. При source, runtime или I/O ошибке temporary file удаляется, а прежний destination сохраняется. Source diagnostic возвращает код `1` и выводится в stderr; usage/config/I/O errors возвращают код `2`; нарушение внутреннего renderer invariant — код `3`. Source diagnostics используют logical `file_name`, а не absolute path.

## Библиотечный API sources и diagnostics

```rust
use linen::{Diagnostic, DiagnosticLabel, ErrorCode, SourceMap, SourceSpan};

fn main() {
    let mut sources = SourceMap::new();
    let source = sources.add_source("input.ln", "let out = missing; render 1s;");
    let span = SourceSpan::new(source, 10, 17).expect("this identifier span is valid");
    let diagnostic = Diagnostic::error(
        ErrorCode::UnknownName,
        "unknown name `missing`",
        DiagnosticLabel::new(span, "no previous binding defines `missing`"),
    );

    assert_eq!(diagnostic.code().as_str(), "E2001");
    assert_eq!(diagnostic.primary().span(), span);
}
```

`SourceSpan` — `SourceId` и полуинтервальный диапазон `[start, end)`. `SourceMap` выдаёт map-local IDs в порядке добавления и разрешает источники только по ID, допуская duplicate logical names внутри одной карты. Независимые карты могут выдавать равные ID: используйте ID только с создавшей его картой или её идентичной копией.

`check(source)` создаёт одноисточниковую session и возвращает `CheckOutcome` с `SourceMap` и syntax AST либо diagnostic. `compile(source, RenderConfig)` возвращает `Result<CompileOutcome, RenderConfigError>`; `outcome.result` содержит `Result<CompiledProgram, CompileError>`, а программа включает Typed AST, verified IR и bytecode. `CompileError` различает source diagnostic и internal verifier invariant. `create_vm(&program)` создаёт предвыделенную блочную VM, а `render(&program, writer)` потоково записывает WAV через VM в переданный `Write`.

При session-based API caller сохраняет `CompileSession` и не передаёт IDs между sessions. Semantic compilation начинается с пустого `SymbolStorage`; typed results следует разрешать до следующей semantic compilation в той же session. Source-level outcomes сохраняют собственные sources и symbols, поэтому их независимые результаты можно удерживать одновременно.

## Структура

```text
linen/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── SPEC.md
├── src/
│   ├── lib.rs
│   ├── main.rs
│   ├── source.rs
│   ├── api.rs
│   ├── diagnostic.rs
│   ├── syntax/
│   │   ├── mod.rs
│   │   ├── token.rs
│   │   ├── lexer.rs
│   │   ├── ast.rs
│   │   └── parser.rs
│   ├── sema/
│   │   ├── mod.rs
│   │   ├── resolve.rs
│   │   ├── types.rs
│   │   ├── builtins.rs
│   │   └── check.rs
│   ├── ir/                 # typed DAG, lowering и verifier
│   ├── bytecode/           # typed instructions, codegen и verifiers
│   ├── vm/                 # preallocated block execution
│   ├── dsp/
│   │   ├── mod.rs
│   │   └── oscillator.rs
│   └── render/
│       ├── mod.rs
│       └── wav.rs
├── tests/
│   ├── source.rs
│   ├── cli.rs
│   ├── diagnostics.rs
│   ├── lexer.rs
│   ├── parser.rs
│   ├── sema.rs
│   ├── ir_verify.rs
│   ├── bytecode_verify.rs
│   ├── vm.rs
│   ├── allocations.rs
│   ├── differential.rs
│   ├── properties.rs
│   ├── compatibility.rs
│   ├── audio.rs
│   ├── wav.rs
│   └── reference.rs
├── test-support/
│   └── reference/
├── fixtures/
│   ├── valid/
│   ├── invalid/
│   └── regression/
├── docs/
│   ├── linen — подготовка к экосистеме.md
│   ├── adr/
│   └── development/
└── .github/
    ├── ISSUE_TEMPLATE/
    └── workflows/ci.yml
```

Production-заглушек этапа 3 не осталось. Независимый reference находится только в `test-support` и не используется production VM или DSP как общий oracle. Test-only allocation counter изолирован в `tests/allocations.rs`; production library/binary сохраняют `forbid(unsafe_code)`.

## Разработка по спецификации

- [`SPEC.md`](SPEC.md) — единственный источник правды и полный план этапов.
- [`docs/linen — подготовка к экосистеме.md`](docs/linen%20%E2%80%94%20%D0%BF%D0%BE%D0%B4%D0%B3%D0%BE%D1%82%D0%BE%D0%B2%D0%BA%D0%B0%20%D0%BA%20%D1%8D%D0%BA%D0%BE%D1%81%D0%B8%D1%81%D1%82%D0%B5%D0%BC%D0%B5.md) — архитектурные ограничения и будущая roadmap source libraries, instruments, packages и device artifacts; не текущий список функций.
- [`docs/development/README.md`](docs/development/README.md) — spec-driven workflow.
- [`docs/development/backlog.md`](docs/development/backlog.md) — обоснованные будущие улучшения, включая BUF-001 о PCM16 buffering.
- [`docs/development/task-template.md`](docs/development/task-template.md) — шаблон проверяемой задачи с allowlist.
- [`docs/development/stages/stage-00-foundation.md`](docs/development/stages/stage-00-foundation.md) — объём, traceability и DoD этапа 0.
- [`docs/development/stages/stage-01-syntax.md`](docs/development/stages/stage-01-syntax.md) — объём и приёмка этапа 1.
- [`docs/development/stages/stage-01a-ecosystem-foundation.md`](docs/development/stages/stage-01a-ecosystem-foundation.md) — миграция этапов 0–1 и приёмка 1A.
- [`docs/development/stages/stage-02-types-interpreter-wav.md`](docs/development/stages/stage-02-types-interpreter-wav.md) — контракт, DoD и итоговая приёмка этапа 2.
- [`docs/development/stages/stage-03-ir-bytecode-vm.md`](docs/development/stages/stage-03-ir-bytecode-vm.md) — контракт, подготовительные результаты и DoD этапа 3.
- [`docs/adr/`](docs/adr/) — архитектурные решения, включая ADR 0004/0005 для IR и block VM.
- [`AGENTS.md`](AGENTS.md) — обязательные правила для ИИ-агентов.
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — участие человека и делегирование задач агентам.

Изменение языка или внешнего контракта начинается со спецификации. Основной компилятор, VM и DSP пишутся человеком; агентам по умолчанию делегируются тесты, генераторы, ревью и явно разрешённый бойлерплейт. Модульная система, manifest, registry и package CLI не реализуются как часть ранней подготовки.

## Ближайшие этапы

| Этап | Состояние | Результат |
|---|---|---|
| 0. Каркас и правила проверки | Завершён | Diagnostics, тесты, toolchain и CI |
| 1. Lexer, parser и синтаксические diagnostics | Завершён | Разбор Beep-синтаксиса без type checking и звука |
| 1A. SourceMap и явный контекст компиляции | Завершён | Source identity, session, determinism, четыре новых reserved words |
| 2. Типы, наивный интерпретатор и Beep WAV | Завершён | Semantic `check`, Typed AST, `sine`, `run` и PCM16 WAV |
| 3. IR, bytecode и блочная VM | Завершён | Production VM path, allocation harness и documented benchmark baseline |

Текущий Rust API использует `SourceSpan` с `SourceId`; слова `export`, `package`, `from` и `as` зарезервированы и отклоняются на месте identifiers обычным `E1100`. Следующие этапы начинаются только после отдельной задачи и выполненного DoD предыдущего этапа.

## Лицензия

Проект распространяется по лицензии [MIT](LICENSE). Copyright (c) 2026 Theosov.

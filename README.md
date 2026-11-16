# linen

[![CI](https://github.com/theosovv/linen/actions/workflows/ci.yml/badge.svg)](https://github.com/theosovv/linen/actions/workflows/ci.yml)

**linen** — исследовательский некоммерческий проект статически типизированного функционального языка для описания аудиосигналов. Цель разработки — собственные компилятор, блочная виртуальная машина, DSP-узлы и инструменты исследования звука на Rust.

> Этапы 0, 1 и корректирующий этап 1A завершены. Lexer, parser, source-aware diagnostics, `SourceMap`, явный `CompileSession` и команда `check` реализованы и прошли приёмку по `SPEC.md` 0.2.1. Следующий этап 2 — type checking, resolved IDs, наивный интерпретатор и WAV — начинается по отдельной задаче.

## Что уже работает

- Rust library и binary в одном package.
- `SourceId`, `SourceFile`, `SourceMap` и `SourceSpan` с полуинтервальными byte offsets и Source identity через tokens/AST/diagnostics.
- Предвычисленный line index с Unicode scalar columns, LF/CRLF, lone CR, tab и EOF boundaries.
- Структурированные `Diagnostic`, severity, диагностические коды и typed errors для невалидных source/span связей.
- Явный `CompileSession` с `SourceMap` и `CompileConfig`; `CheckOutcome` сохраняет источники вместе с AST или diagnostic.
- Lexer, AST и recursive descent parser для Beep-синтаксиса, включая лимиты глубины и числа узлов и обработку смешанных synthetic token streams.
- Человекочитаемый diagnostic renderer, разрешающий источник по `SourceId` и использующий logical source name.
- `linen check` со структурными проверками `out`/`render`, diagnostics в stderr и кодами выхода из спецификации.
- Reserved words `export`, `package`, `from` и `as` без добавления module/package grammar.
- Инфраструктурные `--help` и `--version` у исполняемого файла.
- Integration tests, doctest, rustfmt, clippy и GitHub Actions CI.

`check` намеренно не выполняет name resolution, type checking или проверку положительности длительности: эта семантика относится к этапу 2. `run` и `inspect` недоступны. Пользовательские multi-file projects и пакетная функциональность также не реализованы; несколько источников поддерживаются на уровне synthetic in-memory compiler tests.

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

Работают инфраструктурные команды и синтаксическая проверка:

```bash
cargo run -- --help
cargo run -- --version
cargo run -- check fixtures/valid/stage-01-beep.ln
```

После установки binary доступны эквивалентные команды `linen --help`, `linen --version` и `linen check input.ln`. Успешный `check` ничего не печатает. Ошибка исходника возвращает код `1` и diagnostic в stderr; ошибка использования или чтения файла — код `2` и `E5000` для файловой ошибки. Source diagnostic показывает только logical `file_name`; фактический путь остаётся в файловой `E5000`. Нарушение внутреннего renderer invariant возвращает internal error с кодом `3`, без panic.

## Библиотечный API sources и diagnostics

```rust
use linen::{Diagnostic, DiagnosticLabel, ErrorCode, SourceMap, SourceSpan};

fn main() {
    let mut sources = SourceMap::new();
    let source = sources.add_source("input.ln", "let out = 1; render 1s;");
    let span = SourceSpan::new(source, 4, 7).expect("this literal span is valid");
    let diagnostic = Diagnostic::error(
        ErrorCode::UnknownName,
        "unknown name",
        DiagnosticLabel::new(span, "this name is not defined"),
    );

    assert_eq!(diagnostic.code().as_str(), "E2001");
    assert_eq!(diagnostic.primary().span(), span);
}
```

`SourceSpan` — `SourceId` и полуинтервал байтовых offsets `[start, end)`. `SourceMap` выдаёт map-local IDs в порядке добавления и разрешает источники только по ID, допуская duplicate logical names внутри одной карты. Независимые карты могут выдавать равные ID: используйте ID только с создавшей его картой или её идентичной копией. Lookup проверяет наличие индекса, а не происхождение ID. `SourceFile::line_column` вычисляет 1-based Unicode/CRLF locations. `render_diagnostic(&SourceMap, &Diagnostic)` возвращает `Result<String, DiagnosticRenderError>` и не подменяет отсутствующий в карте индекс другим файлом.

`check(source)` создаёт одноисточниковую session с logical name `input.ln` и возвращает `CheckOutcome`. `check_with_config(logical_name, source, config)` принимает явные имя и `CompileConfig`. При session-based API caller сохраняет `CompileSession` до использования AST/diagnostics и не передаёт ID между sessions. Result из `CheckOutcome` разрешается через карту того же outcome; отсутствующий в session индекс возвращает typed `SessionError`.

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
│   └── syntax/
│       ├── mod.rs
│       ├── token.rs
│       ├── lexer.rs
│       ├── ast.rs
│       └── parser.rs
├── tests/
│   ├── source.rs
│   ├── cli.rs
│   ├── diagnostics.rs
│   ├── lexer.rs
│   └── parser.rs
├── fixtures/
│   ├── valid/
│   └── invalid/
├── docs/
│   ├── linen — подготовка к экосистеме.md
│   ├── adr/
│   └── development/
└── .github/
    ├── ISSUE_TEMPLATE/
    └── workflows/ci.yml
```

Новые модули появляются вместе с реальной функциональностью. Пустых каталогов и placeholder API для будущих стадий в репозитории нет.

## Разработка по спецификации

- [`SPEC.md`](SPEC.md) — единственный источник правды и полный план этапов.
- [`docs/linen — подготовка к экосистеме.md`](docs/linen%20%E2%80%94%20%D0%BF%D0%BE%D0%B4%D0%B3%D0%BE%D1%82%D0%BE%D0%B2%D0%BA%D0%B0%20%D0%BA%20%D1%8D%D0%BA%D0%BE%D1%81%D0%B8%D1%81%D1%82%D0%B5%D0%BC%D0%B5.md) — архитектурные ограничения и будущая roadmap source libraries, instruments, packages и device artifacts; не текущий список функций.
- [`docs/development/README.md`](docs/development/README.md) — spec-driven workflow.
- [`docs/development/task-template.md`](docs/development/task-template.md) — шаблон проверяемой задачи с allowlist.
- [`docs/development/stages/stage-00-foundation.md`](docs/development/stages/stage-00-foundation.md) — объём, traceability и DoD этапа 0.
- [`docs/development/stages/stage-01-syntax.md`](docs/development/stages/stage-01-syntax.md) — контракты, состояние реализации, матрица проверки и DoD этапа 1.
- [`docs/development/stages/stage-01a-ecosystem-foundation.md`](docs/development/stages/stage-01a-ecosystem-foundation.md) — миграция этапов 0–1 и фактический отчёт приёмки 1A.
- [`docs/adr/`](docs/adr/) — архитектурные решения, включая ADR 0012 о SourceMap/session.
- [`AGENTS.md`](AGENTS.md) — обязательные правила для ИИ-агентов.
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — участие человека и делегирование задач агентам.

Изменение языка или внешнего контракта начинается со спецификации. Основной компилятор, VM и DSP пишутся человеком; агентам по умолчанию делегируются тесты, генераторы, ревью и явно разрешённый бойлерплейт. Модульная система, manifest, registry и package CLI не реализуются как часть ранней подготовки.

## Ближайшие этапы

| Этап | Состояние | Результат |
|---|---|---|
| 0. Каркас и правила проверки | Завершён | Diagnostics, тесты, toolchain и CI |
| 1. Lexer, parser и синтаксические diagnostics | Завершён | Разбор Beep-синтаксиса без type checking и звука |
| 1A. SourceMap и явный контекст компиляции | Завершён | Source identity через diagnostics/tokens/AST, session, determinism, четыре новых reserved words |
| 2. Типы, наивный интерпретатор и Beep WAV | Следующий, по явной задаче | Первая работающая вертикаль `source → sound` |

Этапы выполняются последовательно. Текущий Rust API использует `SourceSpan` с `SourceId`; слова `export`, `package`, `from` и `as` зарезервированы и отклоняются на месте identifiers обычным `E1100`. Примеры звука будут добавлены после реализации соответствующих команд.

## Лицензия

Проект распространяется по лицензии [MIT](LICENSE). Copyright (c) 2026 Theosov.

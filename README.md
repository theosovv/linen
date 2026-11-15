# linen

[![CI](https://github.com/theosovv/linen/actions/workflows/ci.yml/badge.svg)](https://github.com/theosovv/linen/actions/workflows/ci.yml)

**linen** — исследовательский некоммерческий проект статически типизированного функционального языка для описания аудиосигналов. Цель разработки — собственные компилятор, блочная виртуальная машина, DSP-узлы и инструменты исследования звука на Rust.

> Этапы 0 и 1 завершены: lexer, parser, source diagnostics и команда `check` реализованы и прошли приёмку. Следующий этап 2 добавит type checking, интерпретатор, синтез звука и WAV; эти возможности ещё не реализованы.

## Что уже работает

- Rust library и binary в одном package.
- Структурированные `Span`, `Diagnostic`, severity и диагностические коды.
- Проверка diagnostics через библиотечный API без запуска CLI.
- Lexer, AST и recursive descent parser для Beep-синтаксиса, включая лимиты глубины и числа узлов.
- Unicode/CRLF source locations и человекочитаемый diagnostic renderer.
- `linen check` со структурными проверками `out`/`render`, diagnostics в stderr и кодами выхода из спецификации.
- Инфраструктурные `--help` и `--version` у исполняемого файла.
- Integration tests, doctest, rustfmt, clippy и GitHub Actions CI.

`check` намеренно не выполняет name resolution, type checking или проверку положительности длительности: эта семантика относится к этапу 2. `run` и `inspect` недоступны.

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

Полный набор должен проходить без ignored tests перед завершением этапа 1.

## Текущий CLI

Работают инфраструктурные команды и синтаксическая проверка:

```bash
cargo run -- --help
cargo run -- --version
cargo run -- check fixtures/valid/stage-01-beep.ln
```

После установки binary доступны эквивалентные команды `linen --help`, `linen --version` и `linen check input.ln`. Успешный `check` ничего не печатает. Ошибка исходника возвращает код `1` и diagnostic в stderr; ошибка использования или чтения файла — код `2` и `E5000` для файловой ошибки.

## Библиотечный API diagnostics

```rust
use linen::{Diagnostic, DiagnosticLabel, ErrorCode, Span};

fn main() {
    let span = Span::new(4, 7).expect("this literal span is valid");
    let diagnostic = Diagnostic::error(
        ErrorCode::UnknownName,
        "unknown name",
        DiagnosticLabel::new(span, "this name is not defined"),
    );

    assert_eq!(diagnostic.code().as_str(), "E2001");
    assert_eq!(diagnostic.primary().span(), span);
}
```

`Span` — полуинтервал байтовых offsets `[start, end)`. `Diagnostic` хранит основной и дополнительные labels, notes, severity и стабильный код ошибки. `line_column` вычисляет 1-based Unicode/CRLF locations, а `render_diagnostic` формирует человекочитаемый вывод с именем файла и primary location.

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
│   ├── diagnostic.rs
│   └── syntax/
│       ├── mod.rs
│       ├── token.rs
│       ├── lexer.rs
│       ├── ast.rs
│       └── parser.rs
├── tests/
│   ├── cli.rs
│   ├── diagnostics.rs
│   ├── lexer.rs
│   └── parser.rs
├── fixtures/
│   ├── valid/
│   └── invalid/
├── docs/
│   ├── adr/
│   └── development/
└── .github/
    ├── ISSUE_TEMPLATE/
    └── workflows/ci.yml
```

Новые модули появляются вместе с реальной функциональностью. Временные заглушки этапа 1 удалены; пустых каталогов для будущих стадий в репозитории нет.

## Разработка по спецификации

- [`SPEC.md`](SPEC.md) — единственный источник правды и полный план этапов.
- [`docs/development/README.md`](docs/development/README.md) — spec-driven workflow.
- [`docs/development/task-template.md`](docs/development/task-template.md) — шаблон проверяемой задачи с allowlist.
- [`docs/development/stages/stage-00-foundation.md`](docs/development/stages/stage-00-foundation.md) — объём, traceability и DoD этапа 0.
- [`docs/development/stages/stage-01-syntax.md`](docs/development/stages/stage-01-syntax.md) — контракты, состояние реализации, матрица проверки и DoD этапа 1.
- [`docs/adr/`](docs/adr/) — архитектурные решения.
- [`AGENTS.md`](AGENTS.md) — обязательные правила для ИИ-агентов.
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — участие человека и делегирование задач агентам.

Изменение языка или внешнего контракта начинается со спецификации. Основной компилятор, VM и DSP пишутся человеком; агентам по умолчанию делегируются тесты, генераторы, ревью и явно разрешённый бойлерплейт.

## Ближайшие этапы

| Этап | Состояние | Результат |
|---|---|---|
| 0. Каркас и правила проверки | Завершён | Diagnostics, тесты, toolchain и CI |
| 1. Lexer, parser и синтаксические diagnostics | Завершён | Разбор Beep-синтаксиса без type checking и звука |
| 2. Типы, наивный интерпретатор и Beep WAV | Следующий | Первая работающая вертикаль `source → sound` |

Этапы выполняются последовательно. Примеры звука будут добавлены после реализации соответствующих команд.

## Лицензия

Проект распространяется по лицензии [MIT](LICENSE). Copyright (c) 2026 Theosov.

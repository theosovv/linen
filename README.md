# linen

[![CI](https://github.com/theosovv/linen/actions/workflows/ci.yml/badge.svg)](https://github.com/theosovv/linen/actions/workflows/ci.yml)

**linen** — исследовательский некоммерческий проект статически типизированного функционального языка для описания аудиосигналов. Цель разработки — собственные компилятор, блочная виртуальная машина, DSP-узлы и инструменты исследования звука на Rust.

> Сейчас завершён только инфраструктурный этап 0. Компиляция `.ln`, синтез звука и WAV ещё не реализованы.

## Что уже работает

- Rust library и binary в одном package.
- Структурированные `Span`, `Diagnostic`, severity и начальные диагностические коды.
- Проверка diagnostics через библиотечный API без запуска CLI.
- Инфраструктурные `--help` и `--version` у исполняемого файла.
- Integration tests, doctest, rustfmt, clippy и GitHub Actions CI.

Команды `linen check`, `linen run` и `linen inspect` относятся к следующим этапам и сейчас недоступны.

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

## Текущий CLI

```bash
cargo run -- --help
cargo run -- --version
```

После установки binary доступны эквивалентные команды `linen --help` и `linen --version`. Неизвестные аргументы завершаются с кодом использования `2`.

## Библиотечный API этапа 0

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

`Span` — полуинтервал байтовых offsets `[start, end)`. `Diagnostic` хранит основной и дополнительные labels, notes, severity и стабильный код ошибки. Source rendering и line/column появятся вместе с соответствующей frontend-функциональностью.

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
│   └── diagnostic.rs
├── tests/
│   ├── cli.rs
│   └── diagnostics.rs
├── docs/
│   ├── adr/
│   └── development/
└── .github/
    ├── ISSUE_TEMPLATE/
    └── workflows/ci.yml
```

Новые модули появляются только вместе с реальной функциональностью; пустых каталогов для будущих стадий в репозитории нет.

## Разработка по спецификации

- [`SPEC.md`](SPEC.md) — единственный источник правды и полный план этапов.
- [`docs/development/README.md`](docs/development/README.md) — spec-driven workflow.
- [`docs/development/task-template.md`](docs/development/task-template.md) — шаблон проверяемой задачи с allowlist.
- [`docs/development/stages/stage-00-foundation.md`](docs/development/stages/stage-00-foundation.md) — объём, traceability и DoD этапа 0.
- [`docs/adr/`](docs/adr/) — архитектурные решения.
- [`AGENTS.md`](AGENTS.md) — обязательные правила для ИИ-агентов.
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — участие человека и делегирование задач агентам.

Изменение языка или внешнего контракта начинается со спецификации. Основной компилятор, VM и DSP пишутся человеком; агентам по умолчанию делегируются тесты, генераторы, ревью и явно разрешённый бойлерплейт.

## Ближайшие этапы

| Этап | Состояние | Результат |
|---|---|---|
| 0. Каркас и правила проверки | Завершён | Diagnostics, тесты, toolchain и CI |
| 1. Lexer, parser и синтаксические diagnostics | Следующий | Разбор Beep-синтаксиса без type checking и звука |
| 2. Типы, наивный интерпретатор и Beep WAV | Запланирован | Первая работающая вертикаль `source → sound` |

Этапы выполняются последовательно. Примеры Beep будут добавлены в README только после того, как соответствующие команды действительно заработают.

## Лицензия

Проект распространяется по лицензии [MIT](LICENSE). Copyright (c) 2026 Theosov.

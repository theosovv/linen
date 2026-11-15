# Этап 1 — контракты, fixtures, тесты и каркас

- Статус: подготовительная задача и этап 1 завершены по `SPEC.md` 0.1.1.
- Исполнитель подготовительной задачи: агент.
- Основной lexer/parser и завершение CLI: человек.

> Документ сохранён как историческая задача. Его allowlist не действует для новых изменений. Требования экосистемы из `SPEC.md` 0.2.0 реализуются отдельным этапом 1A по `../stages/stage-01a-ecosystem-foundation.md` и новой явной задаче; этот файл не разрешает текущие production-изменения.

## Контекст

- Этап: 1, Lexer, parser и синтаксические diagnostics.
- Пункты `SPEC.md`: 4.3, 6.1–6.2, 7.3, 7.6–7.7, 9, 10.1.
- Связанные ADR: 0001, 0002, 0011.
- Текущее состояние: этап 0 завершён и его обязательные проверки повторно пройдены.

## Задача

До ручной реализации ядра уточнить неоднозначные контракты, подготовить документ этапа и ADR, создать типы tokens/AST, fixtures и независимые contract tests. Entry points остаются явными временными заглушками по прямому разрешению пользователя.

## Разрешено менять

- `SPEC.md`, `CHANGELOG.md`, `README.md`;
- `docs/development/**`, `docs/adr/**`;
- `src/lib.rs`, `src/main.rs`, `src/diagnostic.rs`, `src/syntax/**`;
- `tests/**`, `fixtures/**`.

Production-код разрешён только в перечисленных файлах. Зависимости, toolchain, `Cargo.toml`, `Cargo.lock`, `AGENTS.md`, IR/sema/VM/DSP и другие файлы не входят в задачу.

## Нецели

- Реализация основного lexer/parser человеком не делегируется агенту.
- Type checking, name resolution, arity и duration semantics этапа 2.
- `run`, `inspect`, WAV, IR, bytecode, VM и DSP.
- Parser recovery и несколько diagnostics за один запуск.
- Объявление этапа завершённым при наличии заглушек или failing tests.

## Контракт и независимые проверки

| Требование | Проверка | Независимый oracle |
|---|---|---|
| Lexer tokens/spans | Positive, negative, boundary vectors в `tests/lexer.rs` | Явно перечисленные tokens и offsets |
| AST и приоритеты | Semantic assertions в `tests/parser.rs` | Структура из EBNF, не production pretty-printer |
| Diagnostics | Код, primary и отобранные secondary spans | Таблица решений `SPEC.md` §6.2 |
| Parser limits | `N` принимается, `N+1` отклоняется | Ручной подсчёт глубины/узлов |
| Arbitrary tokens | Детерминированный corpus без panic | Инвариант безопасного завершения |
| CLI | `check`, streams, exit codes, file errors | Внешний контракт `SPEC.md` §6.1 |

## Критерии приёмки подготовительной задачи

- [x] Неоднозначности вынесены в вопросы и решения внесены сначала в `SPEC.md`.
- [x] Созданы stage document, ADR и явный allowlist.
- [x] Token/AST типы и fixtures созданы без реализации ядра.
- [x] Tests не используют production-функцию для expected и не содержат `#[ignore]`.
- [x] Все временные заглушки удалены человеком.
- [x] Contract tests и общий набор проходят.
- [x] DoD этапа 1 выполнен.

## Обязательные команды

```bash
cargo test --test lexer --locked
cargo test --test parser --locked
cargo test --test diagnostics --locked
cargo test --test cli --locked
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
cargo test --doc --locked
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked
```

До устранения заглушек ожидаются failures в lexer/parser/diagnostics/CLI tests. Это не разрешает пропускать, удалять или ослаблять тесты.

## Отчёт об исполнении

- Изменённые файлы: перечисляются в финальном отчёте агента и diff.
- Добавленные проверки и oracles: см. матрицу выше и документ этапа.
- Выполненные команды и результаты: записаны в `docs/development/stages/stage-01-syntax.md`; fmt, clippy, 47 integration tests, doctest и rustdoc успешны, ignored tests нет.
- Найденные при ревью дефекты diagnostic spans, labels и инварианта `Span` исправлены и покрыты regression-тестами.
- Оставшиеся ограничения соответствуют границам завершённого этапа 1: type checking, name resolution, `run`, `inspect`, звук и WAV относятся к последующим этапам.

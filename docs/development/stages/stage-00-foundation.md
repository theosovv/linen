# Этап 0 — каркас и правила проверки

- Статус: завершён.
- Источник требований: `SPEC.md`, разделы 5, 6.2, 8 и 9 (этап 0).
- Связанные ADR: `0001`, `0002`.

## Ожидаемый результат

Минимальный Rust-проект с библиотекой и бинарным файлом, зафиксированным toolchain, структурированной диагностикой, тестами, форматированием, clippy и CI. Следующий этап можно начинать без смены инфраструктуры.

## Объём и allowlist

В рамках явной пользовательской задачи разрешено создавать и изменять:

- `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `rustfmt.toml`;
- `src/lib.rs`, `src/main.rs`, `src/diagnostic.rs`;
- тесты этапа в `tests/`;
- `SPEC.md` только как перемещение исходного документа из `docs/SPEC.md`, без изменения содержимого;
- корневую документацию, `docs/development/`, `docs/adr/` и файлы GitHub-репозитория, необходимые для процесса и CI.

## Нецели

- Lexer, parser, AST, type checker, interpreter, DSP, IR, bytecode и VM.
- Команды `check`, `run`, `inspect` и создание WAV.
- Пустые модули, placeholder API, `todo!()` и `unimplemented!()`.
- Зависимости, parser generator, WAV- или DSP-библиотеки.
- Новые языковые и диагностические контракты сверх `SPEC.md`.

## Принятый минимальный дизайн

- Один package содержит library crate и binary crate; workspace не создаётся.
- Корень библиотеки является текущим facade и реэкспортирует только реально реализованные типы диагностики. Функции `check`, `compile`, `render` и `api.rs` не создаются до появления соответствующих стадий.
- `Span` представляет полуинтервал байтовых offsets `[start, end)` и не допускает `start > end`.
- `Diagnostic` содержит severity, стабильный код, сообщение, primary label, secondary labels и notes. Доступ к ним не требует CLI.
- `ErrorCode` охватывает начальный набор кодов из `SPEC.md` §6.2. Текст человекочитаемых сообщений пока не закрепляется snapshot-контрактом.
- Бинарный файл поддерживает только инфраструктурные `--help`, `-h`, `--version`, `-V`; отсутствие аргументов показывает help. Ошибка использования возвращает exit code `2`. Это не реализация будущих команд компилятора.
- Line/column rendering не добавляется до появления source renderer; byte offsets уже доступны и проверяются.

## Матрица проверки

| Требование | Проверка | Oracle |
|---|---|---|
| Полуинтервал `Span` | Конструктор, длина, пустой span, неверный диапазон | Явные offsets и длины |
| Состав `Diagnostic` | Severity, код, labels, notes | Явно созданные значения |
| Стабильность начальных кодов | Полный список строк `E####`, уникальность | Таблица `SPEC.md` §6.2 |
| Доступ без CLI | Integration test использует только library API | Прямые assertions |
| Минимальный binary | Help, version, ошибка использования | Явные streams и exit codes |
| Качество каркаса | fmt, clippy, тесты, rustdoc | Стандартные инструменты Rust |

## Definition of Done

- [x] `cargo test --all-targets --locked` проходит.
- [x] `cargo fmt --check` проходит.
- [x] `cargo clippy --all-targets --locked -- -D warnings` проходит.
- [x] Diagnostics проверяются библиотечными integration tests без запуска CLI.
- [x] Нет модулей и API для будущих стадий.
- [x] Спецификация находится в корне как `SPEC.md` без изменения содержимого.
- [x] CI содержит те же обязательные проверки.

## Фактические проверки

Проверки выполнены локально 2026-11-11:

- `cargo fmt --check` — успешно.
- `cargo clippy --all-targets --locked -- -D warnings` — успешно, без предупреждений.
- `cargo test --all-targets --locked` — успешно: 10 integration tests, 0 failed, 0 ignored.
- `cargo test --doc --locked` — успешно: 1 doctest.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked` — успешно.
- `cargo run --locked -- --help` и `cargo run --locked -- --version` — успешно.
- `cargo metadata --locked --no-deps --format-version 1` — успешно.
- Относительные Markdown-ссылки и синтаксис пяти GitHub YAML-файлов — успешно.
- Поиск `TODO`, `FIXME`, `todo!()`, `unimplemented!()` и `unreachable!()` в `src/` — совпадений нет.

Удалённый GitHub Actions workflow не запускался: репозиторий не коммитился и не отправлялся на GitHub. Локально выполнен тот же набор Rust-команд. `actionlint`/`yamllint` в окружении отсутствуют; YAML проверен через доступный parser, без удалённой схемной валидации GitHub.

# Changelog

В файле фиксируются заметные изменения проекта. Изменения языковых и внешних контрактов сначала вносятся в `SPEC.md` и её changelog; здесь они дополнительно отмечаются с указанием совместимости.

## Unreleased

### Added

- Минимальный Rust package с library и binary, зафиксированным toolchain и `Cargo.lock`.
- Структурированные `Span`, `Diagnostic`, `DiagnosticLabel`, `Severity` и начальные коды ошибок из спецификации.
- Integration tests diagnostics и инфраструктурного CLI, doctest публичного API.
- Настройки rustfmt, clippy и GitHub Actions CI.
- Spec-driven workflow, шаблон задачи, документ этапа 0 и ADR 0001–0002.
- Правила для ИИ-агентов, README, CONTRIBUTING и базовые файлы GitHub-репозитория.
- Лицензия MIT.
- Спецификация 0.1.1: уточнены границы этапа 1, malformed literals, primary spans, Unicode line/column, fail-fast diagnostics и parser limits; добавлены `E1102` и `E1103`.
- Документ этапа 1, задача на подготовительный каркас и ADR 0011.
- Типы tokens/AST, `ParserConfig`, CLI dispatch `check`, fixtures и независимые lexer/parser/diagnostics/CLI contract tests для этапа 1.

### In progress

- Этап 1: `Lexer::tokenize`, `Parser::parse_program`, `line_column` и `render_diagnostic` остаются явными временными `todo!()` по согласованному объёму. Рабочий `linen check` ещё не реализован, contract tests до реализации падают на заглушках.

### Compatibility

- Существующие языковые программы и аудиоконтракты отсутствуют; обратной совместимости на этапе 0 пока нет.
- Инфраструктурные `--help` и `--version` сохранены. Help включает незавершённый `check`, но это не объявляет его работающим.
- `run` и `inspect` не объявлены работающими и не имеют placeholder-реализаций.
- Изменения спецификации 0.1.1 уточняют ещё не реализованный этап и не ломают существующие `.ln`-программы.

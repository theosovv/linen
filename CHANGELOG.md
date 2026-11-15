# Changelog

В файле фиксируются заметные изменения проекта. Изменения языковых и внешних контрактов сначала вносятся в `SPEC.md` и её changelog; здесь они дополнительно отмечаются с указанием совместимости.

## Unreleased

### Added

- Спецификация 0.2.0 и документ подготовки к модульной/пакетной экосистеме: source libraries, instruments, presets, device packages, capabilities, resources и compatibility rules.
- Спецификация обязательного корректирующего этапа 1A перед этапом 2 и ADR 0012 о `SourceMap`, `SourceSpan` и `CompileSession`. Production-реализация 1A ещё не выполнялась.
- Минимальный Rust package с library и binary, зафиксированным toolchain и `Cargo.lock`.
- Структурированные `Span`, `Diagnostic`, `DiagnosticLabel`, `Severity` и начальные коды ошибок из спецификации.
- Integration tests diagnostics и инфраструктурного CLI, doctest публичного API.
- Настройки rustfmt, clippy и GitHub Actions CI.
- Spec-driven workflow, шаблон задачи, документ этапа 0 и ADR 0001–0002.
- Правила для ИИ-агентов, README, CONTRIBUTING и базовые файлы GitHub-репозитория.
- Лицензия MIT.
- Спецификация 0.1.1: уточнены границы этапа 1, malformed literals, primary spans, Unicode line/column, fail-fast diagnostics и parser limits; добавлены `E1102` и `E1103`.
- Документ этапа 1, задача на подготовительный каркас и ADR 0011.
- Ручные lexer/parser, tokens/AST, `ParserConfig`, source diagnostics и рабочая команда `check` для этапа 1.
- Fixtures и независимые lexer/parser/diagnostics/CLI contract tests.

### Fixed

- Primary spans `E1101`/`E1103` и primary/secondary spans `E2003` приведены к контракту спецификации; добавлены regression-тесты.
- Diagnostic renderer подписывает primary caret текстом primary label, а не secondary label.
- Поля `Span` снова приватны, поэтому диапазон нельзя создать в обход проверки `Span::new`.

### Changed

- Спецификация 0.2.0 требует Source identity через tokens/AST/Typed AST/IR, явное владение sources/config, resolved IDs, builtin descriptors, `NodeOrigin`, детерминированный output и разделение language/VM/package versions.
- `export`, `package`, `from` и `as` зарезервированы в спецификации для будущих modules/packages. Это решение ещё не реализовано в текущем lexer.
- Правила разработки дополнены корректирующими stage documents для новых требований к завершённым этапам и проверками source/determinism/capability инвариантов.

### Completed

- Этап 1 завершён по спецификации 0.1.1: lexer, parser, source diagnostics и `check` прошли приёмку. Перед этапом 2 требуется реализация и приёмка этапа 1A.

### Compatibility

- Существующие языковые программы и аудиоконтракты отсутствуют; обратной совместимости на этапе 0 пока нет.
- Инфраструктурные `--help` и `--version` сохранены. Help включает рабочий `check`.
- `run` и `inspect` не объявлены работающими и не имеют placeholder-реализаций.
- Изменения спецификации 0.1.1 уточнили ещё не реализованный на тот момент этап и не ломали существующие `.ln`-программы.
- Спецификация 0.2.0 сознательно меняет лексический контракт: после реализации 1A четыре новых reserved words нельзя будет использовать как identifiers. В текущих fixtures таких declarations нет; compatibility tests обязаны сохранить остальные программы без изменения expected.
- Миграция `Span` → `SourceSpan` и source-aware API затрагивает нестабильный внутренний Rust API, а не стабильные diagnostic codes или Beep semantics. Текущий код ещё не мигрирован.
- Module/package syntax, stable bytecode ABI, registry и device packages не добавлены.

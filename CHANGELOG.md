# Changelog

В файле фиксируются заметные изменения проекта. Изменения языковых и внешних контрактов сначала вносятся в `SPEC.md` и её changelog; здесь они дополнительно отмечаются с указанием совместимости.

## Unreleased

### Added

- Спецификация 0.2.0 и документ подготовки к модульной/пакетной экосистеме: source libraries, instruments, presets, device packages, capabilities, resources и compatibility rules.
- Спецификация корректирующего этапа 1A и ADR 0012 о `SourceMap`, `SourceSpan` и `CompileSession`.
- Реализация 1A: `SourceId`, `SourceFile`, `SourceMap`, `SourceSpan` и line index в базовом `source` module.
- Source-aware tokens, AST, diagnostics и renderer с typed `DiagnosticRenderError` для unknown IDs, reversed/out-of-bounds offsets и non-UTF-8 boundaries.
- `CompileSession`, `CompileConfig`, `SessionError` и `CheckOutcome`, сохраняющий `SourceMap` вместе с `Result<Program, Diagnostic>`.
- Synthetic multi-source, boundary, mixed-token, in-process и cross-process determinism tests; fixtures для четырёх reserved words и identifier boundaries.
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
- Поля `Span` снова приватны, поэтому диапазон нельзя создать в обход проверки `Span::new`; на этапе 1A этот инвариант перенесён в `SourceSpan::new`.
- CLI больше не включает absolute/canonical parent path в source diagnostic: используется display-safe logical `file_name`; файловый `E5000` сохраняет фактический путь.
- Parser детерминированно возвращает `E1100` на первом token с чужим SourceId и не объединяет spans разных источников.

### Changed

- Внутренний Rust API мигрирован с однофайлового `Span` на `SourceSpan`; lexer/parser получают SourceId, renderer — SourceMap, простой `check` возвращает владеющий источниками `CheckOutcome`.
- Спецификация 0.2.1 после ревью явно ограничивает `SourceId` создавшей его `SourceMap` или её идентичной копией. Независимые карты могут выдавать равные индексы; `UnknownSource` означает отсутствующий в карте индекс, а не обнаруженный foreign same-index ID. Глобальный provenance и mutable ID generator не добавляются.
- `export`, `package`, `from` и `as` теперь фактически зарезервированы в lexer и отклоняются parser как `E1100` на месте identifier. Prefix/case variants остаются identifiers; module/package grammar не добавлена.
- Спецификация 0.2.0 требует Source identity через tokens/AST/Typed AST/IR, явное владение sources/config, resolved IDs, builtin descriptors, `NodeOrigin`, детерминированный output и разделение language/VM/package versions.
- Правила разработки дополнены корректирующими stage documents для новых требований к завершённым этапам и проверками source/determinism/capability инвариантов.

### Completed

- Этап 1 завершён по спецификации 0.1.1: lexer, parser, source diagnostics и `check` прошли приёмку.
- Этап 1A завершён по спецификации 0.2.1: SourceMap/SourceSpan, session/facade, source-aware diagnostics, CLI logical names, reserved words и determinism прошли узкие и общие проверки.

### Compatibility

- Сознательный contract break спецификации 0.2.0 реализован: четыре новых reserved words нельзя использовать как identifiers. Старые fixtures без них сохраняют прежние expected.
- Уточнение 0.2.1 не меняет поведение реализованных `.ln`-программ или diagnostic codes. Оно документирует ограничение нестабильного Rust API: результат и diagnostics нельзя разрешать через независимую `SourceMap`; используйте карту той же session/outcome.
- Миграция `Span` → `SourceSpan` и source-aware API затрагивает только нестабильный внутренний Rust API. Стабильные diagnostic codes, primary byte offsets, fail-fast order и Beep syntax сохранены.
- Однофайловый CLI остаётся прежним по командам и streams: успешный `check` молчит, source error использует exit `1`, usage/UTF-8/file error — exit `2`, internal renderer error — exit `3`.
- Пользовательские multi-file projects, module/package syntax, stable bytecode ABI, registry и device packages не добавлены.
- `run` и `inspect` по-прежнему недоступны и не имеют placeholder-реализаций.

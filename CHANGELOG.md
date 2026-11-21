# Changelog

В файле фиксируются заметные изменения проекта. Изменения языковых и внешних контрактов сначала вносятся в `SPEC.md` и её changelog; здесь они дополнительно отмечаются с указанием совместимости.

## Unreleased

### Added

- Production-реализация этапа 4: function/block/parser semantics, resolved IDs/scopes, Typed AST, graph-construction inlining, eager roots, sharing/fresh state, per-call Float evaluator, call origins и expansion limits. Все 208 integration tests и полный набор проходят; временные заглушки удалены.

- Спецификация 0.5.0 и ADR 0007 для этапа 4: lexical `FunctionId`/`BindingId`, functions/blocks/pipeline, one-time eager args/locals, sharing/fresh state, extended roots, per-call Float diagnostics, inner-to-outer call origins и `FunctionLimits` с новым `E2106`. Добавлены stage/task documents. На момент подготовительной фазы production-функции ещё не работали; финальная реализация и приёмка добавлены отдельной записью.
- Подготовительный test-facing каркас этапа 4: syntax/Typed representations, `FunctionId`, `FunctionLimits`, call origins/verifiers/renderer plumbing, восемь явных временных заглушек, independent function/block reference и fixed-seed generator. Добавлены 28 integration tests и девять fixtures. Общий corpus — 208 tests: 182 passed, 26 ожидаемых failed на parser stubs, 0 ignored; все 180 прежних tests проходят.

- Production path этапа 3 переведён на verified IR/bytecode и блочную VM: renderer использует один preallocated scratch buffer, variable blocks и source mapping через debug origins. Naive production interpreter удалён после three-way parity; reference остаётся независимым oracle. Structured VM creation/runtime errors получают CLI exit `3`, missing origin обрабатывается явно.
- Allocation-counting integration test с разрешённым test-only `GlobalAlloc` подтверждает 0 heap allocations в steady-state `render_into` и `reset`. Production library/binary сохраняют `#![forbid(unsafe_code)]`; новых dependencies нет. Финальный corpus содержит 180 integration tests, все проходят.
- Спецификация 0.4.0 для этапа 3: canonical typed DAG, evaluation roots всех top-level bindings, sharing/NodeOrigin, IR/bytecode verification, capability/resource models, typed registers/StateId/output, debug stripping, block VM/reset/errors и renderer migration. Добавлены ADR 0004/0005 и stage/task documents.
- Подготовительный test-facing каркас этапа 3: IR/bytecode/VM representations, descriptors, configs, structured compile/VM errors, facade `create_vm` и ровно семь явных временных заглушек. Добавлены 52 integration tests в `ir_verify`, `bytecode_verify`, `vm`, `differential`, `properties`, `compatibility`, три fixtures и fixed-seed generator. Все 173 tests компилируются; 99 passed, 74 ожидаемых failed до production-реализации, 0 ignored.
- Production-реализация этапа 2: resolver и детерминированные `BindingId`/`BuiltinId`, session-owned `SymbolStorage`, `Float`/`Sig`, Typed AST с явными lifts, descriptor-driven `sine`, checked compile-time arithmetic и duration/sample/RIFF validation.
- Наивный посемпловый interpreter, независимое resettable состояние `sine`, runtime NaN/Inf diagnostics с structured sample index и потоковый mono PCM16 WAV-writer через `Write`.
- Команда `run` с `--output`, `--sample-rate`, `--block-size`, `--max-seconds` и `--force`; temporary file создаётся в output directory и публикуется только после полного успеха. Semantic `check` выполняет name/type/duration checks с sample rate 48 000 Гц.
- Regression fixtures и tests для повторной компиляции в одной `CompileSession` и broken-symlink output. Финальный набор содержит 121 integration test: `audio` 9, `cli` 24, `diagnostics` 8, `lexer` 14, `parser` 23, `reference` 3, `sema` 20, `source` 14, `wav` 6.
- Спецификация 0.3.0 для этапа 2: явный Typed AST/lift, плоские `BindingId`/`BuiltinId`, semantic diagnostic spans/order, structured `sample_index`, новые `E2105`/`E4003`, duration/RIFF overflow и protective render limit. Добавлены ADR 0003, 0006 и 0009, stage/task documents; на момент этой подготовительной записи production-этап ещё не был завершён.
- Подготовительный каркас этапа 2: Typed AST, `BindingId`/`BuiltinId`, `SymbolStorage`, descriptor `sine`, `RenderConfig`, compile outcomes, CLI argument/atomic host boilerplate и семь явно документированных временных entry-point заглушек. Добавлены sema/audio/WAV/CLI/reference tests, 11 fixtures и независимый test-support reference. На момент подготовительной приёмки все 116 integration tests компилировались; 77 passed, 39 ожидаемых failed, ignored tests не было.
- Спецификация 0.2.0 и документ подготовки к модульной/пакетной экосистеме: source libraries, instruments, presets, device packages, capabilities, resources и compatibility rules.
- Спецификация корректирующего этапа 1A и ADR 0012 о `SourceMap`, `SourceSpan` и `CompileSession`.
- Реализация 1A: `SourceId`, `SourceFile`, `SourceMap`, `SourceSpan` и line index в базовом `source` module.
- Source-aware tokens, AST, diagnostics и renderer с typed `DiagnosticRenderError` для unknown IDs, reversed/out-of-bounds offsets и non-UTF-8 boundaries.
- `CompileSession`, `CompileConfig`, `SessionError` и `CheckOutcome`, сохраняющий `SourceMap` вместе с `Result<Program, Diagnostic>`.
- Synthetic multi-source, boundary, mixed-token, in-process и cross-process determinism tests; fixtures для четырёх reserved words и identifier boundaries.
- Минимальный Rust package с library и binary, зафиксированный toolchain и `Cargo.lock`.
- Структурированные `Span`, `Diagnostic`, `DiagnosticLabel`, `Severity` и начальные коды ошибок из спецификации.
- Integration tests diagnostics и инфраструктурного CLI, doctest публичного API.
- Настройки rustfmt, clippy и GitHub Actions CI.
- Spec-driven workflow, шаблон задачи, документ этапа 0 и ADR 0001–0002.
- Правила для ИИ-агентов, README, CONTRIBUTING и базовые файлы GitHub-репозитория.
- Лицензия MIT.
- Спецификация 0.1.1: уточнены границы этапа 1, malformed literals, primary spans, Unicode line/column, fail-fast order и parser limits; добавлены `E1102` и `E1103`.
- Документ этапа 1, задача на подготовительный каркас и ADR 0011.
- Ручные lexer/parser, tokens/AST, `ParserConfig`, source diagnostics и рабочая команда `check` для этапа 1.
- Fixtures и независимые lexer/parser/diagnostics/CLI contract tests.

### Fixed

- Block VM выполняет instructions в Vec order для каждого sample, а nonfinite output возвращается как structured VM error на `SetOutput`. Первый runtime diagnostic, его span и sample index снова совпадают с посемпловым reference при всех block sizes, включая конкурирующие errors в разных bindings.
- Structural bytecode verifier не выделяет память пропорционально объявленным register/state counts. `Vm::new` проверяет arithmetic overflow и Vec capacity limits до verifier/runtime allocations и возвращает `SizeOverflow` вместо capacity-overflow panic.
- `SetOutput` получает отдельный origin полного out value, включая aliases; frequency origin `Sine` сохранён для `E4001`. Добавлены regression fixtures/tests и exact-span проверка.
- Повторная semantic compilation в одной `CompileSession` начинается с пустого `SymbolStorage`: `BindingId` снова выдаются от нуля, результат и storage не зависят от предыдущих компиляций.
- Atomic publish без `--force` использует создание hard link на завершённый temporary file вместо небезопасного безусловного rename. Broken symlinks и файлы, появившиеся после предварительной проверки, не перезаписываются.
- Diagnostics получили конкретные имена, types, arity, sample rate и limits вместо повторяющихся generic messages. Renderer выводит secondary source locations и notes; E1102 и пояснения E2004 исправлены, E4000 показывает invalid numeric value.
- Primary spans `E1101`/`E1103` и primary/secondary spans `E2003` приведены к контракту спецификации; добавлены regression-тесты.
- Diagnostic renderer подписывает primary caret текстом primary label, а не secondary label.
- Поля `Span` снова приватны, поэтому диапазон нельзя создать в обход проверки `Span::new`; на этапе 1A этот инвариант перенесён в `SourceSpan::new`.
- CLI больше не включает absolute/canonical parent path в source diagnostic: используется display-safe logical `file_name`; файловая `E5000` сохраняет фактический путь.
- Parser детерминированно возвращает `E1100` на первом token с чужим SourceId и не объединяет spans разных источников.

### Changed

- `check` теперь выполняет предусмотренные спецификацией 0.3.0 name/type/duration checks. Программы, которые на этапе 1 проходили только syntax checks, теперь отклоняются при semantic errors.
- Человекочитаемые diagnostic messages стали предметными и не дублируют primary labels. Коды, `SourceId`, primary/secondary spans, fail-fast order и structured sample index не изменены; точное оформление текста остаётся нестабильным контрактом.
- Внутренний Rust API мигрирован с однофайлового `Span` на `SourceSpan`; lexer/parser получают SourceId, renderer — SourceMap, простой `check` возвращает владеющий источниками `CheckOutcome`.
- Спецификация 0.2.1 после ревью явно ограничивает `SourceId` создавшей его `SourceMap` или её идентичной копией. Независимые карты могут выдавать равные индексы; `UnknownSource` означает отсутствующий в карте индекс, а не обнаруженный foreign same-index ID. Глобальный provenance и mutable ID generator не добавляются.
- `export`, `package`, `from` и `as` теперь фактически зарезервированы в lexer и отклоняются parser как `E1100` на месте identifier. Prefix/case variants остаются identifiers; module/package grammar не добавлена.
- Спецификация 0.2.0 требует Source identity через tokens/AST/Typed AST/IR, явное владение sources/config, resolved IDs, builtin descriptors, `NodeOrigin`, детерминированный output и разделение language/VM/package versions.
- Правила разработки дополнены корректирующими stage documents для новых требований к завершённым этапам и проверками source/determinism/capability инвариантов.

### Completed

- Этап 4 завершён 2026-11-21 по спецификации 0.5.0: функции, lexical scopes, блоки, pipeline, eager roots, sharing/fresh state, call origins, per-call Float diagnostics и expansion limits реализованы. Все 208 integration tests, differential/properties, zero-allocation checks, общий набор и реальные CLI check/run/atomic-error сценарии проходят.

- Этап 3 завершён 2026-11-21 по спецификации 0.4.0: IR/bytecode/block VM, VM renderer, source diagnostics, differential/properties/compatibility, allocation-free steady-state и benchmark baseline проверены. Все 180 integration tests и общий набор проходят.
- Этап 2 завершён 2026-11-20 по спецификации 0.3.0: Typed AST/lifts, `sine`, interpreter, streaming WAV, `run`, regression-исправления и ручная Beep/atomic/determinism приёмка выполнены. Все 121 integration tests и общий проверочный набор проходят.
- Этап 1 завершён по спецификации 0.1.1: lexer, parser, source diagnostics и `check` прошли приёмку.
- Этап 1A завершён по спецификации 0.2.1: SourceMap/SourceSpan, session/facade, source-aware diagnostics, CLI logical names, reserved words и determinism прошли узкие и общие проверки.

### Compatibility

- Сознательный contract break спецификации 0.2.0 реализован: четыре новых reserved words нельзя использовать как identifiers. Старые fixtures без них сохраняют прежние expected.
- Уточнение 0.2.1 не меняет поведение реализованных `.ln`-программ или diagnostic codes. Оно документирует ограничение нестабильного Rust API: результат и diagnostics нельзя разрешать через независимую `SourceMap`; используйте карту той же session/outcome.
- Миграция `Span` → `SourceSpan` и source-aware API затрагивает только нестабильный внутренний Rust API. Стабильные diagnostic codes, primary byte offsets, fail-fast order и Beep syntax сохранены.
- Включение semantics в `check` является документированным переходом этапа 2, а не ослаблением parser. Тексты diagnostics улучшены без изменения стабильных кодов и byte spans.
- Однофайловый CLI сохраняет streams и exit codes: успешный `check` молчит, source error использует exit `1`, usage/config/UTF-8/file error — exit `2`, internal renderer error — exit `3`. `run` теперь реализован и печатает output report.
- Пользовательские multi-file projects, module/package syntax, `inspect`, stable bytecode ABI, registry и device packages не добавлены. Этап 3 завершён: production IR/bytecode/VM и WAV renderer работают, существующие language/diagnostic/WAV contracts сохранены; benchmark baseline измерен для исправленного sample-major dispatch.

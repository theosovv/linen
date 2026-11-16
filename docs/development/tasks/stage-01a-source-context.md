# Этап 1A — SourceMap, явный контекст и независимые contract tests

- Статус: историческая подготовительная задача завершена; последующая production-реализация 1A также завершена по отдельному явному поручению пользователя агенту.
- Исполнитель подготовительной задачи: агент — SDD-документ, tests и fixtures. Первоначальный план оставлял основной production-код человеку; фактическая миграция затем была целиком делегирована агенту.
- Основной stage document: `docs/development/stages/stage-01a-ecosystem-foundation.md`.

> Этот документ фиксирует историческую подготовительную задачу и её исходный test-facing API. Окончательный статус этапа и уточнение области `SourceId` по `SPEC.md` 0.2.1 находятся в основном stage document; этот файл самостоятельно не разрешает начать этап 2.

## Контекст

- Этап: 1A, SourceMap и явный контекст компиляции.
- Пункты `SPEC.md` 0.2.0: 0, 3, 4.3, 5.1–5.12, 6.1–6.2, 6.5, 7.2–7.3, 7.6, 7.9–7.11, 9 (этап 1A), 10.1, 13–14.
- Связанные ADR: 0001, 0002, 0011, 0012.
- Экосистемные инварианты: `SourceId`/`SourceSpan`, `SourceMap`, явное владение sources/config, logical names, отсутствие скрытого I/O и детерминированный результат.
- Текущее состояние: этапы 0 и 1 завершены по `SPEC.md` 0.1.1. До подготовки этой задачи `cargo test --all-targets --locked` успешно выполнил 47 tests. Production-код всё ещё использует однофайловый `Span`, а `export`, `package`, `from`, `as` остаются identifiers.

Спецификация этапа и ADR 0012 уже приняты. Новые языковые контракты, stage document и ADR в этой задаче не создаются.

## Задача

Подготовить независимые contract tests и fixtures для миграции tokens, AST, diagnostics и high-level facade на source-aware модель. Зафиксировать test-facing Rust API, чтобы production-реализация человека и integration tests использовали один явный seam.

Новые tests до production-миграции ожидаемо не компилируются или выявляют старое поведение. Их нельзя пропускать, удалять или ослаблять ради зелёного промежуточного CI. Подготовительная задача не считается приёмкой этапа 1A.

## Разрешено менять

- `docs/development/tasks/stage-01a-source-context.md`;
- `tests/source.rs`;
- `tests/diagnostics.rs`;
- `tests/lexer.rs`;
- `tests/parser.rs`;
- `tests/cli.rs`;
- `fixtures/valid/stage-01a-*.ln`;
- `fixtures/invalid/stage-01a-*.ln`.

Production-код менять запрещено. `SPEC.md`, ADR, `README.md`, `CHANGELOG.md`, `AGENTS.md`, `Cargo.toml`, `Cargo.lock`, toolchain и другие документы не входят в allowlist. Расширение требует отдельного согласования.

## Нецели

- Реализация `source`, `api`, diagnostics, lexer/parser или CLI в production-коде.
- Placeholders, `todo!()`, `unimplemented!()`, скрытые fallback и временные production-заглушки.
- Пользовательские multi-file projects, module/package grammar и module resolution.
- `ModuleId`, `PackageId`, `ModuleLoader`, virtual filesystem и source provider plugins.
- Manifest, lockfile, registry, network, package cache и package CLI.
- Name resolution, type checker, Typed AST, interpreter, DSP, WAV, IR, bytecode и VM.
- Parser recovery, несколько diagnostics за один `check`, новые dependencies и ignored tests.
- Изменение прежних diagnostic codes, byte offsets, fail-fast порядка или expected без основания в `SPEC.md` 0.2.0.

## Test-facing API для production-миграции

ADR 0012 намеренно не закрепляет точные Rust-сигнатуры. Подготовленные tests используют следующий нестабильный API как механическую конкретизацию принятого решения, а не новый языковой контракт. Если человек выбирает другие сигнатуры, он заранее адаптирует вызовы tests без ослабления assertions и согласует новый seam в этом документе.

```text
SourceId::index() -> usize

SourceMap::new()
SourceMap::add_source(logical_name, text) -> SourceId
SourceMap::get(SourceId) -> Option<&SourceFile>

SourceFile::id() -> SourceId
SourceFile::logical_name() -> &str
SourceFile::text() -> &str
SourceFile::line_column(offset) -> Option<LineColumn>

SourceSpan::new(source, start, end) -> Result<SourceSpan, InvalidSpan>
SourceSpan::empty(source, offset) -> SourceSpan
SourceSpan::{source,start,end,len,is_empty}

lex(source, source_id)
parse(source, source_id)
parse_with_config(source, source_id, config)
parse_tokens_with_config(tokens, source_id, config)
Lexer::new(source, source_id)
Parser::new(tokens, source_id, config)

render_diagnostic(&SourceMap, &Diagnostic)
    -> Result<String, DiagnosticRenderError>

CompileConfig { parser: ParserConfig }
CompileSession::new(config)
CompileSession::{sources,sources_mut}
CompileSession::check(SourceId)
    -> Result<Result<Program, Diagnostic>, SessionError>

check(source) -> CheckOutcome
check_with_config(logical_name, source, config) -> CheckOutcome
CheckOutcome {
    sources: SourceMap,
    result: Result<Program, Diagnostic>,
}
```

`SourceId` выдаётся в порядке добавления начиная с индекса `0`; tests получают unknown ID из независимой карты с большим числом источников, чтобы его индекс отсутствовал в проверяемой карте, и не требуют публичного raw-ID constructor. По уточнению `SPEC.md` 0.2.1 независимые карты могут выдавать равные ID: ID действителен только в исходной карте или её идентичной копии, а foreign same-index ID находится вне контракта API. `LineColumn` остаётся доступным из корня crate, но line index принадлежит `source`, чтобы сохранить направление зависимостей `source ← diagnostic ← syntax ← api`.

`Option` на низкоуровневых `get`/`line_column` явно отличает отсутствующий индекс/offset от успешного lookup, но не доказывает происхождение ID. Renderer преобразует все невалидные source/span связи в typed error и не выбирает другой источник. `SourceSpan::new` отклоняет reversed offsets; renderer дополнительно защищает свои внутренние инварианты.

## Контракт и независимые проверки

| Требование | Проверка | Независимый oracle |
|---|---|---|
| `SPEC.md` §3, §5.8, ADR 0012 — Source identity | Два source с duplicate logical names; независимые maps с одинаковым порядком добавления | Явные IDs `0`, `1`, structured getters |
| §4.3, §6.2 — `SourceSpan` | Обычный, empty, EOF, reversed spans; SourceId во всех assertions | Вручную заданные `(start, end)` |
| §4.3, §6.2 — line index | Unicode scalar columns, LF, CRLF, lone CR, tab, EOF, non-UTF-8 boundary и out-of-bounds | Вручную рассчитанные line/column и offsets |
| §6.2 — multi-source diagnostics | Одинаковые logical names и offsets разрешаются в разный текст по SourceId | Явные source lines и structured spans |
| §5.8, §6.2 — renderer errors | Unknown ID, out-of-bounds, invalid UTF-8 start/end, invalid secondary source | `DiagnosticRenderError`, отсутствие fallback/panic |
| §4.3, §5.8 — source-aware lexer | Все token/EOF spans сохраняют нестандартный SourceId; прежние vectors сохраняют offsets | Явные token vectors и SourceIds |
| §4.3, §6.2 — source-aware parser | Program/declarations/expressions/annotations/durations сохраняют SourceId | Рекурсивный обзор AST и явные offsets |
| §6.2, §7.6 — arbitrary tokens | Первый чужой SourceId даёт `E1100`; empty/missing EOF сохраняют прежний контракт; deterministic corpus не паникует | Явный foreign span и повторный structured result |
| §5.8 — session/facade ownership | AST/diagnostic используются с сохранённой SourceMap; default/explicit logical names; unknown session ID | Явные logical names, коды и renderer result |
| §5.11 — in-process determinism | Повторный valid/invalid `check` | Полное structural equality `Result<Program, Diagnostic>` и rendered errors |
| §6.1, §6.5 — CLI paths/processes | Одинаковый `file_name` в разных parent dirs; два independent processes | Полное equality exit/stdout/stderr; отсутствие parent path |
| §4.3 — reserved words | Четыре exact keywords, prefix/case variants, keyword на месте declaration name | Явные TokenKinds, identifiers и `E1100` primary spans |
| §7.9 — compatibility | Все прежние tests и fixtures | Старые expected плюс SourceId; изменены только сознательно зарезервированные слова |

Для каждой diagnostic проверяются code, SourceId и primary byte span. Expected не вычисляется production lexer/parser/renderer. Multi-source cases используют только synthetic in-memory sources. Tests не используют сеть, время, random без фиксированного seed или внешние аудиофайлы.

## Порядок production-реализации человеком

1. Реализовать `src/source.rs` и провести `tests/source.rs` для SourceMap/SourceSpan/line index.
2. Перевести diagnostics и renderer на SourceMap с typed errors; пройти `tests/diagnostics.rs` и renderer cases в `tests/source.rs`.
3. Передать SourceId через lexer/parser/AST, отклонять смешанные streams и пройти `tests/lexer.rs`, `tests/parser.rs`.
4. Добавить `src/api.rs`, session/outcome и перевести CLI на logical `file_name`; пройти `tests/source.rs`, `tests/cli.rs`.
5. Добавить четыре reserved token kinds и пройти keyword/compatibility cases без module grammar.
6. Выполнить полный DoD и проверки stage document, обновить фактические статусы только после зелёной приёмки.

## Критерии приёмки подготовительной задачи

- [x] Diff ограничен allowlist; production-код не изменён.
- [x] Все прежние assertions сохранены и дополнены SourceId там, где появляется source identity.
- [x] Добавлены positive, negative и boundary cases из матрицы.
- [x] Diagnostics assertions проверяют code, SourceId и primary byte span.
- [x] Нет `#[ignore]`, placeholders, независимых от времени/сети/random входов и production-derived expected.
- [x] Четыре новых reserved words и prefix/case variants имеют lexer/parser/fixture coverage.
- [x] Ожидаемые failures до production-реализации явно записаны; этап 1A не объявлен завершённым.

Окончательный DoD этапа 1A находится только в `docs/development/stages/stage-01a-ecosystem-foundation.md` и этим документом не заменяется.

## Обязательные команды после production-реализации

```bash
cargo test --test source --locked
cargo test --test diagnostics --locked
cargo test --test lexer --locked
cargo test --test parser --locked
cargo test --test cli --locked
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
cargo test --doc --locked
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked
```

Дополнительно вручную проверяются valid Beep, invalid source из двух разных parent directories и два independent CLI processes. Несуществующие `compatibility`, `differential` и `properties` targets не вызываются.

## Отчёт об исполнении

- Изменённые файлы: этот task document; `tests/source.rs`, `tests/diagnostics.rs`, `tests/lexer.rs`, `tests/parser.rs`, `tests/cli.rs`; один valid и четыре invalid `stage-01a` fixtures. Production-код, `SPEC.md`, ADR, README/changelog и Cargo-файлы не изменялись.
- Добавленные проверки и oracles: подготовлено 69 integration tests, включая перенесённые прежние assertions; явные IDs/offsets/locations, structured AST/diagnostics и process output используются как независимые expected. Ignored tests нет.
- Baseline до изменений: `cargo test --all-targets --locked` — 47 passed, 0 failed, 0 ignored.
- `cargo fmt --check` и `git diff --check` — успешно.
- `cargo test --test source --locked`, `cargo test --test diagnostics --locked`, `cargo test --test lexer --locked`, `cargo test --test parser --locked` — ожидаемо не компилируются: production ещё не экспортирует SourceMap/SourceSpan/session API и не имеет четырёх token kinds. Механическая ошибка подготовки в двух parser assertions была исправлена и повторная проверка оставила только ошибки отсутствующего API.
- `cargo test --test cli --locked` — 11 passed, 1 failed; regression test показывает, что текущий CLI выводит разные absolute parent paths вместо одинакового logical `file_name`.
- `cargo clippy --lib --bins --locked -- -D warnings` — успешно для неизменённого production-кода.
- `cargo clippy --all-targets --locked -- -D warnings` и `cargo test --all-targets --locked` — ожидаемо останавливаются на отсутствующем source-aware API в новых tests; полный набор не считается пройденным.
- `cargo test --doc --locked` — 1 passed, 0 failed; `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked` — успешно для текущего production API.
- Read-only I/O review: filesystem, environment и console calls найдены только в `src/main.rs`; текущие `diagnostic` и `syntax` скрытого I/O не содержат. `HashMap`, system time, network и random в production не используются.
- Найденные дефекты: относительно `SPEC.md` 0.2.0 отсутствуют SourceId/SourceMap/session, source-aware renderer и четыре reserved words; CLI source diagnostic содержит физический путь. Это подтверждает уже специфицированный объём 1A, а не новый контракт.
- Оставшиеся ограничения на момент этой подготовительной задачи: production-реализация, зелёная приёмка всех 69 tests, общий clippy/test suite и обновление фактического статуса этапа ещё не были выполнены. Последующая миграция была отдельно поручена агенту; её итоговый отчёт находится в `../stages/stage-01a-ecosystem-foundation.md`.

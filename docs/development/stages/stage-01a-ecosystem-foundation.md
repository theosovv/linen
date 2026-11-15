# Этап 1A — SourceMap и явный контекст компиляции

- Статус: спецификация принята 2026-11-16; production-реализация и новые tests не начаты.
- Блокирует: этап 2 — type checker, наивный интерпретатор и Beep WAV.
- Источник требований: `SPEC.md` 0.2.0, разделы 0, 2.3–3, 4.3, 5.1–5.12, 6.1–6.2, 6.5, 7.2–7.3, 7.6, 7.9–7.11, 9 (этап 1A), 10.1, 13–14.
- Дополнительный контекст: `docs/linen — подготовка к экосистеме.md`, разделы 3.3, 4.1–4.8, 9 (E0), 10.1, 11 и 13.
- Связанные ADR: `0001`, `0002`, `0011`, `0012`.

## 1. Зачем нужен корректирующий этап

Этапы 0 и 1 завершены по `SPEC.md` 0.1.1. Их исходные DoD и фактические проверки остаются действительными историческими результатами. Спецификация 0.2.0 добавила новые требования после этой приёмки; они не считаются выполненными задним числом.

До появления Typed AST и IR нужно устранить однофайловую модель positions и сделать владение компиляционным контекстом явным. Это дешевле, чем менять offsets на source-aware positions после появления resolver, lowering, bytecode debug info и compatibility artifacts.

Этап 1A не является реализацией модульной системы. Пользователь по-прежнему проверяет один `.ln`-файл командой `linen check`; несколько источников создаются только synthetic in-memory tests для проверки модели.

## 2. Подтверждённые расхождения текущего кода с целевой архитектурой

| Текущее состояние | Требуемое состояние после 1A |
|---|---|
| `src/diagnostic.rs`: `Span` содержит только `start/end` | `SourceSpan` дополнительно содержит `SourceId` |
| Отдельные `SourceId`, `SourceFile`, `SourceMap` и line index отсутствуют | Базовый `source` module владеет этими реальными типами |
| `src/syntax/lexer.rs` и `parser.rs` создают spans из `&str`/tokens без source context | Lexer/parser получают ожидаемый `SourceId`; все spans его сохраняют |
| `render_diagnostic` получает отдельные `source_name` и `source` | Renderer разрешает primary span через сохранённую `SourceMap` |
| `check(&str)` возвращает diagnostic без владеющей карты источников | Source-level outcome сохраняет `SourceMap` вместе с `Result`, либо caller удерживает session |
| `src/lib.rs` не имеет compile session/config owner | Реальный `CompileSession` владеет `SourceMap` и `CompileConfig` |
| CLI передаёт displayed physical path в renderer | Source diagnostic использует logical `file_name`; physical path остаётся в файловой `E5000` |
| `export`, `package`, `from`, `as` лексируются как identifiers | Четыре слова имеют собственные reserved token kinds |

Production-код в задаче подготовки этой спецификации не изменялся.

## 3. Ожидаемый результат

После завершения 1A:

- один файл и несколько synthetic sources используют одну `SourceMap`-модель;
- tokens, AST и diagnostic labels содержат `SourceSpan` с правильным `SourceId`;
- diagnostics не смешивают источники по одинаковым offsets или logical names;
- `CompileSession` явно владеет sources/config; простой source-level facade остаётся удобным и не теряет карту источников;
- lexer/parser/type-independent compiler code не выполняет filesystem, network, environment, stdout, system-time или current-directory I/O;
- fail-fast diagnostics, byte offsets, Unicode/CRLF locations и CLI exit codes сохраняются;
- результат одинакового `check` детерминирован в одном и разных processes;
- четыре новых reserved words отклоняются на месте identifier без добавления module/package grammar;
- этап 2 может вводить resolved IDs и Typed AST `SourceSpan` без второй миграции однофайлового представления.

## 4. Проект allowlist для отдельной задачи реализации

Этот список является частью спецификации будущей задачи, а не самостоятельным разрешением агенту менять production-код. Исполнение требует отдельного явного поручения с повторением allowlist и ролей.

### Основной production-код — человек

- `src/source.rs` — новый базовый source module;
- `src/api.rs` — реальный `CompileSession`, source-level outcome/facade;
- `src/diagnostic.rs`;
- `src/syntax/**`;
- `src/lib.rs`, `src/main.rs`.

### Tests и fixtures — агент по отдельным задачам

- `tests/source.rs` и непосредственно необходимые helpers;
- `tests/diagnostics.rs`, `tests/lexer.rs`, `tests/parser.rs`, `tests/cli.rs`;
- `fixtures/valid/stage-01a-*.ln`, `fixtures/invalid/stage-01a-*.ln`;
- дополнительный test-support только при доказанной необходимости.

### Документация после фактической реализации

- `README.md`, `CHANGELOG.md`, `AGENTS.md`;
- `docs/development/stages/stage-01a-ecosystem-foundation.md`;
- другие документы только для устранения фактического расхождения.

`SPEC.md` и ADR не меняются во время реализации без нового согласования. `Cargo.toml`, `Cargo.lock`, toolchain и dependencies не входят в задачу. Новые `sema`, `ir`, `bytecode`, `vm`, `dsp`, `render` и module/package directories не создаются.

## 5. Нецели

- Пользовательские multi-file projects и module graph.
- Синтаксис `mod`, `use`, `export`, `from`, `as`, `package`, visibility rules и module resolution.
- `ModuleId`, `PackageId`, `ModuleLoader`, virtual filesystem, source provider plugins.
- Manifest, lockfile, content hashes, cache, semver resolver, registry, HTTP и cryptography.
- `linen add/install/publish`, native dynamic loading и общий plugin framework.
- Name resolution, type checker, Typed AST, interpreter, DSP, WAV, IR, bytecode и VM.
- `BindingId`, `FunctionId`, `BuiltinId`, `NodeOrigin`, stable bytecode ABI и device packages.
- Recovery и несколько source diagnostics за один `check`.
- Массовый рефакторинг tests в snapshots или изменение старых diagnostic codes/expected без отдельного основания.

## 6. Принятые контракты реализации

### 6.1. Source model

Концептуальные сущности:

```text
SourceId
  компактный типизированный ID внутри одной CompileSession

SourceFile
  id: SourceId
  logical_name: String
  text: String
  line_index: LineIndex

SourceSpan
  source: SourceId
  start: usize
  end: usize

SourceMap
  упорядоченное хранилище SourceFile

CompileConfig
  parser: ParserConfig
```

Правила:

1. `SourceMap` выдаёт IDs детерминированно в порядке добавления. ID не вычисляется из path/name/hash/random, не является глобальным и не сериализуется как package/module identity.
2. `SourceMap` разрешает источник только по `SourceId`, а не по logical name. Duplicate logical names допустимы для synthetic sources.
3. `SourceSpan` сохраняет существующий полуинтервал `[start, end)` и `usize` offsets, чтобы не вводить отдельный source-size contract. Конструктор отклоняет `start > end`; empty и EOF spans допустимы.
4. Проверка принадлежности ID конкретной `SourceMap`, `start/end <= text.len()` и UTF-8 boundaries выполняется при lookup/location/rendering. Неизвестный ID/offset не подменяется первым источником и не вызывает panic.
5. `logical_name` является диагностическим именем, а не canonical/absolute path. Правила module logical names откладываются до соответствующего ADR.
6. Line index сохраняет текущие правила: 1-based line/column, Unicode scalar columns, LF и CRLF как один перевод, lone CR как whitespace, tab как одна колонка.
7. Source module не читает файлы и не зависит от syntax, CLI или packages.

### 6.2. Tokens, AST и diagnostics

1. Все `Token`, `Program`, declarations, expressions, identifiers, type annotations, durations и diagnostic labels используют `SourceSpan` вместо `Span`.
2. Lexer получает `SourceId` и `&str`; parser получает ожидаемый `SourceId` даже для empty/arbitrary token stream. Отсутствие EOF обрабатывается по прежнему контракту.
3. Первый token с чужим `SourceId` даёт `E1100` с primary span этого token. Объединять spans разных источников запрещено; такая ситуация не должна приводить к panic или молчаливому выбору ID.
4. Старые primary/secondary spans `E1000`–`E1103`, `E2002`–`E2004`, порядок structural checks и fail-fast остаются прежними, но дополняются `SourceId`.
5. Renderer получает `SourceMap` и выбирает источник primary span по ID. Неизвестный ID, invalid/reversed/out-of-bounds offsets и non-UTF-8 boundary возвращают явную `DiagnosticRenderError` либо эквивалентную typed error; silent fallback запрещён.
6. Внутренний Rust API может измениться, поскольку он нестабилен. Стабильные diagnostic codes и их смысл не меняются.

### 6.3. Session, facade и владение SourceMap

1. `CompileSession` находится в высокоуровневом `api` и владеет `SourceMap` и `CompileConfig`. Пока конфигурация содержит только реально существующий `ParserConfig`.
2. Session-based check принимает `SourceId` уже добавленного источника и возвращает `Result<Program, Diagnostic>`. Caller сохраняет session до использования AST/diagnostic и renderer. Unknown ID на этой boundary возвращает typed session error, а не source diagnostic с неразрешимым span.
3. Простой `check(source)` создаёт одноисточниковую session, но не возвращает голый diagnostic с потерянной картой. Source-level `CheckOutcome` (или эквивалентный явно названный тип) владеет `SourceMap` и `Result<Program, Diagnostic>` вместе. Default logical name — `input.ln`.
4. Source-aware вариант принимает явные logical name, source text и config. Физический путь в библиотечный compile API не передаётся.
5. Отдельный `DiagnosticSink` не создаётся: fail-fast использует `Result`. `SymbolStorage`, resolved IDs и поля будущих стадий появляются только на этапе 2 и позднее.
6. Нет глобального current source, mutable interner, random, environment config или cache между sessions.

### 6.4. CLI

1. `main.rs` читает UTF-8 файл один раз на host boundary и передаёт logical name/text в session. Compiler stages файловый путь не получают.
2. Для однофайлового source diagnostic logical name — `path.file_name()` в display-safe UTF-8 представлении. Absolute/canonical path и parent directories не попадают в source diagnostic или semantic artifact.
3. `E5000` при чтении может показывать фактический путь: это host I/O error, а не diagnostic внутри источника.
4. Успешный `check` ничего не печатает. Source error — stderr/exit `1`, usage/UTF-8/file error — exit `2`, невозможный внутренний renderer invariant — понятная internal error/exit `3`, не panic/101.
5. `run`, `inspect` и package/module commands не добавляются.

### 6.5. Reserved words

Lexer добавляет token kinds `Export`, `Package`, `From`, `As` для точных слов:

```text
export
package
from
as
```

- Они резервируются наравне с существующими future keywords и не получают grammar productions.
- На месте identifier parser возвращает обычный `E1100` с primary span keyword.
- `exported`, `package1`, `from_`, `as2` и варианты с другим регистром остаются identifiers по существующим правилам.
- Существующие fixtures на момент подготовки не содержат этих слов как declarations/identifiers. Старые programs без них должны сохранить expected; использующие их программы являются сознательным compatibility break `SPEC.md` 0.2.0.

### 6.6. Детерминизм и I/O

- При одинаковых logical name, UTF-8 source и config совпадают AST/diagnostics и их порядок.
- Повторная проверка выполняется в одном process и минимум в двух отдельных CLI processes.
- Разные physical parent directories при одинаковом logical `file_name` не меняют source diagnostic.
- Absolute path, current directory, directory listing, system time, locale, hostname, network и `HashMap` iteration не влияют на результат.
- При появлении новой коллекции output порядок задаётся стабильным ключом; в 1A не добавляется несортированный serialized artifact.
- `SourceId` не влияет на языковую семантику; аудио ещё не реализуется и не добавляется ради проверки.

## 7. Порядок реализации человеком

1. **Source module:** реализовать source identity, SourceMap, SourceSpan и line index. Узкая проверка: `cargo test --test source --locked`.
2. **Diagnostics:** перевести labels/location/renderer на SourceSpan/SourceMap и typed rendering errors. Проверка: `cargo test --test diagnostics --locked`.
3. **Syntax:** передать SourceId через lexer/parser/AST, обработать смешанные arbitrary tokens и сохранить прежние spans. Проверки: `cargo test --test lexer --locked`, `cargo test --test parser --locked`.
4. **Session/facade/CLI:** добавить реальный session и outcome, сохранить SourceMap до renderer, перевести CLI на logical `file_name`. Проверка: `cargo test --test cli --locked` и ручные valid/invalid cases.
5. **Reserved words:** добавить четыре token kinds и keyword/compatibility cases, не меняя EBNF.
6. **Приёмка:** выполнить все узкие и общие проверки, удалить созданные artifacts, обновить фактический статус документов и только затем разрешить этап 2.

Агентские tests могут готовиться заранее отдельными задачами с независимыми expected. Агент не реализует основной source/session/parser код, если это не поручено отдельно человеком.

## 8. Матрица проверки

| Требование | Positive/negative/boundary cases | Независимый oracle |
|---|---|---|
| `SourceId` identity | Два источника, duplicate logical names, одинаковые offsets | Явно созданные IDs/sources; сравнение structured values |
| Детерминированная выдача ID | Одинаковый порядок добавления в двух sessions | Явный ожидаемый порядок, не random/hash |
| `SourceSpan` | Обычный, empty, EOF, reversed, out-of-bounds, non-UTF-8 boundary | Вручную заданные offsets и ожидаемые errors |
| Line index | Unicode, LF, CRLF, lone CR, tab, EOF | Вручную рассчитанные line/column |
| Source-aware tokens/AST | Все node/declaration/EOF spans | Явные `(SourceId, start, end)` vectors |
| Смешанные tokens | Чужой SourceId, missing EOF, пустой stream | `E1100` на первом чужом token либо прежний контракт; отсутствие panic |
| Diagnostic structure | Primary/secondary labels в одном и разных synthetic sources | Явные codes и SourceSpans |
| Renderer | Правильный primary source, unknown ID, invalid offset | Явный logical name/location или typed error |
| Session ownership | Diagnostic/AST используются после return из facade | SourceMap остаётся доступной; renderer разрешает каждый span |
| CLI logical name | Два разных parent dirs, одинаковый `file_name` | Одинаковый stderr без absolute parent |
| Reserved words | Четыре exact keywords и prefix/case variants | Явные token kinds/identifiers и `E1100` spans |
| In-process determinism | Повторный одинаковый `check` | Полное structural equality результата |
| Cross-process determinism | Два независимых CLI запуска | Полное равенство exit/stdout/stderr для одного logical input |
| Отсутствие скрытого I/O | Read-only review разрешённых compiler modules | Нет fs/env/network/time/stdout/current-dir calls |
| Compatibility | Все прежние valid/invalid fixtures и contract tests | Старые expected, кроме документированных reserved-word cases |

Для diagnostics assertions обязательны code, `SourceId` и primary byte span. Snapshot текста сам по себе недостаточен. Expected не вычисляется production lexer/parser/renderer.

## 9. Definition of Done

- [ ] Source module и все типы реализованы без placeholders и filesystem/package dependencies.
- [ ] Один файл проходит через SourceMap; минимум два synthetic source различаются по ID.
- [ ] Tokens/AST/diagnostic labels используют SourceSpan; прежние byte offsets и codes сохранены.
- [ ] Смешанный SourceId и invalid renderer inputs обрабатываются явно без panic.
- [ ] Session/outcome сохраняет SourceMap столько, сколько используются AST/diagnostics.
- [ ] CLI использует logical `file_name`; файловый E5000 и exit codes не регрессировали.
- [ ] Четыре reserved words и prefix/case boundaries покрыты; module/package grammar отсутствует.
- [ ] In-process и cross-process determinism tests проходят.
- [ ] Все прежние fixtures, не затронутые сознательным keyword break, сохраняют expected.
- [ ] Нет новых dependencies, ignored tests, network/time/random dependencies и скрытых fallback.
- [ ] Нет ModuleLoader, manifest, registry, package CLI, plugin framework и future-stage modules.
- [ ] Узкие suites и общий набор действительно запущены и прошли.
- [ ] README/AGENTS/changelog описывают уже фактически реализованное состояние; этап 2 не начат в том же diff.

## 10. Обязательные команды приёмки

После появления соответствующих test targets:

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

Дополнительно вручную проверяются valid Beep, invalid source из двух разных parent directories и два independent CLI processes. Несуществующие `compatibility`, `differential` и `properties` targets заранее не вызываются; прежняя compatibility обеспечивается существующими contract tests/fixtures.

## 11. Текущий статус проверок

- Production-реализация 1A: **не выполнялась**.
- Новые tests/fixtures 1A: **не добавлялись**.
- Приёмочные команды 1A: **не запускались как приёмка**.
- Открытых выборов по зарезервированным словам нет: все четыре утверждены.
- После реализации этот раздел заменяется фактическими командами/результатами, найденными дефектами и оставшимися ограничениями; статус меняется только при полном DoD.

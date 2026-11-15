# 0012. SourceMap, SourceSpan и явный CompileSession

- Статус: принят.
- Дата: 2026-11-15.
- Заменяет: часть ADR 0002 о представлении source positions только байтовыми offsets.
- Дополняет: ADR 0001 и 0011.

## Контекст

Этапы 0 и 1 реализовали `Span(start, end)`, tokens/AST и diagnostics для одного источника. Parser получает `&str` и не выполняет I/O, а CLI отдельно читает файл. Этого достаточно для однофайлового `check`, но offsets нельзя связать с источником после появления зависимостей, а физический путь не должен становиться семантической идентичностью модуля.

`SPEC.md` 0.2.0 и документ подготовки к экосистеме требуют сохранить source identity через compiler stages и явно владеть контекстом компиляции. При этом module syntax, package loader, virtual filesystem, registry и plugin framework не относятся к текущей задаче.

## Варианты

1. Оставить offsets и добавлять имя файла только при выводе. Минимальный diff, но diagnostics из разных источников неразличимы, а дальнейшая миграция AST/Typed AST/IR становится дороже.
2. Ввести `SourceId`, `SourceFile`, `SourceMap`, `SourceSpan` и одноисточниковый `CompileSession`, не добавляя загрузчик модулей. Решает текущую проблему и имеет небольшой объём.
3. Сразу ввести `ModuleLoader`, virtual filesystem, package IDs и source provider plugins. Даёт ненужные сейчас abstractions и смешивает name resolution с загрузкой.

## Решение

Выбран второй вариант.

### Источники и spans

- Новый базовый модуль `source` владеет типами `SourceId`, `SourceFile`, `SourceMap`, `SourceSpan` и line index. Он не зависит от syntax, CLI или файловой системы.
- `SourceId` — типизированный компактный ID, выдаваемый `SourceMap` в детерминированном порядке добавления. Он уникален и стабилен только внутри одной компиляции, не сериализуется как глобальная идентичность и не вычисляется из пути или random.
- `SourceFile` содержит ID, UTF-8 `logical_name`, исходный `String` и line index. `SourceMap` хранит источники упорядоченно и разрешает `SourceId` без использования имени как identity.
- Логическое имя является диагностическим именем, а не физическим путём. Synthetic sources могут иметь одинаковые имена; их spans всё равно различаются по `SourceId`. Правила module/package logical names фиксируются будущими ADR, а не этим решением.
- `SourceSpan` содержит `SourceId` и `usize` offsets `[start, end)`. Сохранение `usize` не меняет существующий контракт размера входа; явная ширина нужна только будущим сериализуемым форматам и выбирается их ADR. Конструктор отклоняет `start > end`.
- Принадлежность `SourceId` конкретной `SourceMap`, границы offsets и UTF-8 boundaries проверяются при разрешении source/line/column. Неизвестный источник и невалидный offset возвращают явную ошибку lookup/rendering, а не panic и не подставленный источник по умолчанию.
- Tokens, AST и diagnostic labels используют `SourceSpan`. Объединять offsets в один span можно только внутри одного `SourceId`. Низкоуровневый parser получает ожидаемый `SourceId` явно, включая разбор пустого/arbitrary token stream; чужой token отклоняется детерминированным `E1100`.
- Line index и вычисление колонок сохраняют правила ADR 0011: Unicode scalar values, LF/CRLF как один перевод, lone CR как whitespace, tab как одна колонка.

### Session и граница загрузки

- `CompileSession` появляется в высокоуровневом `api`, поскольку теперь существует реальная композиция sources, `CompileConfig` и `check`. Пустые module/package/symbol abstractions не создаются.
- На этапе 1A session владеет только `SourceMap` и фактически нужной конфигурацией. `CompileConfig` содержит `ParserConfig`; поля будущих стадий заранее не добавляются.
- Fail-fast diagnostic возвращается через `Result`. Отдельный `DiagnosticSink` появляется только вместе с recovery или несколькими diagnostics; пустой sink сейчас не создаётся. `SymbolStorage` появляется с resolver на этапе 2.
- Session-based check принимает ID уже добавленного источника. Caller сохраняет session до использования AST/diagnostic; unknown ID на этой boundary возвращает typed session error, а не source diagnostic с неразрешимым span.
- Простой `check(source)` остаётся facade: он создаёт session с одним in-memory источником и default logical name `input.ln`. Его `CheckOutcome` (или эквивалентный тип) владеет `SourceMap` вместе с `Result<Program, Diagnostic>`, поэтому источник не теряется при возврате. Source-aware API позволяет передать явное logical name и конфигурацию. Lexer/parser по-прежнему не открывают файлы.
- CLI один раз читает файл на host boundary, передаёт текст в session и сохраняет её до renderer. Для source diagnostic однофайлового CLI logical name формируется из `file_name`, а не из absolute/canonical path. Файловая ошибка `E5000` может показывать фактический путь, потому что относится к host I/O, а не к semantic artifact.
- Renderer получает `SourceMap` и выбирает источник по `SourceId` из primary span. Невозможность разрешить источник/offset является явной ошибкой renderer/internal invariant, а не поводом для panic пользовательского исходника.

### Детерминизм

- Ни `SourceId`, ни diagnostics, ни будущие semantic artifacts не зависят от absolute path, directory listing, system time, locale, network, random или `HashMap` iteration.
- Порядок fail-fast проверок этапа 1 остаётся неизменным. При появлении коллекций diagnostics/artifacts порядок задаётся стабильным ключом до вывода или сериализации.
- Reserved words `export`, `package`, `from`, `as` добавляются в lexer согласно `SPEC.md` §4.3. Это не вводит `use`, `mod`, `export` или package grammar.

## Последствия

- Внутренний Rust API этапа 1 изменится: `Span` станет `SourceSpan`, source-aware lexer/parser и renderer получат дополнительный контекст. Этот API заранее объявлен нестабильным; языковые diagnostic codes и byte offsets сохраняются.
- Однофайловые программы сохраняют прежние AST, diagnostics и CLI semantics, кроме сознательного резервирования четырёх слов и нестабильного оформления displayed source name.
- Два synthetic источника можно различать уже на этапе 1A, хотя пользовательский multi-file project ещё отсутствует.
- На этапе 2 resolver обязан вводить `BindingId`, `FunctionId` и `BuiltinId`; повторный строковый lookup как идентичность не закрепляется. Typed AST сохраняет `SourceSpan`.
- На этапе 3 IR получает `NodeOrigin`; persisted bytecode не становится стабильным ABI, а любой сохраняемый формат получает собственную версию.
- Будущий resolver не получает права загружать файлы: loading остаётся на host/project boundary.

## Проверка

- SourceMap tests создают два и более in-memory источника, включая одинаковые logical names, и проверяют независимость одинаковых offsets по `SourceId`.
- Boundary tests покрывают reversed/empty/EOF spans, unknown IDs, invalid UTF-8 offsets, Unicode, LF/CRLF, lone CR и tab.
- Lexer/parser/diagnostic tests проверяют `SourceId` во всех spans и детерминированный `E1100` для смешанного token stream без panic.
- Renderer tests выбирают primary source по ID и явно отклоняют неизвестный ID/invalid offset.
- Contract tests повторяют одинаковый `check` в одном process и в разных processes; результаты и порядок diagnostics совпадают.
- Keyword vectors различают четыре reserved words и identifiers с теми же префиксами. Старые fixtures, не использующие эти слова как имена, проходят без изменения expected.
- Общий Rust-набор и ручной CLI `check` подтверждают отсутствие регрессий. Никакие module/package tests или network operations не добавляются.

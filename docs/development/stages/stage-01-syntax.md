# Этап 1 — Lexer, parser и синтаксические diagnostics

- Статус: завершён 2026-11-15 по `SPEC.md` 0.1.1; реализация, исправления ревью и исходные проверки завершены.
- Источник исходных требований: `SPEC.md` 0.1.1, разделы 4.3, 5.1–5.4, 6.1–6.2, 7.3, 7.6–7.7, 9 (этап 1), 10.1.
- Связанные ADR: `0001`, `0002`, `0011`; актуализация source model — `0012`.

> Это исторический отчёт этапа. Спецификация 0.2.0 последовательно добавила `SourceSpan`/`SourceMap`, session, детерминированные multi-source tests и reserved words `export`, `package`, `from`, `as`. До корректирующего этапа код соответствовал описанному ниже однофайловому представлению и принимал эти четыре слова как identifiers. Миграция перед этапом 2 завершена в `stage-01a-ecosystem-foundation.md`; исходные DoD и команды ниже не переписываются задним числом.

## Ожидаемый результат

`linen check input.ln` выполняет lexer, recursive descent parser и структурные проверки Beep. Успешный check ничего не печатает; syntax diagnostic содержит стабильный код и точный primary span, а CLI выводит её в stderr с `path:line:column` и возвращает код `1`. Type checking, name resolution, интерпретатор и звук остаются на этапе 2.

## Объём и allowlist подготовительной задачи

Пользователь явно разрешил:

- `SPEC.md`, `CHANGELOG.md`, `README.md`;
- `docs/development/**`, `docs/adr/**`;
- `src/lib.rs`, `src/main.rs`, `src/diagnostic.rs`, `src/syntax/**`;
- `tests/**`, `fixtures/**`.

В этой задаче специально разрешены временные `todo!()` в обозначенных entry points. Это исключение из обычного запрета заглушек не означает готовность этапа или пригодность `check` для использования.

## Фактическое состояние реализации

Реализованы:

- типы `Token`, `TokenKind`, AST и `ParserConfig` с defaults `max_depth = 128`, `max_nodes = 10_000`;
- ручной lexer с UTF-8 byte spans, malformed-literal diagnostics и гарантированным progress;
- recursive descent parser, структурные проверки `out`/`render` и оба лимита;
- Unicode/CRLF line/column и source diagnostic renderer;
- сквозной `linen check`, чтение UTF-8 файла и файловая ошибка `E5000` с exit code `2`;
- valid/invalid fixtures и contract tests с независимо заданными expected.

Временные `todo!()` удалены. После ревью исправлены primary spans `E1101`/`E1103`, primary/secondary spans `E2003` и инвариант конструирования `Span`; соответствующие regression-тесты добавлены.

## Принятый дизайн представлений

- `Token` хранит `TokenKind` и байтовый `Span`; EOF представлен явным token с пустым span.
- Числа и durations представляются как `f64`; syntactically valid infinity не является `E1001`.
- `Program` содержит упорядоченные `LetDecl` и финальный `RenderDecl`.
- `ExprKind` различает числа, идентификаторы, именованные вызовы, unary/binary операции и `Group`. Скобки сохраняются как отдельный AST-узел и входят в лимит узлов.
- AST сохраняет spans идентификаторов, типов, длительностей, выражений и целых declarations. Span declaration включает `;`, span выражения-вызова — закрывающую `)`, span группы — обе скобки.
- Rust API syntax является нестабильным внутренним API для библиотечных тестов; стабильными являются языковые и diagnostic contracts, а не layout структур.

## Границы `check` на этапе 1

Выполняются:

- лексические ошибки `E1000`/`E1001`;
- грамматика и приоритеты, `E1100`/`E1101`;
- лимиты `E1102`/`E1103`;
- ровно один финальный `render`, ровно один `out`: `E2002`–`E2004`.

Не выполняются:

- `E2000` для обычных повторных имён, `E2001` и проверка forward references;
- типы, lift, `E2100`–`E2104`;
- проверка положительности и sample count длительности;
- `run`, `inspect`, IR, VM, DSP и WAV.

## Порядок реализации человеком

1. **Lexer:** заполнить `Lexer::tokenize`, сохраняя UTF-8 boundaries и гарантированный progress. Проверка: `cargo test --test lexer --locked`.
2. **Parser:** реализовать recursive descent, structural checks и оба лимита. Проверки: `cargo test --test parser --locked`, затем `cargo test --test lexer --test parser --locked`.
3. **Diagnostics:** реализовать Unicode/CRLF locations и renderer в `diagnostic.rs`, не перенося source rendering в CLI. Проверка: `cargo test --test diagnostics --locked`.
4. **CLI:** завершить сквозной `check`, не добавляя compiler logic в `main.rs`. Проверка: `cargo test --test cli --locked` и ручной запуск valid/invalid fixtures.
5. Удалить все временные `todo!()`, выполнить общий набор и только затем менять статус этапа на завершённый.

## Матрица проверки

| Требование | Проверка | Независимый oracle |
|---|---|---|
| Числа, durations, keywords | `tests/lexer.rs`: явные token vectors | Значения и offsets перечислены вручную |
| Whitespace, LF/CRLF, comments | Token и EOF spans | Независимо рассчитанные byte offsets |
| Malformed input | `E1000`/`E1001` и primary span | Явные negative vectors |
| Приоритет и ассоциативность | Semantic assertions по AST | Ожидаемое дерево из EBNF |
| Calls и скобки | Аргументы, `Group`, неименованный callee | Явные структуры и offending token spans |
| `out`/`render` | Positive, missing, duplicate, misplaced | Код и primary/secondary spans из §6.2 |
| Отложенная семантика | `stage-01-semantics-deferred.ln` принимается | Разделение этапов 1 и 2 |
| Лимиты | Принять `N`, отклонить `N+1` | Независимо посчитанные depth/nodes |
| Malformed source/token streams | Фиксированный seed, 64 Unicode sources и 64 token streams | Инвариант «нет panic», success или diagnostic |
| Line/column | Unicode, CRLF, lone CR, tab, invalid offsets | Явные пары line/column |
| CLI | Success, source error, file error, usage | Streams и exit codes из §6.1 |

Fixtures находятся в `fixtures/valid/stage-01-*.ln` и `fixtures/invalid/stage-01-*.ln`. Expected не вычисляется production lexer/parser.

## Definition of Done этапа

- [x] Все примеры Beep разбираются в ожидаемую структуру.
- [x] Обязательные lexer/parser cases из `SPEC.md` §7.3 покрыты и проходят.
- [x] Diagnostics содержат код и корректный primary span.
- [x] CRLF и комментарии не изменяют позиции ошибочно.
- [x] Parser не паникует на property corpus и не зависает на malformed input.
- [x] Лимиты глубины и узлов работают на границах `N`/`N+1`.
- [x] `check` не выполняет type checking, name resolution или звук.
- [x] В production нет временных `todo!()` и скрытых fallback.
- [x] Полный общий набор проверок проходит без ignored tests.
- [x] README описывает только реально работающий `check`.

## Фактические проверки

После реализации и исправлений ревью локально 2026-11-15:

- `cargo fmt --check` — успешно.
- `cargo clippy --all-targets --locked -- -D warnings` — успешно, без предупреждений.
- `cargo test --all-targets --locked` — успешно: 47 integration tests, 0 failed, 0 ignored.
- `cargo test --doc --locked` — успешно: 1 doctest.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked` — успешно.
- Ручные запуски `check`: valid Beep завершился с кодом `0` без вывода; missing semicolon дал `E1101` и `2:1`; duplicate `out` дал `E2003` и `2:5`.

До изменений этапа 1 повторно подтверждён DoD этапа 0: `cargo fmt --check`, clippy, 10 integration tests, doctest и rustdoc прошли.

Проверки подготовленного каркаса выполнены локально 2026-11-11:

- `cargo fmt --check` — успешно.
- `cargo clippy --all-targets --locked -- -D warnings` — успешно, без предупреждений.
- `cargo test --all-targets --locked --no-run` — успешно: все test targets компилируются.
- `cargo test --test lexer --locked` — 0 passed, 11 failed; все достигают `Lexer::tokenize`.
- `cargo test --test parser --locked` — 0 passed, 16 failed; source-based cases достигают lexer, arbitrary-token case — `Parser::parse_program`.
- `cargo test --test diagnostics --locked` — 6 passed, 2 failed; failures относятся к `line_column`.
- `cargo test --test cli --locked` — 7 passed, 3 failed; help/version, usage, missing file и invalid UTF-8 работают, три `check` cases достигают lexer.
- `cargo test --all-targets --locked` — не проходит и останавливается на первом failing target (`cli`); это не пройденная приёмка.
- `cargo test --doc --locked` — успешно: 1 doctest.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked` — успешно.

Всего в узких suites 32 failures связаны с четырьмя задокументированными заглушками; ignored tests нет. Признаков неправильных expected или других production-дефектов на текущем каркасе не обнаружено, но корректность contract tests окончательно проверяется только на реальной реализации. Failing tests не считаются пройденной приёмкой этапа.

# Этап 4 — Функции, блоки и pipeline

- Статус: завершён 2026-11-21; production functions/blocks/pipeline, полный DoD и финальная приёмка выполнены.
- Источник требований: `SPEC.md` 0.5.0, разделы 0, 2.2, 3, 4.3–4.5, 4.8, 5.1–5.11, 6.1–6.5, 7, 8, 9 (этап 4), 10–14.
- Связанные ADR: `0007`; сохраняются `0001`–`0006`, `0009`, `0011`, `0012`.
- Экосистемные инварианты: source-aware origins, compilation-local resolved IDs, deterministic expansion, явный session/config, отсутствие скрытого I/O и module/package infrastructure.
- Блокирует: этап 5 и последующие языковые этапы.

## 1. Контекст

Этап 3 завершён: все 180 integration tests зелёные, production render использует verified IR/bytecode/block VM, steady-state allocations и benchmark задокументированы. Этап 4 расширяет язык, но не меняет bytecode execution model:

```text
functions/blocks/pipeline
        ↓ parse + semantic checks
Typed AST with FunctionId/BindingId
        ↓ checked graph-construction instantiation
canonical IR + eager roots + call origins
        ↓ существующие codegen/verifiers/VM
WAV
```

Пользователь выбрал подготовительную задачу агента. Агент фиксирует SDD-контракты, механические representations и независимые tests; человек реализует parser, resolver/type checking и function lowering.

## 2. Ожидаемый результат после завершения

- Работают top-level функции с обязательными сигнатурами, expression/body blocks и locals.
- Pipeline эквивалентен явному вызову и сохраняет точные spans.
- `BindingId` охватывает top-level/params/locals; `FunctionId` однозначно связывает вызовы с определениями.
- Ранние top-level values/functions доступны телу; forward/self/indirect recursion и higher-order values отклоняются.
- Аргументы и `let` вычисляются один раз и обозначают sharing; тело каждого вызова получает свежие state slots.
- Eager unused arguments/locals сохраняют runtime errors через roots.
- Compile-time `Float` корректно конкретизируется для каждого вызова; diagnostics содержат определение и call chain.
- Expansion limits исключают неограниченный рост compile-time graph construction.
- Все прежние programs, diagnostics, WAV, block-invariance, reset и zero-allocation contracts сохраняются.

## 3. Объём и allowlist подготовительной задачи

### Документация

- `SPEC.md`, `CHANGELOG.md`, `README.md`, `AGENTS.md`;
- `docs/**`.

### Production-каркас

- `src/api.rs`, `src/diagnostic.rs`, `src/lib.rs`;
- `src/syntax/**`, `src/sema/**`, `src/ir/**`, `src/bytecode/**`, `src/render/**`.

### Tests и oracles

- `tests/**`, `fixtures/**`, `test-support/reference/**`.

Production-код агента ограничен representations, механическими helpers, явными entry-point заглушками и существующей origin plumbing. `Cargo.toml`, `Cargo.lock`, toolchain, `src/main.rs`, `src/vm/**`, `src/dsp/**` и другие paths запрещены.

## 4. Нецели

- Новые builtins/DSP: пример с `clamp` в §4.5 является синтаксическим примером, а не поручением добавлять `clamp`.
- Higher-order functions, anonymous functions, closures как runtime values, generics, перегрузки и recursion.
- Function-call bytecode instructions, optimizations, CSE, dead-node elimination или buffer reuse.
- `inspect`, persisted/stable bytecode ABI, modules/imports/exports, packages, registry, plugins.
- Stereo, filters, feedback, events, probes и другие следующие этапы.
- Изменение CLI flags, WAV/PCM16, старых diagnostic codes или numeric tolerances.
- Объявление этапа завершённым при временных заглушках или ожидаемых красных tests.

## 5. Принятые контракты реализации

### 5.1. Syntax и spans

1. `Program.declarations` сохраняет исходный порядок `TopDecl::{Let,Function}`. Структурный `out` ищется только среди top-level `Let`.
2. Function declaration хранит name, typed parameters, return annotation, body expression и span от `let` до `;`.
3. Block expression содержит locals и обязательный result; block span включает braces. Пустой block и trailing `;` после result отклоняются.
4. Pipeline desugars в named `Call`: LHS становится первым аргументом. Полный call span включает LHS и RHS; callee span — только RHS identifier.
5. Function/body blocks и каждый desugared expression участвуют в существующих parser depth/node limits. Function declaration считается одним declaration node; parameter/type identifiers отдельными nodes не являются.
6. Function declarations допустимы только на top level; callee остаётся именем, а pipeline RHS — именем с optional call suffix.

### 5.2. Resolution и typing

1. Top-level declarations обрабатываются в порядке исходника. Duplicate declaration проверяется до value/body; функция становится видимой только после успешной проверки тела.
2. Parameters получают IDs в порядке сигнатуры. Parameters и locals непосредственно тела составляют один scope; вложенный block разрешает shadowing.
3. Функция видит ранние top-level values/functions. Self/forward callee — `E2102`; unresolved value — `E2001`. Function name в value position не является first-class value.
4. Пользовательские values/functions не дублируют имена друг друга; function `sine` отклоняется `E2000`. Старый value binding `sine` не мешает builtin call.
5. Callee resolution предшествует arity, затем arguments проверяются слева направо. Return `Float → Sig` и argument lifts явно представлены в Typed AST.
6. Duplicate parameter/local — `E2000`; return/argument mismatch — `E2100`; wrong arity — `E2101`. Spans и secondary labels закреплены в SPEC §6.2.
7. Тело проверяется даже у невызванной функции. Не зависящие от параметров Float errors выдаются сразу; зависимые — при каждой конкретизации.

### 5.3. Graph construction

1. User function не создаёт IR node сама по себе. Невызванная функция не создаёт nodes/roots.
2. Args lower’ятся слева направо один раз и получают eager roots; parameter reference возвращает arg `NodeId`.
3. Каждый local lower’ится один раз и получает eager root; result lower’ится после locals.
4. Body expression nodes создаются заново на каждый вызов; `Sine` получает новый node/state. Captured top-level values переиспользуются.
5. `Graph.roots` включает top-level values, user-call args и locals. Внутренние roots предшествуют enclosing top-level root; duplicates разрешены.
6. Canonical topological IDs, explicit lifts, left-to-right nodes и отсутствие optimizations сохраняются.
7. `FunctionLimits` применяются до неконтролируемого expansion. Нарушение — `E2106`, не panic и не internal invariant.

Подробности: ADR 0007.

### 5.4. Origins и runtime

1. `NodeOrigin` содержит primary и `inlined_from: Vec<CallOrigin>`; chain идёт от inner к outer call.
2. Каждый `CallOrigin` содержит `FunctionId` и полный call span. Args не получают frame вызываемой функции; shared nodes не переписываются.
3. IR/bytecode origin verifiers проверяют все spans. Debug stripping не меняет nodes/registers/state/samples.
4. Source-aware compile/runtime diagnostics добавляют secondary call chain и сохраняют primary в определении, code и sample index.
5. VM instructions и numeric algorithms не меняются; steady-state остаётся allocation-free.

## 6. Порядок production-реализации человеком

1. **Lexer/parser:** punctuation, declarations, blocks, pipeline desugaring, limits и spans. Проверки: `cargo test --test lexer --locked`, `cargo test --test parser --locked`, `cargo test --test functions --locked`.
2. **Resolved representations:** scopes, SymbolStorage, FunctionId/BindingId, Typed declarations/calls/blocks. Проверка: `cargo test --test sema --locked`.
3. **Type checking и Float:** calls/arity/coercion, shadowing, per-call constants/lifts, diagnostics и expansion budget. Проверки: `cargo test --test functions --locked`, `cargo test --test diagnostics --locked`.
4. **Lowering:** environments, eager roots, sharing/fresh state и call chains. Проверки: `cargo test --test ir_verify --locked`, `cargo test --test bytecode_verify --locked`.
5. **Renderer/reference:** secondary origins, independent function reference, analytic runtime cases. Проверки: `cargo test --test reference --locked`, `cargo test --test audio --locked`, `cargo test --test differential --locked`.
6. **Properties/compat:** generated call/block/pipeline corpus, limits, reset/block partitions/debug strip/determinism. Проверки: `cargo test --test properties --locked`, `cargo test --test compatibility --locked`, `cargo test --test allocations --locked`.
7. **Приёмка:** удалить все временные заглушки, запустить полный набор, выполнить real CLI check/run, atomic/error сценарии и удалить generated artifacts.

При изменении нестабильных Rust signatures человек адаптирует tests без ослабления expected и обновляет task document.

## 7. Матрица проверки

| Требование | Positive/negative/boundary cases | Независимый oracle |
|---|---|---|
| §4.3/§4.5 — tokens/AST | `=>`, braces, `|>`, функции, blocks, malformed RHS/params/result | Явные TokenKind/AST variants и byte offsets |
| Приоритет pipeline | `a + b |> f`, chains, calls, invalid arbitrary callee | Явный desugared AST |
| Scopes/IDs | capture, shadowing, duplicates, self/forward/indirect recursion | Явные FunctionId/BindingId и exact diagnostics |
| Typing/lifts | params/args/return/locals, wrong arity/types, unused function | Явные типы, lift nodes, code/SourceId/spans |
| Compile-time Float | generic body, per-call finite/overflow/div-zero/lift | Независимые f64/f32 формулы и exact spans/call chain |
| ADR 0007 — sharing | repeated let/parameter/capture, two calls | Явные NodeId/roots/state counts |
| Eager roots | unused parameter/local errors и ordering | Exact первый code/span/sample index |
| Origins | nested direct/pipeline calls, args, corruption, stripping | Явные primary/call spans и inner-to-outer chain |
| Limits | zero, exact boundary, boundary+1, exponential composition | Аналитически посчитанные depth/visits и E2106 |
| §7.4/§7.5 — runtime | functions/blocks/pipeline при всех block sizes/reset | Independent reference и analytic samples |
| Compatibility | все прежние fixtures/CLI/WAV/determinism/allocations | Старые expected без ослабления |

Audio comparison использует `2e-5 + 2e-5 × abs(expected)` для sine, точное сравнение для чистой арифметики. IDs, roots, counts, spans, error enums и PCM bytes сравниваются точно.

## 8. Definition of Done этапа

- [x] Пример `mix`, пользовательские композиции `sine`, expression blocks и locals работают.
- [x] Pipeline полностью эквивалентен явному вызову по Typed AST semantics, IR и samples.
- [x] Два вызова не делят state; каждый arg/let/captured binding сохраняет выбранный sharing.
- [x] Eager unused args/locals сохраняют runtime errors и их порядок.
- [x] Resolved IDs и origin chains ссылаются на правильные определения/spans; shadowing детерминирован.
- [x] Recursion, forward calls и higher-order использования получают стабильные diagnostics.
- [x] Per-call Float/lift diagnostics содержат definition primary и inner-to-outer call chain.
- [x] Expansion limits и все boundary cases проходят без panic/неограниченного роста.
- [x] IR/bytecode verifiers проверяют call origins; debug stripping не меняет звук.
- [x] Прежние fixtures, block partitions, reset, differential/properties и zero-allocation tests проходят.
- [x] Нет заглушек, ignored tests, новых dependencies или future-stage infrastructure.
- [x] Узкие и общие проверки действительно запущены и прошли; real CLI сценарии проверены.
- [x] README/changelog/development docs соответствуют фактически работающему состоянию.

## 9. Обязательные команды приёмки

```bash
cargo test --test lexer --locked
cargo test --test parser --locked
cargo test --test sema --locked
cargo test --test functions --locked
cargo test --test ir_verify --locked
cargo test --test bytecode_verify --locked
cargo test --test reference --locked
cargo test --test audio --locked
cargo test --test differential --locked
cargo test --test properties --locked
cargo test --test compatibility --locked
cargo test --test allocations --locked
cargo test --test cli --locked
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
cargo test --doc --locked
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked
```

Дополнительно: real `check`/`run` функции с блоком и pipeline, nested runtime diagnostic, reset/block sizes `1/3/64/128/256`, debug stripping, independent processes и atomic output.

## 10. Фактические результаты

### Подготовительная задача агента — 2026-11-21

- Спецификация обновлена до 0.5.0; принят ADR 0007. Зафиксированы lexical IDs/scopes, захват ранних top-level values, one-time eager args/locals, sharing/fresh state, eager roots, per-call Float checks, call origins и expansion limits с `E2106`.
- Добавлены mechanical representations и ровно восемь разрешённых временных entry-point заглушек: три parser, три sema и две lowering. Старый Beep path не содержит глобальных заглушек.
- IR/bytecode verifiers проверяют inline call spans; renderer переносит retained chain в secondary diagnostics. Independent reference и fixed-seed generator расширены functions/blocks/pipeline.
- Добавлены 28 integration tests и девять fixtures. Общий corpus — 208 integration tests: все 180 прежних tests проходят; `functions` содержит 2 passed/24 ожидаемых failed, новые differential/property tests тоже ожидаемо падают на parser stub.
- Все targets компилируются через `--no-run`. `cargo fmt --check`, `git diff --check`, clippy `-D warnings`, doctest и rustdoc `-D warnings` успешны.
- Полный `--no-fail-fast` дал 182 passed, 26 ожидаемых failed, 0 ignored. Обычный общий test command останавливается на differential с 6 passed/1 failed. Это не зелёная приёмка production-этапа.
- Production functions/blocks/pipeline, конкретные sema/lowering algorithms, stage-4 reference/differential execution и real CLI scenarios не реализованы и не проверены. Следующий шаг — ручная реализация по разделу 6, удаление восьми заглушек и полный DoD.


### Production-реализация и финальная приёмка — 2026-11-21

- Parser поддерживает function declarations, blocks, pipeline desugaring, required annotations, source spans и parser limits. Semantic resolution использует lexical scopes и compilation-local `BindingId`/`FunctionId`; Typed AST сохраняет declarations/calls/blocks.
- Function checks включают duplicate/shadowing rules, return/argument coercion, explicit lifts, recursion/forward rejection и definition-primary call-chain diagnostics. Generic `DependsOnParameters` и concrete per-call `Float` evaluation различны; eager locals/args и expansion limits проверяются.
- Lowering использует `BindingId → NodeId` environment, eager roots для args/locals, sharing references/captured values, fresh body/state nodes на каждый вызов и inner-to-outer `CallOrigin` chains. Function/block/pipeline не добавляют bytecode/VM instructions.
- Исправлены два test-only ожидания/_helper-а: identity call с `Sig` parameter учитывает три Typed expression visits (`Number`, inserted lift, body identifier), а VM helper разбивает output по configured block size. Production-семантика ради этих tests не ослаблялась.
- Allocation-counting test дополнительно покрывает stage-4 function/block/pipeline graph и подтверждает 0 heap allocations в steady-state `render_into`/`reset`.
- Все узкие suites прошли. `cargo test --all-targets --locked` дал **208 integration tests, 0 failed, 0 ignored**. `cargo fmt --check`, clippy `-D warnings`, doctest (1 passed), rustdoc `-D warnings` и `git diff --check` успешны.
- Реальный `run fixtures/valid/stage-04-blocks-pipeline.ln` создал mono PCM16 WAV на 48 samples/140 байт с корректными RIFF fields. Runtime fixture с unused local вернул `E4001` на sample 0 с definition primary и call origin, итоговый WAV не опубликован.
- Generated WAV/temp artifacts удалены. Все пункты DoD выполнены; этап 4 завершён, этап 5 не начат.

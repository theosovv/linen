# Этап 4 — SDD, test-facing каркас и независимые contract tests

- Статус: историческая подготовительная задача агента завершена 2026-11-21. Последующая production-реализация и финальная приёмка описаны в основном stage document; этап 4 завершён.
- Исполнитель подготовительной задачи: агент.
- Основной parser/type checker/function lowering: человек.
- Основной stage document: `docs/development/stages/stage-04-functions-blocks-pipeline.md`.
- Связанный ADR: `0007`.

## Контекст

- Этап: 4, функции, блоки и pipeline.
- Baseline: этап 3 завершён, дерево чистое, все 180 integration tests проходят.
- Пользователь явно разрешил механический production-каркас с ограниченными временными `todo!()`. Это исключение только для подготовительного diff, не для завершения этапа.
- Пользователь выбрал захват ранних top-level values/functions, one-time args с sharing, eager unused args/locals, запрет duplicate/builtin function names, definition-primary + call chain diagnostics и отдельные expansion limits с `E2106`.

## Задача

1. Обновить SPEC до 0.5.0 и создать ADR 0007, stage/task documents.
2. Добавить syntax/Typed representations, resolved IDs/storage, FunctionLimits, call origins и механическую plumbing.
3. Оставить только явно перечисленные entry-point заглушки для основной семантики человека.
4. Подготовить positive/negative/boundary tests, fixtures, independent reference и generators.
5. Запустить проверки, зафиксировать ожидаемые failures и не объявлять production-этап завершённым.

## Разрешено менять

- `SPEC.md`, `CHANGELOG.md`, `README.md`, `AGENTS.md`;
- `docs/**`;
- `src/api.rs`, `src/diagnostic.rs`, `src/lib.rs`;
- `src/syntax/**`, `src/sema/**`, `src/ir/**`, `src/bytecode/**`, `src/render/**`;
- `tests/**`, `fixtures/**`, `test-support/reference/**`.

Production разрешён только как representations, механические helpers, origin plumbing и явные заглушки. `Cargo.toml`, `Cargo.lock`, toolchain, `src/main.rs`, `src/vm/**`, `src/dsp/**` и другие paths запрещены.

## Нецели

- Основная реализация функций агентом, placeholder success или скрытый fallback.
- `clamp` и другие новые builtins/DSP, VM function instructions и optimizations.
- Higher-order/anonymous functions, generics, recursion, modules/packages и следующие этапы.
- Изменение CLI/WAV/PCM16, dependencies/toolchain, удаление/ослабление/ignore tests.
- Зелёная приёмка этапа 4 в подготовительном diff.

## Test-facing API

Это нестабильный механический Rust seam. Человек может уточнять signatures, адаптируя tests и документ без ослабления independent expected.

### Syntax AST

```text
Program { declarations: Vec<TopDecl>, render, span }
TopDecl::{Let(LetDecl), Function(FunctionDecl)}
FunctionDecl { name, parameters, result: TypeAnnotation, body: Expr, span }
Parameter { name, annotation, span }
ExprKind::{
    Number, Identifier, Call, Unary, Binary, Group,
    Block { bindings: Vec<LetDecl>, result: Box<Expr> },
}
```

Pipeline представляется обычным `ExprKind::Call`; отдельного Pipeline AST node нет. `TokenKind` добавляет `LeftBrace`, `RightBrace`, `PipeGreater`, `FatArrow`.

### Typed AST и symbols

```text
FunctionId::{new,index}
FunctionLimits { max_call_depth, max_expanded_nodes } // 128, 10_000
TypedProgram { declarations: Vec<TypedTopDecl>, output, render, span }
TypedTopDecl::{Binding(TypedBinding), Function(TypedFunction)}
TypedFunction { id, name, parameters, result, body, span }
TypedParameter { id, name, annotation, span }
TypedExprKind::{
    Number, Identifier,
    BuiltinCall { builtin, callee, arguments },
    FunctionCall { function, callee, arguments },
    Unary, Binary, Group, LiftFloatToSig,
    Block { bindings, result },
}
FunctionSymbol { id, name, parameter_types, result, span }
SymbolStorage::{get,get_function,len,function_len,is_empty}
CompileConfig { parser, functions: FunctionLimits }
```

`BindingId` уникален в пределах compilation для top-level/parameter/local. `FunctionId` и `BuiltinId` — отдельные index spaces. Storage очищается перед каждой semantic compilation и выдаёт IDs детерминированно.

### Origins и lowering

```text
CallOrigin { function: FunctionId, call_span: SourceSpan }
NodeOrigin { primary, inlined_from: Vec<CallOrigin> }
NodeOrigin::new(primary) // пустой inlined_from
IrVerifyError::InvalidInlineOrigin { node, inline_index, span }

lower(&TypedProgram) -> Graph
```

`Graph` сохраняет прежние поля; roots расширены eager args/locals по ADR 0007. Codegen/bytecode структурно прежние, но origin verification проходит primary и каждый call span. `NodeOrigin` больше не `Copy` из-за Vec; механические clones не меняют semantics.

## Явно разрешённые временные заглушки

Только следующие private entry points могут содержать `todo!("stage 4: ...")`:

- `Parser::parse_function_decl` — сигнатура и тело top-level функции;
- `Parser::parse_block_expr` — locals и обязательный result;
- `Parser::parse_pipeline_suffix` — RHS/desugaring и spans;
- `sema::check::check_function` — function scope/signature/body;
- `sema::check::check_block` — nested scope/local inference;
- `sema::check::check_function_call` — resolution/concretization/budget;
- `ir::lower::lower_block` — local environment, eager roots и result;
- `ir::lower::lower_function_call` — env/eager roots/fresh body/origins.

Каждая заглушка достижима только соответствующим новым syntax/Typed variant; старый Beep path не заменяется глобальным `todo!()`. `unimplemented!()`, фиктивный AST/IR/bytecode, тишина и placeholder success запрещены. Все заглушки удаляются до завершения этапа.

## Контракт и независимые проверки

| Требование | Проверка | Независимый oracle |
|---|---|---|
| §4.5 — syntax | `tests/functions.rs`, lexer/parser cases и fixtures | Явные AST/tokens/spans |
| Scopes/IDs/types | capture/shadowing/duplicates/recursion/arity/lifts | Явные IDs/types и exact diagnostics |
| ADR 0007 — graph | args/locals roots, sharing, fresh states | Явные NodeId/state/node sequences |
| Origins | nested direct/pipeline calls и verifier corruption | Явные source offsets и FunctionId |
| E2106 | zero/exact/over limits и exponential source | Аналитический depth/visit count |
| Runtime | mix/blocks/pipeline/fresh state/eager errors | Аналитические samples и independent reference |
| Compatibility | все прежние suites и fixtures | Старые expected без изменений |

Expected не вычисляется production parser/checker/lowering/VM. Diagnostics проверяют code, SourceId, primary/secondary byte spans и sample index там, где применимо. Audio использует independent reference или аналитическую формулу и согласованный tolerance.

## Критерии приёмки подготовительной задачи

- [x] Diff ограничен allowlist; основная семантика не реализована агентом.
- [x] SPEC/ADR/stage/task отражают все согласованные решения до production-реализации.
- [x] Каркас компилируется; заглушки находятся только в восьми разрешённых entry points.
- [x] Tests покрывают positive, negative, boundary и independent oracles без ignored.
- [x] Старые assertions не ослаблены; mechanical API migration не меняет старые expected.
- [x] Все проверки запущены; ожидаемые failures и непроверенное явно перечислены.
- [x] Этап 4 не отмечен завершённым и README не обещает работающие функции.

## Обязательные команды подготовительной задачи

```bash
cargo test --test functions --locked --no-run
cargo test --test differential --locked --no-run
cargo test --test properties --locked --no-run
cargo test --all-targets --locked --no-run
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
cargo test --doc --locked
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked
```

До production-реализации новые tests ожидаемо падают. `--no-run` доказывает компиляцию, но не зелёную приёмку. Дополнительный `--no-fail-fast` используется только для полного учёта failures.

## Отчёт об исполнении

- Обновлены `SPEC.md` 0.5.0, ADR 0007, stage/task documents и статусы README/AGENTS/development docs. Согласованные capture/sharing/eager/origin/limit rules зафиксированы до ручной реализации.
- Добавлены mechanical syntax/Typed representations, `FunctionId`, расширенный `SymbolStorage`, `FunctionLimits`, `CallOrigin`/`inlined_from`, origin verification и renderer secondary labels. Ровно восемь явных `todo!()` находятся в разрешённых entry points; `unimplemented!()`, placeholder success и ignored tests отсутствуют.
- Independent reference расширен function/block graph construction со scoped values, eager args/locals, sharing и fresh sine phases. Fixed-seed generator создаёт функции с capture, nested blocks и pipeline. Stage-4 поведение reference ещё не проверено end-to-end, поскольку production parser не выдаёт Typed AST.
- Добавлены 28 integration tests: `functions` 26, `differential` 1, `properties` 1; добавлены девять fixtures. Общий corpus — 208 integration tests. Все 180 прежних tests проходят.
- `cargo test --all-targets --locked --no-run` успешен без warnings. `cargo fmt --check`, `git diff --check`, clippy `-D warnings`, doctest (1 passed) и rustdoc `-D warnings` успешны.
- `cargo test --test functions --locked`: 2 passed, 24 ожидаемых failed. Полный `--no-fail-fast`: 182 passed, 26 ожидаемых failed, 0 ignored. Это 24 function tests и по одному generated differential/property test; все достигают явных parser stubs. Обычный общий test command останавливается на differential: 6 passed, 1 failed.
- Production functions/blocks/pipeline, semantic/lowering correctness, generated corpus execution, real stage-4 CLI render и полный DoD не проверены. Следующий шаг человека — реализация восьми entry points, удаление заглушек и зелёная приёмка всех 208 tests.

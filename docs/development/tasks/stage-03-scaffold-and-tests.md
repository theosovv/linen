# Этап 3 — SDD, test-facing каркас и независимые contract tests

- Статус: историческая подготовительная задача агента завершена 2026-11-20. Последующая production-реализация, benchmark и приёмка описаны в основном stage document; этап 3 завершён 2026-11-21.
- Исполнитель подготовительной задачи: агент.
- Основной IR lowering/codegen/verifiers/VM: человек.
- Основной stage document: `docs/development/stages/stage-03-ir-bytecode-vm.md`.
- Связанные ADR: `0004`, `0005`; базовые ограничения — `0001`, `0002`, `0003`, `0006`, `0009`, `0011`, `0012`.

## Контекст

- Этап: 3, IR, bytecode и блочная VM.
- Пункты `SPEC.md` 0.4.0: 5.1–5.11, 6.1–6.5, 7.1–7.11, 9 (этап 3), 10–14.
- Экосистемные инварианты: `SourceSpan`/`SourceMap`, resolved IDs, origins, deterministic codegen, capabilities/resources, отсутствие скрытого I/O.
- Baseline: этап 2 завершён, дерево чистое, 121 integration test зелёный.
- Пользователь прямо разрешил временные заглушки в этом подготовительном diff. Это исключение не распространяется на завершение этапа.
- Пользователь выбрал сохранение вычисления всех top-level bindings, включая unused, через явные evaluation roots.

## Задача

1. Уточнить `SPEC.md` canonical IR/roots/sharing, typed registers/state/output, debug origins, internal errors и acceptance contracts.
2. Создать ADR 0004/0005, stage document и этот проверяемый task.
3. Добавить механические IR/bytecode/VM representations, descriptors, config/facade seams и явные entry-point заглушки.
4. Подготовить independent verifier/VM/differential/property/compatibility tests, fixtures и fixed-seed generators.
5. Запустить проверки, честно записать expected failures и не объявлять этап завершённым.

## Разрешено менять

- `SPEC.md`, `CHANGELOG.md`, `README.md`, `AGENTS.md`;
- `docs/adr/**`, `docs/development/**`;
- `src/api.rs`, `src/diagnostic.rs`, `src/lib.rs`, `src/main.rs`;
- `src/sema/builtins.rs`, `src/sema/mod.rs`, `src/dsp/**`, `src/render/**`;
- `src/ir/**`, `src/bytecode/**`, `src/vm/**`;
- `tests/**`, `fixtures/**`, `test-support/reference/**`.

Production-код разрешён только в перечисленных путях и только как representations, механические helpers и facade boilerplate, не основная реализация ядра. `Cargo.toml`, `Cargo.lock`, toolchain, dependencies и другие paths не входят в allowlist.

## Нецели

- Реализация lowering/codegen/verifiers/VM semantics агентом.
- Optimizations, persistence/stable ABI, `inspect`, functions, новые DSP и module/package infrastructure.
- Скрытые fallback, placeholder success, `#[ignore]` и удаление failing tests.
- Зелёная приёмка этапа 3 в этом diff.

## Test-facing API

Это нестабильный механический Rust seam, а не новый языковой контракт. Он конкретизирует ADR и позволяет писать integration tests до реализации. При изменении сигнатур человек адаптирует вызовы tests и этот документ, сохраняя independent expected.

### IR

```text
Graph { nodes, roots, output, render_samples }
NodeId::{new,index}
Node { id, op, result_type, origin }
NodeOrigin { primary: SourceSpan }
ValueType::{Float,Sig}
NodeOp::{
    ConstFloat,
    LiftFloatToSig,
    Neg,
    Add, Sub, Mul, Div,
    Sine,
}
NodeOp::descriptor(result_type: ValueType) -> NodeDescriptor
NodeDescriptor { capability, resources }

lower(&TypedProgram) -> Graph
verify(&Graph, &SourceMap) -> Result<(), IrVerifyError>
```

Roots следуют declaration order; output входит в roots. Node IDs совпадают с indices и inputs предшествуют consumer. Identifier references разделяют NodeId; Group не создаёт node. Для Sine origin.primary указывает на frequency argument.

### Capability/resource metadata

```text
RuntimeCapability::CoreAudio
ResourceModel {
    scalar_registers: usize,
    audio_registers: usize,
    state_slots: usize,
    state_bytes: usize,
}
BuiltinDescriptor {
    // существующие fields +
    capability: RuntimeCapability,
    resources: ResourceModel,
}
```

Все текущие ops имеют `CoreAudio`. Const/scalar ops требуют один scalar register; signal ops — один audio register; Sine дополнительно требует один state slot и `size_of::<f64>()` state bytes.

### Bytecode

```text
Bytecode {
    instructions,
    scalar_register_count,
    audio_register_count,
    state_count,
    output: AudioRegister,
    debug_origins: Option<Vec<NodeOrigin>>,
}
ScalarRegister / AudioRegister / StateId::{new,index}
Operand::{Scalar,Audio}
BinaryOp::{Add,Subtract,Multiply,Divide}
Instruction::{
    ConstScalar,
    Lift,
    NegScalar,
    NegAudio,
    BinaryScalar,
    BinaryAudio,
    Sine,
    SetOutput,
}

codegen(&Graph, NodeOrigin) -> Bytecode
verify(&Bytecode) -> Result<(), BytecodeVerifyError>
verify_origins(&Bytecode, &SourceMap) -> Result<(), BytecodeVerifyError>
Bytecode::strip_debug_info()
```

Codegen использует один register на node, unique sequential StateId и dedicated output register с ровно одной финальной SetOutput. Второй аргумент передаёт origin полного out value для SetOutput независимо от frequency origin у Sine. `BinaryAudio`/Sine допускают typed scalar или audio operands. Debug origins либо отсутствуют, либо имеют по одному valid origin на instruction.

### VM и facade

```text
VmConfig { sample_rate: u32, block_size: usize }
VmConfig::default()                  // 48000, 128
VmConfig::validate() -> Result<(), VmConfigError>

Vm::new(&Bytecode, VmConfig) -> Result<Vm, VmCreateError>
Vm::reset()
Vm::render_into(&mut [f32]) -> Result<(), VmRuntimeError>
VmRuntimeError::{
    BlockTooLarge,
    NonFiniteFrequency { instruction, state, sample_index },
    NonFiniteOutput { sample_index },
    SampleIndexOverflow,
}

CompiledProgram { typed, ir, bytecode, config }
create_vm(&CompiledProgram) -> Result<Vm, VmCreateError>
render(&CompiledProgram, impl Write) -> Result<RenderReport, RenderError>
```

`CompileOutcome.result` использует `CompileError::{Diagnostic,IrVerify,BytecodeVerify}`: source errors остаются exit `1`, verifier invariant — exit `3`. `CompileSession::check` не меняется. Production compile сохраняет origins; VM runtime error остаётся structured и не содержит source Diagnostic.

Empty block — no-op; блок больше configured maximum отклоняется до dispatch. Все registers/buffers/states выделяются в `new`, reset очищает registers/states/index. Внутри блока instructions выполняются в Vec order для каждого sample, а SetOutput отклоняет nonfinite output до следующего sample, сохраняя прежний fail-fast порядок. Render выделяет один scratch block вне loop после перевода на VM.

## Явно разрешённые временные заглушки

Только следующие entry points могут содержать `todo!("stage 3: ...")` до ручной реализации:

- `ir::lower::lower` — Typed AST → canonical Graph;
- `ir::verify::verify` — graph invariants/origins;
- `bytecode::codegen::codegen` — Graph и полный out origin → Bytecode;
- `bytecode::verify::{verify,verify_origins}` — structural/origin checks;
- `vm::execute::Vm::{new,render_into}` — allocation/setup и block dispatch.

Механические IDs, representations, descriptors, config validation, debug stripping, reset и error types должны быть реальными. Никакая заглушка не возвращает фиктивный успех, пустой graph/bytecode, тишину или WAV. `unimplemented!()` не используется. Все заглушки удаляются до завершения этапа.

## Контракт и независимые проверки

| Требование | Проверка | Независимый oracle |
|---|---|---|
| §5.6, ADR 0004 — IR/lowering | `tests/ir_verify.rs`: explicit graphs, roots, sharing, spans | Явные nodes/IDs/offsets |
| IR verifier | Manual corruption каждого invariant | Exact structured error variants |
| §5.5/§5.6 — metadata | Каждый op и sine descriptor | Явная capability/resource table |
| §5.7, ADR 0005 — bytecode | `tests/bytecode_verify.rs`: codegen layout и corruption | Явные instructions/register/state counts |
| VM contract | `tests/vm.rs`: scalar/audio ops, output, reset, errors | Аналитические values и exact errors |
| §7.4 — block independence | Fixed sizes, empty calls, variable partitions | Reference/whole-block samples |
| §7.5, ADR 0009 — differential | `tests/differential.rs` и existing reference | Независимый посемпловый interpreter |
| §7.6 — properties | `tests/properties.rs` с fixed-seed generator | Mathematical/reference invariants |
| Compatibility | `tests/compatibility.rs`, все прежние tests/fixtures | Прежние expected без ослабления |
| Origins/debug | Valid SourceIds/spans, stripped parity | SourceMap checks и sample comparison |

Expected не вычисляется production lowering/codegen/VM/DSP. Для diagnostics проверяются code, SourceId, primary span и sample_index; для verifier — exact structured variant. Audio использует посемпловый `2e-5 + 2e-5 × abs(expected)`. `#[ignore]` и новые dependencies не добавляются.

Allocation-counting harness требует `unsafe impl GlobalAlloc` либо новой dependency, а оба варианта находятся вне текущих разрешений. Он не подделывается в подготовительном diff и остаётся отдельным блокером финального DoD до явного test-only решения. Benchmark также относится к финальной production-приёмке.

## Критерии приёмки подготовительной задачи

- [x] Diff ограничен согласованным и расширенным allowlist; основная семантика не реализована агентом.
- [x] SPEC/ADR/stage/task устраняют согласованные неоднозначности до tests.
- [x] Test-facing каркас компилируется; заглушки находятся только в разрешённых entry points.
- [x] IR/bytecode/VM/differential/property/compatibility tests покрывают positive, negative и boundary cases с independent expected.
- [x] Для diagnostics проверяются code, SourceId, primary span и sample_index там, где требуется.
- [x] Нет ignored tests, новых dependencies, production-derived expected и скрытых fallback.
- [x] Прежние source/sema/CLI/WAV assertions не ослаблены; изменения типов compile error не меняют source semantics.
- [x] Все проверки запущены; ожидаемые failures и непроверенные сценарии явно перечислены.
- [x] Этап 3 не отмечен завершённым, README не обещает работающую VM.

Окончательный DoD находится в stage document и этим подготовительным чек-листом не заменяется.

## Обязательные команды подготовительной задачи

```bash
cargo test --test ir_verify --locked --no-run
cargo test --test bytecode_verify --locked --no-run
cargo test --test vm --locked --no-run
cargo test --test differential --locked --no-run
cargo test --test properties --locked --no-run
cargo test --test compatibility --locked --no-run
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
cargo test --doc --locked
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked
```

До production-реализации tests, достигающие явные `todo!()`, ожидаемо падают; `--no-run` доказывает компиляцию всех новых targets. Полный test набор не считается пройденным при ожидаемых failures. Результат фиксируется без `--no-fail-fast`, а дополнительный `--no-fail-fast` используется только для полного учёта ожидаемых красных tests.

## Отчёт об исполнении

- Подготовлены `SPEC.md` 0.4.0, ADR 0004/0005, stage/task documents и актуализированы README/AGENTS/development status. По согласованию все top-level bindings сохраняются evaluation roots, включая unused.
- Добавлены реальные representations и механические helpers: IR nodes/IDs/origins/descriptors, typed bytecode registers/instructions/state IDs, debug stripping, VM config/errors/reset, compile/facade composition. Ровно семь явных `todo!()` находятся только в разрешённых entry points. `unimplemented!()`, placeholder success и ignored tests отсутствуют.
- Добавлены 52 tests: `ir_verify` 13, `bytecode_verify` 9, `vm` 14, `differential` 6, `properties` 5, `compatibility` 5; три fixtures и fixed-seed generator. Источники expected — явные graphs/instructions/spans, analytic formulas, independent reference и прежние fixtures.
- Все шесть узких `--no-run` команд и общий `--no-run` успешны. `cargo fmt --check`, `git diff --check` и `cargo clippy --all-targets --locked -- -D warnings` успешны без warnings.
- `cargo test --doc --locked` — 1 passed; `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked` — успешно.
- Обычный `cargo test --all-targets --locked` ожидаемо останавливается на `audio` с 3 passed/6 failed. Полный `--no-fail-fast`: 99 passed, 74 failed, 0 ignored из 173 integration tests. 71 failure непосредственно достигает явных заглушек; три CLI failures — parent assertions после того же child lowering panic.
- Production Beep через VM, verifier correctness на реализации, block/differential green results, allocations и benchmark не проверены. Allocation-counting harness требует отдельного test-only unsafe/dependency решения и не имитировался.
- Подготовительная задача завершена, production-этап — нет. Следующий шаг человека — реализация по stage §6, удаление семи заглушек и полная приёмка.

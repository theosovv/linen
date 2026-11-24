# Этап 5D — SDD, PolyBLEP scaffold и независимые contract tests

- Статус: подготовительная задача завершена 2026-11-22; production 5D остаётся ручной следующей задачей.
- Исполнитель подготовительной задачи: агент.
- Основной lowering/DSP: человек.
- Основной stage document: `docs/development/stages/stage-05d-polyblep-anti-aliasing.md`.
- Связанный ADR: `0016`.

## Контекст

- Этап: 5D, отдельный anti-aliasing подэтап после завершённого этапа 5.
- Baseline: дерево чистое; спецификация 0.6.0 сообщает о 239 integration tests и одном allocation harness, все проходят.
- Пользователь явно выбрал отдельные algorithm-specific builtins, охват saw/square, explicit naive fallback выше Nyquist и схему SDD + mechanical scaffold + tests.
- Основная цель — не реализовать DSP за человека, а устранить неоднозначности и подготовить проверяемые точки ручной реализации.

## Задача

1. Обновить SPEC до 0.7.0, создать ADR 0016, stage/task documents и отдельные compatibility/changelog записи.
2. Добавить два builtin/IR/bytecode/VM representation и mechanical descriptor/codegen/verifier/state plumbing.
3. Оставить ровно три перечисленные entry-point заглушки для основной семантики человека.
4. Добавить independent PolyBLEP reference, fixed-seed source generation, analytic/spectral tests, positive/negative/boundary cases и fixtures.
5. Запустить compile/static/full checks, учесть ожидаемые failures и не объявлять production 5D завершённым.

## Разрешено менять

- `SPEC.md`, `CHANGELOG.md`, `AGENTS.md`, `docs/**`;
- `src/sema/types.rs`, `src/sema/builtins.rs`;
- `src/ir/graph.rs`, `src/ir/lower.rs`, `src/ir/verify.rs`;
- `src/bytecode/instruction.rs`, `src/bytecode/codegen.rs`, `src/bytecode/verify.rs`;
- `src/dsp/mod.rs`, `src/dsp/oscillator.rs`;
- `src/vm/state.rs`, `src/vm/execute.rs`;
- `tests/**`, `fixtures/**`, `test-support/reference/**`.

Production разрешён только как representations, descriptors, mechanical plumbing и явные заглушки. `README.md`, `Cargo.toml`, `Cargo.lock`, toolchain и другие paths запрещены.

## Нецели

- Основной PolyBLEP/lowering algorithm агентом, placeholder success или скрытый fallback сверх SPEC.
- Изменение naive oscillators, общий AA selector, triangle/phasor AA и другие алгоритмы.
- Filters/feedback, modules/imports/source stdlib, plugins, stable bytecode ABI и следующие этапы.
- Новые dependencies/toolchain, CLI/WAV/diagnostic changes, ignore/delete/ослабление tests или рост tolerances.
- Объявление новых builtins работающими или этапа завершённым в подготовительном diff.

## Test-facing representations

```text
BuiltinId::{SAW_POLYBLEP,SQUARE_POLYBLEP}
NodeOp::{SawPolyblep(NodeId),SquarePolyblep(NodeId)}
Instruction::{SawPolyblep { dst,freq,state },SquarePolyblep { dst,freq,state }}
dsp::{SawPolyblep,SquarePolyblep}
```

Новые structs имеют `new`, `phase`, `reset` по образцу существующих phase oscillators. Это нестабильные Rust seams; человек может уточнять внутренние helpers без изменения independent expected.

## Три явные временные заглушки

1. `ir::lower::lower_stage5d_builtin`.
2. `dsp::SawPolyblep::next_sample`.
3. `dsp::SquarePolyblep::next_sample`.

Каждая достижима только новым 5D operation. Старые paths не содержат новых глобальных заглушек. Все три удаляются до production-завершения этапа.

## Контракт и независимые проверки

| Требование | Проверка | Независимый oracle |
|---|---|---|
| §4.8.4/ADR 0016 — saw | initial/wrap/left/right/regular samples | Fixed vectors из явной `f64` formula |
| §4.8.4/ADR 0016 — square | wrap и half-cycle/shifted correction | Fixed vectors из явной `f64` formula |
| Frequency branches | zero, negative, `dt=0.5`, `dt>0.5`, NaN/Inf | Analytic branch/phase expectations |
| §5.5–§5.7 | descriptors, IR/bytecode/VM, origins/resources/state | Exact enums/counts/spans/corruption errors |
| §7.4–§7.6 | partitions/reset/reference/generated corpus | Independent test-support implementation |
| Spectral acceptance | fixed direct-DFT alias-bin set для saw/square | Analytic bins и минимум 20 dB reduction |
| Compatibility | все прежние tests/fixtures/WAV/CLI | Старые expected без изменений |

Expected не вычисляется production DSP. Reference реализует formulas отдельно и сам проверяется fixed vectors. Diagnostics проверяют code, SourceId, primary/call spans и frame index. Audio использует существующий abs+rel tolerance.

## Критерии приёмки

- [x] Изменения ограничены allowlist и выбранным 5D контрактом.
- [x] Ровно три `todo!`, никаких других placeholders или ignored tests.
- [x] Tests покрывают normal/edge/error/spectral/state/verifier cases с independent expected.
- [x] Все прежние tests и allocation scenario проходят без ослабления.
- [x] Все targets компилируются, static/doc checks запущены, ожидаемые failures перечислены.
- [x] Документация не обещает работающий production 5D до ручной реализации.

## Обязательные команды

```bash
cargo test --test polyblep --locked --no-run
cargo test --all-targets --locked --no-run
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked --no-fail-fast
cargo test --doc --locked
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked
git diff --check
```

Дополнительно отдельно запускаются прежние suites без новых generated cases, если нужно доказать baseline, и поиск `todo!`/`unimplemented!` для проверки списка.

## Отчёт об исполнении

- Изменённые файлы ограничены согласованным allowlist: SPEC/ADR/stage/task/status documents, перечисленные mechanical `src` files, tests/fixtures и test-support/reference. `README.md`, Cargo manifests, toolchain, CLI и WAV не изменены.
- Добавлены 15 tests, три fixtures, independent PolyBLEP formulas/reference и fixed-seed generator. Oracles: exact rational sample vectors, analytic frequency branches, explicit IR/bytecode/state/layout values, direct DFT bins и прежние compatibility expected.
- Все targets компилируются. `cargo fmt --check`, `git diff --check`, clippy `-D warnings`, doctest (1 passed) и rustdoc `-D warnings` успешны.
- Aggregate: 253 integration + 2 allocation tests, **244 passed, 11 expected failed, 0 ignored**. Все 239 прежних integration tests и прежний allocation scenario проходят. Все 11 failures достигают только трёх документированных заглушек.
- Найденные подготовительные ошибки относятся к tests/проектному контракту: обновлены два descriptor counts, decimal expected записаны exact rational formulas, shifted-phase rounding исправлен в SPEC/ADR до production. Production-дефектов прежнего кода не выявлено.
- Оставшиеся ограничения: основной lowering, оба DSP transitions, production spectral/allocation/real CLI и полный 5D DoD не выполнены. Подготовительная задача завершена; production 5D не завершён.

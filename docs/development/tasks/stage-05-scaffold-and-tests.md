# Этап 5 — SDD, test-facing каркас и независимые contract tests

- Статус: историческая подготовительная задача агента завершена 2026-11-22. Последующая production-реализация и финальная приёмка описаны в основном stage document; этап 5 завершён.
- Исполнитель подготовительной задачи: агент.
- Основной sema/lowering/DSP: человек.
- Основной stage document: `docs/development/stages/stage-05-basic-oscillators-stereo.md`.
- Связанные ADR: `0013`, `0014`, `0015`.

## Контекст

- Этап: 5, базовые осцилляторы и стерео.
- Baseline: этап 4 завершён, дерево чистое; все 208 integration tests и отдельный allocation harness проходят, 0 ignored.
- Пользователь явно выбрал SDD + mechanical production scaffold + tests и разрешил восемь ограниченных временных `todo!()` только для подготовительного diff.
- Приняты явный `Stereo`, interleaved frames, rising naive saw, triangle от `-1`, linear three-argument mix, equal-power pan и `noise(seed)` со SplitMix64.
- Anti-aliasing остаётся отдельным подэтапом и не реализуется/не моделируется скрыто.

## Задача

1. Обновить SPEC до 0.6.0 и создать ADR 0013/0014/0015, stage/task documents и status/changelog updates.
2. Добавить type/builtin/IR/bytecode/VM/WAV representations и mechanical channel/resource/origin plumbing.
3. Оставить ровно восемь перечисленных entry-point заглушек для основной семантики человека.
4. Подготовить positive/negative/boundary tests, fixtures, independent reference и fixed-seed generator.
5. Запустить compile/static/full checks, зафиксировать ожидаемые failures и не объявлять production-этап завершённым.

## Разрешено менять

- `SPEC.md`, `CHANGELOG.md`, `README.md`, `AGENTS.md`;
- `docs/**`;
- `src/api.rs`, `src/diagnostic.rs`, `src/lib.rs`, `src/main.rs`;
- `src/syntax/**`, `src/sema/**`, `src/ir/**`, `src/bytecode/**`;
- `src/vm/**`, `src/dsp/**`, `src/render/**`;
- `tests/**`, `fixtures/**`, `test-support/reference/**`.

Production разрешён только как representations, descriptors, mechanical plumbing и явные заглушки. `Cargo.toml`, `Cargo.lock`, toolchain и другие paths запрещены.

## Нецели

- Основные phase/pan/noise/sema/lowering algorithms агентом, placeholder success или скрытый fallback.
- PolyBLEP/anti-aliasing, filters/feedback, generic multichannel, events и следующие этапы.
- Новые dependencies/toolchain, stable bytecode ABI, modules/packages и CLI feature creep.
- Удаление/ослабление/ignore tests, рост tolerances или перезапись старых mono expected.
- Зелёная приёмка этапа 5 в подготовительном diff.

## Test-facing representations

Это нестабильные Rust seams. Человек может уточнять signatures с механической адаптацией tests и документов, не меняя independent expected.

```text
TypeName::{Float,Sig,Stereo}
Type::{Float,Sig,Stereo}
ValueType::{Float,Sig,Stereo}
BuiltinId::{SINE,PHASOR,SAW,SQUARE,TRIANGLE,GAIN,MIX,STEREO,PAN,NOISE}
RuntimeCapability::{CoreAudio,Multichannel}

NodeOp::{
    ConstFloat,LiftFloatToSig,Neg,Add,Sub,Mul,Div,Sine,
    Phasor,Saw,Square,Triangle,
    Stereo,Pan,Noise { seed, stream },
}
RandomStreamId::{new,seed}

Instruction::{
    ...existing,
    Phasor,Saw,Square,Triangle,MakeStereo,Pan,Noise,SetStereoOutput,
}
OutputLayout::{Mono(AudioRegister),Stereo { left, right }}
```

`Bytecode.output` становится явным `OutputLayout`; codegen register map различает scalar/audio/stereo pair. `Vm::render_into` сохраняет mono slice contract и принимает interleaved stereo values. `RenderReport` дополнительно exposes channel count; `samples` остаётся frames.

## Восемь явных временных заглушек

1. `sema::check::validate_noise_seed`.
2. `ir::lower::lower_stage5_builtin`.
3. `dsp::Phasor::next_sample`.
4. `dsp::Saw::next_sample`.
5. `dsp::Square::next_sample`.
6. `dsp::Triangle::next_sample`.
7. `dsp::pan`.
8. `dsp::Noise::next_sample`.

Каждая достижима только новым stage-5 operation; старый Beep/functions/mono path не заменяется глобальным `todo!()`. Все заглушки удаляются до завершения этапа.

## Контракт и независимые проверки

| Требование | Проверка | Независимый oracle |
|---|---|---|
| §4.8.1/ADR 0013 | phase/waveform/gain/mix vectors и edges | Явные formulas/samples |
| §4.8.2/ADR 0014 | Stereo types/layout/pan/interleaving/WAV | Explicit layouts, trig vectors, manual bytes |
| §4.8.3/ADR 0015 | seed diagnostics/SplitMix64/reset/stream identity | Fixed independent `u64` и audio vectors |
| §5.5–§5.7 | descriptors, origins, resources, verifier corruption | Exact enum/IDs/counts/spans |
| §7.4–§7.6 | partitions/reset/generated differential/properties | Independent reference, no production DSP expected |
| Compatibility | все 208 прежних tests и mono CLI/WAV/atomic | Прежние expected |

Diagnostics проверяют code, SourceId, primary/secondary spans и frame index там, где применимо. DSP использует посемпловые comparisons; expected не вычисляется production функцией.

## Критерии приёмки подготовительной задачи

- [x] Diff ограничен allowlist; основная семантика не реализована агентом.
- [x] SPEC/ADR/stage/task отражают выбранные contracts до production-реализации.
- [x] Каркас компилируется; ровно восемь разрешённых `todo!()` и никаких других placeholders.
- [x] Tests покрывают positive/negative/boundary и independent oracles без ignored.
- [x] Все 208 прежних tests проходят; старые assertions не ослаблены.
- [x] Все проверки запущены; ожидаемые failures и непроверенное явно перечислены.
- [x] Этап 5 не отмечен завершённым и README не обещает работающие новые builtins.

## Обязательные команды подготовительной задачи

```bash
cargo test --test oscillators --locked --no-run
cargo test --test stereo --locked --no-run
cargo test --test noise --locked --no-run
cargo test --all-targets --locked --no-run
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
cargo test --doc --locked
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked
```

Дополнительно `git diff --check`, поиск всех `todo!`/`unimplemented!` и полный `--no-fail-fast` для учёта ожидаемых failures. Новые красные tests не являются завершением production-этапа.

## Отчёт об исполнении

- Изменённые файлы ограничены согласованным allowlist: SPEC/ADR/stage/task/status docs, перечисленные `src` representations/plumbing, `tests`, `fixtures` и `test-support/reference`. `Cargo.*`, toolchain и dependencies не изменены.
- Добавлены 28 integration tests, девять fixtures, mono/stereo/noise generators и independent phase/pan/SplitMix64 oracles. Ручной stereo VM/WAV scenario создаёт exact 56-byte PCM16 output; allocation harness подтверждает 0 steady-state allocations.
- Все 208 прежних integration tests и allocation harness проходят. Новые suites: 5/5 expected failed в `oscillators`, 7/3 в `stereo`, 4/4 в `noise`. Aggregate 236 integration + 1 allocation test: 225 passed, 12 expected failed, 0 ignored.
- Все targets компилируются через `--no-run`; `cargo fmt --check`, `git diff --check`, clippy `-D warnings`, doctest и rustdoc `-D warnings` успешны. Полный `--no-fail-fast` учитывает 12 failures только на восьми документированных заглушках.
- Согласованный `mix`/builtin function-name break и миграция fixture `mix_signals` записаны в changelog. Unknown-function fixture использует `noize`, поскольку `noise` стал descriptor.
- Найденные подготовительные ошибки (Stereo function lookahead, arithmetic/WAV expected, повторный span и параллельный allocation counter) исправлены до отчёта. Production algorithms, generated end-to-end execution и real stage-5 CLI не проверены; этап 5 не завершён.

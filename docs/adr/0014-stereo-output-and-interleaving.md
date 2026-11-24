# 0014. Явный Stereo и interleaved двухканальный output

- Статус: принят.
- Дата: 2026-11-21.
- Дополняет: ADR 0003, 0004, 0005, 0007 и 0009.

## Контекст

Текущие Typed AST, IR, bytecode, VM и WAV допускают только один `Sig` output. Этапу 5 нужны независимые left/right каналы без преждевременного общего multichannel API. Нужно одновременно определить source typing, владение двумя каналами в graph/register layout, единицы block size и порядок PCM interleaving.

## Варианты

1. Кодировать stereo парой неявных `Sig` без отдельного типа. Отклонено: тип `pan`/`stereo` и допустимый `out` были бы неоднозначны.
2. Добавить явный мономорфный `Stereo`, ровно два канала и interleaved VM output. Это минимальное расширение текущей VM и WAV.
3. Сразу добавить `Channels`, произвольное channel count и planar generic buffers. Это избыточно для текущего этапа и требует отдельной resource/type модели.

## Решение

Выбран вариант 2.

### Язык и типы

```text
type_name = Float | Sig | Stereo

stereo(left: Sig, right: Sig): Stereo
pan(input: Sig, position: Sig): Stereo
```

- `Stereo` — first-class graph-construction value: его можно связывать, передавать и возвращать из пользовательских функций с обязательными аннотациями.
- `out` принимает `Sig` или `Stereo`. `Float → Sig` lift прежний; неявных `Float → Stereo`, `Sig → Stereo`, `Stereo → Sig` нет. Mono не дублируется автоматически в два канала.
- Арифметика и unary minus над `Stereo` не добавляются. `gain`/`mix` принимают только `Sig`; композиция каналов выполняется явно до `stereo`.
- `stereo` вычисляет left, затем right ровно один раз и сохраняет sharing. Неиспользуемые stereo values остаются eager roots.

### pan

```text
p = clamp(position, -1, +1)
angle = (p + 1) × π / 4
left  = input × cos(angle)
right = input × sin(angle)
```

Center `p = 0` даёт gain `sqrt(1/2)` на каждый канал; hard left/right дают `(input, 0)` и `(0, input)` в пределах numeric tolerance. Значения вне `[-1, 1]` clamp’ятся. Нечисловая position возвращает `E4001` с primary position expression, frame index и call origins; nonfinite input распространяется по IEEE и обнаруживается на output. `pan` stateless, требует два audio registers и имеет capability `Multichannel`.

### IR и bytecode

- `ValueType::Stereo` обозначает ровно два `Sig` канала. `NodeOp::Stereo` и `NodeOp::Pan` возвращают `Stereo`; обычные scalar/audio ops не принимают этот тип.
- Codegen связывает stereo node с парой последовательно выделенных, но явно адресуемых audio registers. `MakeStereo`/`Pan` записывают left/right; отдельный state не нужен.
- Bytecode содержит явный `OutputLayout::{Mono,Stereo}`. Dedicated output registers не записываются обычными instructions; финальная `SetOutput` или `SetStereoOutput` выбирает layout.
- Verifiers проверяют channel types, register initialization, output layout и отсутствие записи в любой dedicated output register. Debug origins сохраняются по одному на instruction; stripping не меняет layout или samples.

### VM и frames

`Vm::render_into(&mut [f32])` возвращает mono samples либо interleaved stereo:

```text
mono:   M0, M1, M2, ...
stereo: L0, R0, L1, R1, ...
```

- `block_size` — maximum frames, а не число скалярных `f32`. Для stereo output slice содержит чётное число values и не более `block_size × 2`; mono контракт длины не меняется.
- `sample_index` и `RenderReport.samples` считают frames. Пустой slice — no-op; odd stereo slice и oversized slice отклоняются до dispatch.
- Для каждого frame instructions сохраняют Vec order. На output сначала проверяется left, затем right; первая nonfinite channel даёт structured error с frame index. Выходной slice копируется только после успешного dispatch всего блока по прежнему правилу.
- Все registers и interleaved scratch выделяются заранее. Steady-state allocations, filesystem/network/clock I/O отсутствуют.

### WAV

Mono сохраняет прежний 44-byte PCM16 header. Stereo использует channels `2`, block align `4`, byte rate `sample_rate × 4` и data size `frames × 4`. PCM16 quantization/clamp прежний для каждого канала; запись — `L0,R0,L1,R1,...` little-endian. RIFF sizes используют checked arithmetic с фактическим channel count. Atomic publish и diagnostics не меняются.

## Последствия

- Старые mono programs, block sizes, sample indices, PCM bytes и CLI report сохраняются.
- Stereo memory растёт по числу audio registers и максимум в два раза для output scratch, а не по длительности render.
- Явный `OutputLayout` исключает угадывание channel count из длины slice.
- Общий multichannel, channel arrays, downmix/upmix и planar public API не появляются в этом этапе.

## Проверка

- Parser/sema: `Stereo` annotations, lifts, out acceptance/rejection, function signatures, no Stereo arithmetic и exact spans.
- Explicit stereo graph/bytecode layout, malformed registers/output/state corruption и debug stripping.
- Раздельные left/right signals, interleaving, odd/oversized buffers, variable frame partitions и reset.
- Pan vectors при `-2/-1/0/1/2`, nonfinite position, nonfinite output left-before-right и origins.
- Независимо рассчитанный 44-byte stereo header и PCM16 byte vectors; mono WAV fixtures остаются побайтово прежними.
- Allocation counter проверяет mono и stereo `render_into`/`reset` после инициализации.

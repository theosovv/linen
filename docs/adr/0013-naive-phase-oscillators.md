# 0013. Наивные фазовые осцилляторы и арифметические builtins

- Статус: принят.
- Дата: 2026-11-21.
- Дополняет: ADR 0004, 0005, 0006, 0007 и 0009.

## Контекст

Этап 5 добавляет `phasor`, `saw`, `square`, `triangle`, `gain` и `mix`. Для каждого stateful-узла нужны однозначные начальное состояние, порядок output/update, отрицательные и нечисловые частоты, reset и block-invariance. Наивные формы волн обязаны оставаться отличимыми от будущего anti-aliasing.

## Варианты

1. Вычислять каждую формулу от абсолютного `sample_index`. Это плохо поддерживает сигнальную частоту и расходится с состоянием `sine`.
2. Хранить независимую `f64` phase в периодах у каждого вызова и вычислять waveform до обновления фазы. Это повторяет проверенный контракт ADR 0006.
3. Нормализовать фазу один раз за аудиоблок или скрыто заменить naive waveform на PolyBLEP. Оба варианта меняют наблюдаемую семантику и запрещены.

## Решение

Выбран вариант 2. Все четыре новых осциллятора имеют сигнатуру `(freq: Sig): Sig`, начальную phase `0.0` и общий transition:

```text
if runtime freq is not finite:
    E4001 before changing phase
value = waveform(phase) as f32
phase = (phase + f64(freq) / sample_rate).rem_euclid(1.0)
```

Формулы до преобразования в `f32`:

| Builtin | `waveform(phase)` |
|---|---|
| `phasor` | `phase` |
| `saw` | `2 × phase − 1` |
| `square` | `+1`, если `phase < 0.5`; иначе `−1` |
| `triangle` | `4 × phase − 1`, если `phase < 0.5`; иначе `3 − 4 × phase` |

- Диапазоны: `phasor` — `[0, 1)`, `saw`/`triangle` — `[-1, 1]`, `square` — `{-1, +1}`.
- Нулевая частота повторяет начальное значение; отрицательная использует Euclidean modulo; конечная частота выше Nyquist даёт естественный aliasing.
- Это наивные sampled waveforms без band limiting. PolyBLEP или другой anti-aliasing будет отдельным узлом/подэтапом и не заменит эти формулы скрыто.
- Каждый вызов и каждое function-body инстанцирование получают новый state slot. Sharing binding разделяет узел по ADR 0007. `reset()` возвращает phase к `0.0`.
- Origin phase-узла указывает на frequency expression и сохраняет inner-to-outer call chain. Нечисловая frequency даёт существующую `E4001` с zero-based frame index.
- Каждый узел требует один audio register, один state slot и одну `f64` phase. Capability — `CoreAudio`.

### gain и mix

```text
gain(input: Sig, amount: Sig): Sig = input × amount
mix(a: Sig, b: Sig, balance: Sig): Sig = a × (1 − balance) + b × balance
```

`Float` аргументы получают обычные explicit lifts. `balance` не ограничивается: `0` выбирает `a`, `1` выбирает `b`, остальные конечные значения интерполируют или экстраполируют по формуле. Dynamic arithmetic сохраняет IEEE 754; нечисловой результат обнаруживается на output по общему контракту.

Оба builtin являются graph-construction sugar и не получают отдельных runtime instructions. Lowering вычисляет аргументы слева направо ровно один раз и повторно использует `NodeId` там, где формула ссылается на `balance` дважды. Synthetic arithmetic nodes получают полный call span и текущий call origin chain. Descriptor самого sugar не добавляет registers/state; фактические lowered nodes получают обычные resource descriptors.

## Последствия

- Числовой результат не зависит от block partition, `SourceId`, `NodeId` или debug info.
- `sine` сохраняет прежний алгоритм и не обязан механически переписываться на новый phase type.
- Два вызова с одинаковыми аргументами не делят phase, даже если их первые samples совпадают.
- Независимый reference записывает формулы и transitions отдельно от production DSP.

## Проверка

- Явные vectors при phase `0`, `0.125`, `0.25`, `0.5`, `0.75` и `0.875`, включая точные discontinuity-adjacent значения.
- Нулевая, отрицательная и выше-Nyquist частота; nonfinite frequency не меняет phase.
- Block sizes `1/3/64/128/256`, variable partitions, reset и long-render phase bounds.
- Аналитические RMS/DC/peak/symmetry checks дополняют, но не заменяют посемпловые vectors.
- `gain`/`mix`: exact arithmetic vectors, `balance` `-1/0/0.5/1/2`, sharing stateful arguments и function fresh state.
- IR/bytecode verifiers проверяют типы, origins, state uniqueness и resource coverage; прежние sine/mono tests не меняют expected.

# 0015. Детерминированный noise с явным seed и SplitMix64

- Статус: принят.
- Дата: 2026-11-21.
- Дополняет: ADR 0004, 0005, 0007 и 0009.

## Контекст

Этап 5 включает `noise`, но спецификация запрещает системный random и зависимость потока от `NodeId`, `SourceId`, адреса, instruction index, inlining или optimization. Нужны воспроизводимый алгоритм, стабильная stream identity, явная начальная точка reset и переносимые numeric vectors.

## Варианты

1. `noise(): Sig` с общим seed в `RenderConfig`. Требует дополнительного правила назначения разных streams экземплярам после inlining.
2. `noise(seed: Float): Sig` с явным per-node seed. Одинаковый seed намеренно даёт одинаковую последовательность, а stream identity не зависит от compiler layout.
3. Системный random, hash `NodeId` или address-based seed. Запрещены детерминизмом и Source identity инвариантами.

Для алгоритма рассматривались PCG32 и SplitMix64. SplitMix64 выбран за короткий полностью определённый unsigned transition, отсутствие special zero state и простые независимые vectors.

## Решение

Выбран вариант 2 со SplitMix64.

### Source contract

```text
noise(seed: Float): Sig
```

Seed вычисляется на graph construction и обязан быть конечным целым в `0..=9_007_199_254_740_991` (`2^53−1`), затем без потерь представляется как `u64`. Finite fractional, negative и out-of-range seed — новая `E2107`. `-0` эквивалентен seed `0`. Существующие `E2104`/`E2105` сохраняют приоритет для ошибок вычисления и нечислового `Float`.

Seed, зависящий от Float parameter, проверяется при каждой function конкретизации. Primary `E2107` — исходное seed expression; active calls добавляются secondary inner-to-outer. Невызванная функция проверяет известные seeds и допускает parameter-dependent seed до конкретизации.

### RandomStreamId и состояние

```text
RandomStreamId = explicit seed as u64

initial:
    initial_seed = seed
    state = seed

reset:
    state = initial_seed
```

`RandomStreamId` равен явному seed и не вычисляется из definition/index/address. Равные seeds в разных nodes, sources и function calls дают одинаковые streams; каждый node при этом имеет независимое текущее состояние. Разные function calls получают fresh state по ADR 0007. Изменение seed — сознательное изменение render compatibility.

IR хранит seed input и `RandomStreamId`; verifier проверяет их согласованность. Codegen переносит stream ID в `Noise` instruction независимо от node/instruction numbering. Runtime state состоит из initial seed и текущего `u64` state и занимает один state slot.

### SplitMix64 transition

Все сложения и умножения выполняются с wrapping modulo `2^64`; shifts и XOR имеют точную unsigned semantics:

```text
next_sample:
    state = state + 0x9E3779B97F4A7C15
    z = state
    z = (z XOR (z >> 30)) × 0xBF58476D1CE4E5B9
    z = (z XOR (z >> 27)) × 0x94D049BB133111EB
    z = z XOR (z >> 31)
    value = float32(z >> 40) / 8388608.0 − 1.0
    return value
```

Используются верхние 24 бита результата. Деление на `2^23` точно представимо в binary floating point; output находится в `[-1, 1)` и не является NaN/Inf. Один transition выполняется на frame независимо от output channel count и block partition. Узел не имеет audio inputs.

Noise получает один audio register, один state slot, `CoreAudio` capability и resource metadata, включающие initial/current `u64`. Debug stripping и output layout не меняют stream.

## Последствия

- Для повторяемого различия двух noise nodes пользователь задаёт разные seeds явно.
- В одинаковых seed nodes samples совпадают по контракту, а не случайно; независимость означает отдельное продвижение/reset состояния.
- Нет скрытого `RenderConfig` random state и необходимости стабилизировать NodeId.
- Будущие modules/optimizations обязаны сохранять explicit stream ID; другой PRNG требует нового ADR и changelog.

## Проверка

- Независимо рассчитанные fixed `u64` и audio vectors для seeds `0`, `1`, `2^53−1`, включая wrapping cases.
- Exact seed boundaries: `-0`, `0`, max accepted; `-1`, `0.5`, `2^53` rejected; existing nonfinite/division diagnostics сохраняют приоритет и spans.
- Одинаковый/разный seed, independent state, function inlining/fresh state, reset, block sizes и variable partitions.
- Повторные processes, SourceId/NodeId/instruction reordering без изменения stream ID, debug stripping и mono/stereo output.
- Reference содержит отдельную реализацию SplitMix64; production expected не вызывает production DSP.

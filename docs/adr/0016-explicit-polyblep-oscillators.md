# 0016. Явные PolyBLEP-осцилляторы и граница будущей stdlib

- Статус: принят.
- Дата: 2026-11-22.
- Дополняет: ADR 0004, 0005, 0006, 0007, 0009 и 0013.

## Контекст

Naive `saw` и `square` этапа 5 намеренно дают естественный aliasing и уже имеют закреплённые samples/compatibility fixtures. Anti-aliasing нельзя подменить скрытой сменой их алгоритма. Одновременно таблица builtins не должна превращаться в каталог всех известных методов синтеза: композиции и алгоритмы, выразимые на linen, в будущем должны входить в явно импортируемую стандартную библиотеку.

Для первого anti-aliasing подэтапа выбран классический 2-point PolyBLEP. Он требует только уже существующей фазы, не выделяет память в steady-state и не зависит от фильтров этапа 6. Он уменьшает aliasing, но не является идеальным band-limited синтезом.

## Варианты

1. Заменить naive `saw`/`square` на PolyBLEP. Это незаметно меняет существующий звук и запрещено ADR 0013.
2. Добавить общий oscillator с параметром выбора алгоритма или глобальный AA mode. До variants/modules это создаёт преждевременный API и смешивает конфигурацию с семантикой конкретного узла.
3. Добавить низкоуровневый `polyblep()` и собирать осцилляторы на linen. Текущий язык не имеет необходимых phase/select/sample-rate операций; добавлять их ради этого подэтапа расширило бы его за пределы DSP-задачи.
4. Добавить два algorithm-specific builtins `saw_polyblep` и `square_polyblep`. Naive-узлы сохраняются, алгоритм явен в исходнике, а будущая stdlib может предоставлять обёртки и композиции без изменения VM.
5. Сразу добавить oversampling, minBLEP, DPW, PolyBLAMP или wavetable variants. Они требуют других состояний, таблиц, фильтров и числовых контрактов и не входят в минимальный подэтап.

## Решение

Выбран вариант 4. Подэтап 5D добавляет только:

```text
saw_polyblep(freq: Sig): Sig
square_polyblep(freq: Sig): Sig
```

Это отдельные IR/bytecode/VM ops с независимой `f64` phase, начальным значением `0.0`, одним audio register и одним state slot. Capability — `CoreAudio`. `Float → Sig`, origins, fresh state при инстанцировании функций, sharing и reset следуют ADR 0006/0007/0013.

### Точный числовой transition

Runtime frequency сначала переводится в `f64` и проверяется на конечность. NaN/Inf возвращают существующую `E4001` до изменения phase. Для конечной frequency:

```text
phase_step = frequency / sample_rate
dt = abs(phase_step)
t = phase                              # [0, 1)
```

Все промежуточные вычисления ниже выполняются в `f64`. Коррекция определяется функцией `B(t, dt)`:

```text
if dt == 0 or dt > 0.5:
    B = 0
else if t < dt:
    x = t / dt
    B = 2*x - x*x - 1
else if t > 1 - dt:
    x = (t - 1) / dt
    B = x*x + 2*x + 1
else:
    B = 0
```

Сравнения строгие; `dt == 0.5` остаётся в PolyBLEP path. Явный naive fallback при `dt == 0` или `dt > 0.5` не является ошибкой и не меняет phase update.

```text
saw_naive = 2*t - 1
saw_sample = saw_naive - B(t, dt)

square_naive = +1, если t < 0.5; иначе -1
square_shifted = t + 0.5, если t < 0.5; иначе t - 0.5
square_sample = square_naive + B(t, dt) - B(square_shifted, dt)
```

Ветвь shifted phase важна для числового контракта: `(t + 0.5).rem_euclid(1)` при округлении суммы к `1.0` перешёл бы к нулю и выбрал противоположную сторону поправки. Здесь `B(1.0, dt)` вычисляется правой ветвью без modulo.

Результат преобразуется в `f32` только после всех поправок. Затем независимо от выбранного waveform/fallback:

```text
phase = (phase + phase_step).rem_euclid(1.0)
```

Отрицательная frequency использует `abs` только для ширины коррекции; phase update остаётся signed. При нулевой frequency повторяется naive waveform текущей phase. В исходном состоянии это `-1` для saw и `+1` для square. На первом семпле с ненулевой `abs(freq) <= Nyquist` обе PolyBLEP-формы дают `0`. Выше Nyquist вывод равен соответствующему naive waveform на той же phase; конечная высокая frequency не отклоняется.

### Граница ядра и stdlib

Название builtin называет конкретный алгоритм, а не обещает универсальный `*_aa` или идеальный band-limit. Другие AA-алгоритмы требуют отдельной задачи, числового контракта и ADR. Их наличие в этом документе не является placeholder или разрешением на реализацию.

Когда появятся modules/source libraries, пользовательские обёртки, композиции и выбор алгоритма должны предпочтительно реализовываться на linen. Новый builtin оправдывается отдельно только для примитива, которому нужны собственные VM state/codegen/performance и который нельзя разумно выразить существующим языком. Модульная система и stdlib в подэтапе 5D не реализуются.

## Последствия

- Все naive samples, fixtures, mono/stereo layout, WAV и diagnostics прежних этапов сохраняются.
- Сознательный compatibility break ограничен function declarations с именами `saw_polyblep` и `square_polyblep`: как и для других builtins, они отклоняются `E2000`. Одноимённые value bindings остаются допустимыми.
- Каждый узел сохраняет resource model прежнего phase oscillator; дополнительных buffers, tables, delay, latency и steady-state allocations нет.
- PolyBLEP не распространяется на `sine`, `phasor`, naive `saw`/`square`, `triangle`, функции или `RenderConfig` неявно.
- Подготовительный diff может содержать только три явно перечисленные в stage/task document заглушки; они не являются работающей реализацией и удаляются до завершения 5D.

## Проверка

- Явные посемпловые vectors для saw/square вне и внутри обеих зон коррекции, включая wrap и half-cycle discontinuity.
- Нулевая, отрицательная, exactly-Nyquist, above-Nyquist и nonfinite frequency; phase не меняется на ошибке.
- Независимый reference повторяет спецификационные formulas отдельно от production DSP; analytic vectors проверяют сам reference.
- Block sizes `1/3/64/128/256`, variable partitions, reset, long render, function sharing/fresh state, origins и debug stripping.
- IR/bytecode type/state/origin corruption cases и descriptor/resource coverage.
- Spectral acceptance: 48 000 frames при sample rate 48 000 Hz и frequency 1001 Hz; для bins `[48, 96, 144, 857, 905, 953, 1049, 1097, 1145, 1858, 1906, 1954]` сумма квадратов magnitudes у PolyBLEP должна быть не менее чем на 20 dB ниже naive. Это дополнение, а не замена посемпловых tests.
- Все прежние tests проходят без изменения audio expected; новые generated differential/property и allocation scenarios добавляются до финальной приёмки 5D.

# 0005. Регистровый bytecode и блочная VM

- Статус: принят.
- Дата: 2026-11-20.
- Дополняет: ADR 0004, 0006 и 0009.

## Контекст

Этап 3 заменяет посемпловый production interpreter блочным execution path. Нужно сохранить `f64` scalar arithmetic, `f32` audio samples, состояние `sine` на каждом семпле, стабильные source diagnostics и независимость от block partition. Bytecode пока не является persisted format или stable ABI.

Спецификация называет scalar/audio registers, state slots, output и conceptual instructions, но оставляет физический layout ADR. Также требуется отделить structured VM error от source-aware `Diagnostic`: VM не зависит от CLI/WAV и не обязана владеть `SourceMap`.

## Варианты

1. Стековая VM с неявными operand stack и untyped values. Отклонена: спецификация выбрала регистровую typed VM, а verifier должен отклонять type/index errors до dispatch.
2. Регистровая VM с отдельными scalar/audio index spaces и state IDs. Соответствует спецификации и делает проверки явными.
3. Использовать `NodeId` как state index и instruction index. Отклонено: удаление/перестановка instructions не должно незаметно менять state layout.
4. Встраивать source `Diagnostic` в VM. Отклонено: это связывает runtime с frontend и затрудняет stripping debug info.
5. Выполнять scalar instructions один раз при инициализации. Возможная поздняя оптимизация, но на этапе 3 выбран единый простой sample-major проход instructions на каждый sample.
6. Выполнять каждую instruction над всем блоком перед переходом к следующей. Отклонено после review: поздняя ошибка ранней instruction могла скрыть более ранний по sample index diagnostic следующей instruction, меняя legacy fail-fast порядок при смене block size.

## Решение

Выбран вариант 2 с простым неоптимизированным dispatch.

### Bytecode

```text
Bytecode
  instructions: Vec<Instruction>
  scalar_register_count: usize
  audio_register_count: usize
  state_count: usize
  output: AudioRegister
  debug_origins: Option<Vec<NodeOrigin>>

ScalarRegister / AudioRegister / StateId
  zero-based typed index в собственном index space

Operand
  Scalar(ScalarRegister)
  Audio(AudioRegister)
```

Instructions:

```text
ConstScalar   { dst: ScalarRegister, value: f64 }
Lift          { dst: AudioRegister, src: ScalarRegister }
NegScalar     { dst: ScalarRegister, src: ScalarRegister }
NegAudio      { dst: AudioRegister, src: AudioRegister }
BinaryScalar  { op, dst: ScalarRegister, lhs: ScalarRegister, rhs: ScalarRegister }
BinaryAudio   { op, dst: AudioRegister, lhs: Operand, rhs: Operand }
Sine          { dst: AudioRegister, freq: Operand, state: StateId }
SetOutput     { src: AudioRegister }
```

- Codegen выделяет один register на IR node в canonical node order; buffer reuse и scalar/audio peephole optimization отсутствуют.
- Для signal binary и `Sine` обычный codegen использует audio operands, потому что lifts уже явно представлены в IR. Verifier и VM также допускают scalar `Operand` для заданного спецификацией bytecode contract.
- Дополнительный dedicated `output` audio register выделяется после node registers. Ровно одна финальная `SetOutput` копирует output node в него. Другие instructions не пишут dedicated output register.
- Каждый `Sine` получает уникальный последовательный `StateId`; state IDs не выводятся из `NodeId` или instruction index.
- `debug_origins`, если присутствует, содержит ровно один `NodeOrigin` на instruction в том же порядке. Codegen отдельным аргументом получает origin полного `out` value expression для `SetOutput`, поэтому alias или frequency origin `Sine` не подменяют его. `strip_debug_info()` удаляет всю коллекцию и не меняет instructions/registers/state layout.

### Bytecode verifier

Structural verifier вызывается до создания VM и проверяет:

1. register indices по отдельным count fields и инициализацию каждого input перед чтением;
2. typed operands для каждой instruction и запрет записи в dedicated output кроме `SetOutput`;
3. finite `ConstScalar`;
4. существование, уникальность и полное покрытие state slots;
5. ровно одну финальную `SetOutput`, инициализированный src и in-range dedicated output;
6. отсутствие чтения uninitialized output или registers.

Initialized/state indices хранятся только для фактически встреченных instructions, без allocation размером в declared register/state count. Origin verifier дополнительно принимает исходную `SourceMap`: если `debug_origins = Some`, его длина равна числу instructions и каждый span валиден. `None` — законный compact bytecode. Ошибки verifier возвращаются как structured `BytecodeVerifyError`; повреждённая программа не достигает dispatch.

### VM creation и state

```text
VmConfig
  sample_rate: u32 = 48000
  block_size: usize = 128

Vm::new(&Bytecode, VmConfig) -> Result<Vm, VmCreateError>
Vm::reset()
Vm::render_into(&mut [f32]) -> Result<(), VmRuntimeError>
```

- `sample_rate` находится в `8000..=384000`; `block_size > 0` и является maximum block length для этой VM.
- До verifier metadata и runtime allocation проверяются arithmetic overflow и Vec capacity limits. При создании выделяются все scalar registers, audio buffers длины `block_size` и state slots. Audio registers заполняются нулями; `Sine` phase равна `0.0`.
- `render_into` принимает пустой блок как no-op и непустые блоки длины `1..=block_size`. Больший блок возвращает typed error до исполнения instructions.
- Внешний цикл dispatch проходит samples внутри блока, внутренний — instructions в Vec order. Scalar instructions повторяются на каждом sample как простой baseline; block size не влияет на ordering runtime errors.
- Scalar values и operations используют `f64`; lift и audio operations используют `f32`. Dynamic division сохраняет IEEE 754, включая NaN/Inf.
- `Sine` вызывает существующий production `dsp::Sine` на каждом sample. Nonfinite frequency возвращает structured error с `StateId`, instruction index и absolute zero-based sample index; phase ошибочного oscillator не меняется.
- `SetOutput` возвращает structured `NonFiniteOutput { sample_index }` до перехода к следующему sample. Все bindings текущего sample уже вычислены, поэтому их frequency errors сохраняют прежний приоритет над output error.
- VM отслеживает absolute sample index между variable-size calls. `reset()` сбрасывает scalar/audio registers, все states и sample index. Повторный render начинается с исходного состояния.
- После успешного dispatch содержимое dedicated output register копируется в caller slice. Пустой вызов не продвигает состояние или sample index.

### Runtime errors и renderer

`VmRuntimeError` не содержит `Diagnostic` и не выполняет source lookup. Для nonfinite frequency он несёт instruction/state/sample indices, для nonfinite output — sample index. Production renderer использует retained bytecode debug origins для преобразования frequency error в существующую `E4001` с frequency span, а output error локализует полным span `out` value.

Если caller удалил debug info, звук не меняется. Попытка source-aware renderer преобразовать frequency error без необходимого origin возвращает explicit internal/render error, а не выбирает другой span и не паникует. Output error не требует bytecode origin. Обычный production compile сохраняет origins, поэтому CLI diagnostics не меняются.

`render` выделяет один scratch block вне steady-state loop, разбивает checked `render_samples` на части не больше configured block size и quantizes samples после каждого успешного `render_into`. Streaming WAV header, PCM16 conversion, protective limit и atomic CLI publish остаются прежними.

### Ресурсы и запрещённые возможности

Steady-state `render_into` не выделяет память и не выполняет filesystem, console, network, environment или clock I/O. Ошибки возвращаются через существующие предвыделенные values, без форматирования strings в hot path.

Этап не добавляет serialized/persisted bytecode, versioned package format, stable ABI, `inspect`, optimization, SIMD, JIT, real-time backend или plugin interfaces. Если persistence появится отдельной задачей, format и VM version будут явными с первой версии.

## Последствия

- Повреждённые indices/types/state/layout отклоняются до dispatch, а runtime unsupported instruction не заменяется тишиной.
- Отдельные typed index spaces исключают accidental scalar/audio mixing.
- Sample-major dispatch сохраняет не только samples, но и первый diagnostic/span/sample index при разных partitions; возможная оптимизация pure instructions требует отдельной differential-проверки.
- Простой один-register-per-node codegen расходует больше памяти, но даёт очевидный baseline для последующих optimizations.
- Debug stripping меняет доступность source localization, но не sample semantics.
- Для доказательства zero-allocation steady-state нужен отдельный allocation-counting test harness; функциональные tests сами по себе это не доказывают.

## Проверка

- Вручную повреждённый bytecode покрывает unknown/uninitialized/mistyped registers, invalid/duplicate/missing states, nonfinite constants, отсутствующую/раннюю/повторную `SetOutput`, output writes, invalid/mismatched origins и экстремальные count-fields.
- Explicit vectors проверяют scalar `f64`, lift, audio/scalar operands, arithmetic, sine и output copy; exact-span test проверяет полный out origin у `SetOutput` для alias.
- VM сравнивается с independent reference на согласованном corpus; analytic sine vectors остаются основным correctness oracle.
- Block sizes `1`, `3`, `64`, `128`, `256`, empty calls и deterministic variable partitions сравниваются посемплово; отдельные regressions сравнивают первый frequency/output diagnostic на всех required block sizes.
- Reset повторяет samples и первый runtime error sample index; oversized block и nonfinite frequency не продвигают ошибочный oscillator.
- Debug-stripped и regular bytecode дают одинаковый звук. Allocation counter и release benchmark выполняются перед финальным DoD, но не в подготовительном scaffold diff.

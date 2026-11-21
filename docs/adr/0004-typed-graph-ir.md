# 0004. Типизированный сигнальный IR

- Статус: принят.
- Дата: 2026-11-20.
- Дополняет: ADR 0001, 0003, 0006, 0009 и 0012.

## Контекст

Этап 2 завершён: Typed AST содержит resolved IDs, типы, явные `Float → Sig` lifts и source spans, а production render использует наивный интерпретатор. Этап 3 должен заменить execution engine без изменения языка, WAV и diagnostics.

Спецификации недостаточно для механического lowering в трёх местах: нужно определить представление `NodeId`, сохранение top-level evaluation и sharing, а также проверку source origins. Отдельно важно не потерять текущее поведение неиспользуемых bindings: наивный interpreter и reference вычисляют все top-level values в порядке исходника, включая runtime errors вне `out`.

## Варианты

1. Хранить дерево выражений и заново исполнять общие subexpressions. Это ближе к Typed AST, но не даёт сигнального DAG и может дублировать stateful-узлы.
2. Хранить DAG в `Vec<Node>` с typed `NodeId`, canonical topological order и явным списком evaluation roots. Это даёт детерминированный layout и простой codegen без hash ordering.
3. Оставить только `output` и удалять неиспользуемые bindings при lowering. Это проще, но меняет существующее поведение runtime errors и нарушает compatibility цель этапа.
4. Разрешить произвольный порядок узлов и топологически сортировать граф в codegen. Это допустимо, но добавляет алгоритм и варианты ordering раньше появления optimization passes.
5. Сразу поддержать merged/unknown и inline origins. На этапе 3 нет ни inlining, ни optimization, поэтому такие состояния были бы преждевременными.

## Решение

Выбран вариант 2; все top-level bindings сохраняются как evaluation roots. Варианты 3–5 отклонены для текущего этапа.

### Представление графа

```text
Graph
  nodes: Vec<Node>
  roots: Vec<NodeId>
  output: NodeId
  render_samples: u64

Node
  id: NodeId
  op: NodeOp
  result_type: ValueType
  origin: NodeOrigin

NodeOrigin
  primary: SourceSpan
```

- `NodeId` — zero-based индекс в `nodes`, действительный только для конкретной версии графа.
- `Node.id` обязан совпадать с индексом. Каждый input имеет индекс строго меньше индекса consuming node. Таким образом canonical graph уже топологически упорядочен, а циклы и forward inputs отклоняются verifier.
- `roots` содержит value каждого top-level binding в declaration order. Несколько roots могут ссылаться на один `NodeId`; это sharing, а не повторное вычисление.
- `output` — value binding `out`, обязан входить в `roots` и иметь тип `Sig`. `render_samples` копируется из проверенного Typed AST и не пересчитывается lowering.
- Все nodes достижимы хотя бы из одного root. Dead-node elimination не выполняется.
- `NodeOrigin` на этапе 3 содержит только `primary`. `inlined_from`, merged и unknown origins появятся только вместе с реальными преобразованиями, которым они нужны.

Начальный набор операций:

```text
ConstFloat(f64)
LiftFloatToSig(NodeId)
Neg(NodeId)
Add(NodeId, NodeId)
Sub(NodeId, NodeId)
Mul(NodeId, NodeId)
Div(NodeId, NodeId)
Sine(NodeId)
```

`ConstFloat` и scalar arithmetic используют `Float`/`f64`; signal operations используют `Sig`/runtime `f32`. Входы `Neg` имеют тип результата. Входы binary operation имеют одинаковый тип, совпадающий с результатом: mixed arithmetic уже содержит явные lift nodes. `Sine` принимает и возвращает `Sig`.

### Lowering

- Bindings обрабатываются слева направо. Для каждого `BindingId` сохраняется один lowered `NodeId`; identifier reference возвращает сохранённый ID и не создаёт node.
- Выражения обходятся слева направо, operands предшествуют результату. Никакого CSE, constant folding или reordering нет.
- `Group` прозрачен и возвращает inner `NodeId`; его span остаётся в Typed AST, но отдельный IR node не создаётся.
- Number, unary/binary, lift и `sine` создают соответствующий node с origin соответствующего Typed expression.
- Для `Sine` `origin.primary` — span frequency argument, поскольку именно он требуется стабильной `E4001`. Полный call span остаётся в Typed AST.
- Value каждого binding добавляется в `roots` после lowering в declaration order. Это сохраняет вычисление и runtime-error ordering всех bindings, включая не используемые в `out`.

### Верификация

`verify(graph, sources)` возвращает structured `IrVerifyError` и проверяет:

1. совпадение `Node.id` с Vec index;
2. существование и строго предшествование каждого input;
3. arity через вариант `NodeOp`, типы inputs/result и finite `ConstFloat`;
4. непустые roots, существование roots/output, наличие output среди roots и `Sig` output;
5. достижимость каждого node хотя бы из одного root;
6. разрешимость каждого primary span в исходной `SourceMap`, включая bounds и UTF-8 boundaries.

Ошибка verifier — internal compiler invariant, а не source diagnostic и не повод для panic на пользовательской программе. Facade возвращает structured compile error; CLI использует internal-exit `3`. Отдельные стабильные source-level `E3xxx` этим решением не вводятся.

### Capabilities и resources

Каждый текущий `NodeOp` имеет центральный descriptor с `RuntimeCapability::CoreAudio` и `ResourceModel`. Resource model указывает число scalar registers, audio registers, state slots и state bytes для одного node до учёта block size:

| Operation | Scalar registers | Audio registers | State slots | State bytes |
|---|---:|---:|---:|---:|
| `ConstFloat`, scalar `Neg`/binary | 1 | 0 | 0 | 0 |
| `LiftFloatToSig`, signal `Neg`/binary | 0 | 1 | 0 | 0 |
| `Sine` | 0 | 1 | 1 | размер одной `f64` phase |

Это classification текущих ops, а не общий target/plugin framework. Точный buffer bytes вычисляются из audio register count и выбранного maximum block size при создании VM. `BuiltinDescriptor` `sine` содержит согласованные capability/resource metadata; повторной независимой таблицы сигнатур не появляется.

## Последствия

- Codegen может проходить `nodes` напрямую, не используя `HashMap` iteration или topological sort.
- Alias roots и repeated identifiers не дублируют oscillator state.
- Неиспользуемая привязка не исчезает и сохраняет прежний runtime error.
- Строгий canonical order должен восстанавливаться каждым будущим pass либо осознанно пересматриваться новым ADR.
- Origin validation требует исходную `SourceMap`; сам граф не владеет текстом и не выполняет I/O.
- IR остаётся нестабильным in-memory Rust representation без serialized format.

## Проверка

- Structural lowering tests проверяют явные node sequences, IDs, roots, sharing, lifts и source spans.
- Вручную повреждённые графы покрывают unknown/forward inputs, ID mismatch, неправильные типы, nonfinite constants, пустые/unknown roots, output вне roots, unreachable nodes, invalid source/offset/UTF-8 origin и циклы.
- Для каждого `NodeOp` проверяются capability/resource descriptors, включая ровно один state slot у каждого `Sine`.
- Differential tests сравнивают VM с независимым reference; отдельный compatibility case доказывает runtime error в unused binding.
- Повторный compile даёт structurally equal graph в одном process и через independent CLI processes.

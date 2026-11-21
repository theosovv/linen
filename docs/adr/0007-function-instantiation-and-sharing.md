# 0007. Инстанцирование функций, sharing и call origins

- Статус: принят.
- Дата: 2026-11-21.
- Дополняет: ADR 0001, 0003, 0004, 0005, 0006, 0009 и 0012.

## Контекст

Этап 4 добавляет пользовательские функции, lexical scopes, локальные `let`, блоки-выражения и pipeline. Production execution остаётся IR/bytecode/VM: функция вызывается во время graph construction и не становится инструкцией VM. Следовательно, lowering инстанцирует тело на каждый вызов.

Нужно одновременно сохранить четыре инварианта: свежие stateful-узлы каждого вызова, sharing каждой привязки, eager runtime errors и диагностируемый источник узлов после inlining. Кроме того, композиция функций может экспоненциально увеличивать граф даже при небольшом исходнике.

## Варианты

1. Подставлять выражение аргумента при каждом обращении к параметру. Это дублирует stateful-узлы и нарушает выбранную value-семантику параметров.
2. Вычислять аргумент один раз, связывать параметр с его `NodeId`, а тело инстанцировать заново. Это сохраняет sharing аргумента и свежесть состояния тела.
3. Добавить function-call instructions в VM. Это усложнило бы runtime и не требуется текущей спецификацией.
4. Удалять неиспользуемые аргументы и locals. Это меняет eager-порядок ошибок и несовместимо с уже закреплённым вычислением всех top-level bindings.
5. Хранить только primary span после inlining. Он остаётся точным, но не объясняет, через какие вызовы узел попал в граф.
6. Выполнять inlining без semantic budget. Рекурсия запрещена, но вложенные вызовы всё равно могут создать экспоненциальный граф и исчерпать память.

## Решение

Выбран вариант 2; VM function calls не добавляются. Неиспользуемые значения вычисляются eager, origins получают call chain, а semantic expansion ограничивается явным budget.

### Lexical identity

- `BindingId` — compilation-local zero-based ID любой top-level binding, parameter или local `let`. Одинаковые имена в разных scopes получают разные IDs; повторный identifier reference сохраняет ID определения.
- `FunctionId` — отдельный zero-based ID пользовательской функции в порядке top-level function declarations. `BuiltinId` остаётся отдельным index space.
- Параметры и `let` непосредственно тела функции образуют один scope. Явно вложенный блок создаёт дочерний scope и разрешает shadowing внешнего value binding. Повторное связывание в одном scope запрещено.
- Функция видит ранее объявленные top-level values/functions, свои параметры и locals. Поздние declarations и сама функция в её теле не видимы, поэтому forward calls и прямая/косвенная рекурсия отклоняются name resolution.
- Пользовательские top-level values/functions не могут повторять имя друг друга. Пользовательская функция не может называться `sine`; существующая возможность иметь value binding `sine` и вызывать builtin `sine` сохраняется.
- Функции не являются values: function name в value position и value binding в callee position не образуют higher-order вызов.

### Аргументы, locals и roots

Для каждого пользовательского вызова:

1. проверяются callee и arity;
2. аргументы разрешаются и lower’ятся слева направо ровно один раз;
3. каждый parameter связывается с уже вычисленным `NodeId` аргумента;
4. locals тела lower’ятся в порядке исходника; каждый local `BindingId` получает один `NodeId`;
5. identifier references параметра/local/top-level binding возвращают сохранённый `NodeId`;
6. каждое исполнение тела создаёт новые nodes для собственных expressions, включая новые `Sine`/state slots;
7. после возвращения результата локальное окружение вызова удаляется.

`Graph.roots` расширяется: помимо value каждого top-level binding он содержит value каждого пользовательского аргумента и каждого local `let` в момент eager-вычисления. Внутренние roots предшествуют root enclosing binding. Duplicate `NodeId` разрешены и обозначают sharing. Невызванная функция не создаёт nodes или roots.

Это не добавляет новых runtime instructions: codegen по-прежнему исполняет canonical nodes в Vec order. Дополнительные roots делают все eager values достижимыми для IR verifier и сохраняют runtime errors неиспользуемых параметров/locals.

### Pipeline

Parser desugaring создаёт обычный named call:

```text
x |> f       → f(x)
x |> f(a, b) → f(x, a, b)
```

Аргументы сохраняют порядок `x`, затем `a`, `b`. Полный desugared call span начинается у левого выражения и заканчивается RHS; `callee.span` остаётся span имени RHS. Отдельного runtime/IR pipeline node нет.

### NodeOrigin и diagnostics

```text
NodeOrigin
  primary: SourceSpan
  inlined_from: Vec<CallOrigin>

CallOrigin
  function: FunctionId
  call_span: SourceSpan
```

- `primary` остаётся span конкретного expression в теле определения; для `Sine` — его frequency expression.
- `inlined_from` упорядочен от внутреннего вызова к внешнему. Каждый `call_span` — полный обычный или desugared pipeline call.
- Аргумент lower’ится до входа в текущую функцию и не получает её frame. Вложенный outer call уже присутствует в chain, если аргумент находится в его теле.
- Captured/shared identifier не создаёт node и не переписывает origin существующего узла.
- `SetOutput` сохраняет полный out value origin; обычные программы без функций имеют пустой chain.
- IR/bytecode origin verifiers проверяют primary и каждый call span в исходной `SourceMap`. Соответствие `FunctionId` определению проверяется typed/lowering tests через `SymbolStorage`.
- Compile-time ошибка, зависящая от конкретных аргументов, имеет primary span выражения в теле и secondary call chain от внутреннего к внешнему. Runtime `E4001` использует тот же порядок secondary labels и сохраняет sample index.

### Compile-time Float

Тело функции типизируется один раз с типами, но не concrete values параметров. Вычисление `Float` различает `Known(f64)` и `DependsOnParameters`; второе не является ошибкой внутри generic body. Литералы, деление на известный ноль и операции, не зависящие от параметров, проверяются сразу.

При каждой конкретизации вызова значения аргументов подставляются в compile-time evaluator. Все Float locals, return value и каждый inserted lift проверяются заново для этих аргументов. Так сохраняются `E2104`/`E2105`, checked `f32` conversion и возможность функции, вычисляющей `Float`. Невызванная функция всё равно проходит все проверки, не зависящие от параметров.

### Expansion budget

```text
FunctionLimits
  max_call_depth: usize = 128
  max_expanded_nodes: usize = 10_000
```

- Активный user call имеет глубину `1`; builtin calls глубину не увеличивают. Лимит `N` допускает `N` уровней.
- Node budget подсчитывает каждый Typed expression visit при конкретизации пользовательского вызова, включая arguments/body, identifiers/groups/blocks и повторные visits в разных вызовах. Обычные выражения вне user call этот budget не расходуют.
- `0` для depth или nodes запрещает пользовательские вызовы, но не ломает программы только с builtins. Лимит `N` допускает `N` visits.
- Budget применяется и к проверкам тел невызванных функций, если они содержат конкретизируемые вызовы. Обход детерминирован: declarations, arguments слева направо, locals и result.
- Нарушение — source diagnostic `E2106`, primary — полный вызов, чья конкретизация превысила лимит; активные outer calls добавляются secondary. Это не internal verifier error.

## Последствия

- Два вызова одной функции получают независимые `StateId`; повторное использование аргумента или `let` не дублирует состояние.
- IR остаётся canonical DAG без function/control-flow ops. Bytecode и VM dispatch не меняются.
- Roots этапа 3 являются подмножеством новой модели, поэтому старые graphs/layouts сохраняются.
- Стриппинг origins по-прежнему не меняет звук, но source-aware runtime diagnostic требует retained chain.
- Expansion budget ограничивает compile-time работу и доказуемо ограничивает число узлов, создаваемых checked source.
- Нестабильные Rust representations меняются без persisted/stable ABI.

## Проверка

- Scope tests: duplicate params/locals, nested shadowing, forward/self/indirect recursion, захват ранних values/functions, одинаковые имена с разными IDs.
- Sharing/state tests: явные graph/state counts, два вызова с разными частотами, аргумент использован дважды, captured binding и reset.
- Eager tests: unused parameter/local с nonfinite frequency сохраняют первый error, span и sample index при всех block sizes.
- Pipeline tests сравнивают desugared AST, typed IDs, IR и samples с явным вызовом.
- Origin tests проверяют primary, точные call spans, inner-to-outer порядок, FunctionId и verifier corruption.
- Float tests проверяют generic unknown values, per-call `E2104`/`E2105`, lifts и определение + call chain diagnostics.
- Boundary tests проверяют `0`, ровно лимит и лимит + 1 для depth/nodes, включая экспоненциальный corpus без большого allocation.
- Independent reference реализует graph-construction instantiation отдельно от production lowering; differential/property suites включают функции, блоки, pipeline, sharing и reset.

# Architecture Decision Records

ADR фиксирует нетривиальное архитектурное решение текущего этапа. Он не вводит новый языковой контракт и не заменяет `SPEC.md`.

## Порядок

1. Создавайте ADR только когда решение действительно требуется реализации.
2. Используйте следующий четырёхзначный номер и короткое имя из `SPEC.md` §5.10 либо согласуйте новое имя.
3. Укажите статус `предложен`, `принят`, `отклонён` или `заменён` и дату.
4. Опишите контекст, рассмотренные варианты, решение, последствия и способ проверки.
5. Не меняйте задним числом смысл принятого решения: для нового решения создайте ADR, который заменяет прежний.

Минимальная структура файла:

```text
# NNNN. Название
- Статус
- Дата
## Контекст
## Варианты
## Решение
## Последствия
## Проверка
```

## Индекс

- [0001 — один crate и границы модулей](0001-single-crate-module-boundaries.md).
- [0002 — source spans и структура diagnostics](0002-source-spans-and-diagnostics.md); представление offsets дополнено и заменено в части `SourceSpan` решением 0012.
- [0003 — Float, явный lift и граница наивного интерпретатора](0003-float-lift-and-signal-semantics.md).
- [0004 — типизированный сигнальный IR](0004-typed-graph-ir.md).
- [0005 — регистровый bytecode и блочная VM](0005-register-block-vm.md).
- [0006 — детерминированное состояние sine](0006-deterministic-oscillator-state.md).
- [0007 — инстанцирование функций, sharing и call origins](0007-function-instantiation-and-sharing.md).
- [0009 — независимые test oracles и числовые допуски](0009-test-oracles-and-numeric-tolerances.md).
- [0011 — лимиты parser и fail-fast diagnostics этапа 1](0011-stage-1-parser-limits-and-diagnostics.md).
- [0012 — SourceMap, SourceSpan и явный CompileSession](0012-source-map-and-compile-session.md).

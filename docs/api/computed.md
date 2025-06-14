# computed()

Creates a computed value that automatically updates when its dependencies change.

## Signature

```typescript
function computed<T>(fn: () => T): ReadonlySignal<T>

interface ReadonlySignal<T> {
  readonly value: T;
  peek(): T;
}
```

## Parameters

- `fn: () => T` - A function that calculates the computed value. Any signals accessed within this function will be tracked as dependencies.

## Returns

Returns a read-only signal object with the following properties:

- `value: T` - Getter for the computed value. Reading this property will track the computed value as a dependency in effects or other computed values.
- `peek(): T` - Method to read the current value without tracking dependencies

## Example

```typescript
import { signal, computed, effect } from 'linen';

const count = signal(0);
const doubled = computed(() => count.value * 2);

console.log(doubled.value); // 0

count.value = 5;
console.log(doubled.value); // 10

effect(() => {
  console.log(`Doubled value: ${doubled.value}`);
});
// Logs: "Doubled value: 10"

count.value = 10;
// Logs: "Doubled value: 20"
```

## Usage Notes

- Computed values are lazy by default - they only recalculate when their value is read and their dependencies have changed.
- Computed values automatically track their dependencies - any signals or other computed values accessed within the computation function.
- Computed values are cached - the computation function will only run when dependencies change.
- Use `peek()` when you want to read a computed value without creating a dependency.

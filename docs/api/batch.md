# Batch Updates

Batching updates is a mechanism that allows grouping multiple signal updates into a single transaction. This prevents unnecessary recalculations of dependent computed values and effects, significantly improving performance when multiple updates occur.

## Basic Usage

```typescript
import { signal, computed, effect, batch } from 'linen';

const count = signal(0);
const double = computed(() => count.value * 2);

effect(() => {
  console.log(`Count: ${count.value}, Double: ${double.value}`);
});

// Without batching - effect runs on each update
count.value = 1; // Output: Count: 1, Double: 2
count.value = 2; // Output: Count: 2, Double: 4
count.value = 3; // Output: Count: 3, Double: 6

// With batching - effect runs only once after all updates
batch(() => {
  count.value = 4;
  count.value = 5;
  count.value = 6;
}); // Output: Count: 6, Double: 12
```

## Nested Batches
Batches can be nested. In this case, effects will only run after the outermost batch completes:

```typescript
batch(() => {
  count.value = 1;
  
  batch(() => {
    count.value = 2;
    count.value = 3;
  });
  
  count.value = 4;
}); // Effect runs only once with value 4

```

## Transactions
transaction is an alias for batch that can be used for more semantically clear code:

```typescript
import { signal, transaction } from 'linen';

const count = signal(0);

transaction(() => {
  count.value = 1;
  count.value = 2;
  count.value = 3;
});

```

## Error Handling
If an error occurs inside a batch, all accumulated effects will run before the error is thrown:

```typescript
batch(() => {
  count.value = 1;
  throw new Error('Test error');
}); // Effect runs with value 1, then error is thrown

```

## Return Value

The batch function returns the result of the executed function:


```typescript
const result = batch(() => {
  count.value = 1;
  return 'result';
});

console.log(result); // 'result'

```

## Performance

Batching is particularly useful in the following scenarios:

1. Multiple signal updates in a loop
2. Updating several interconnected signals
3. Application state initialization
4. Handling events that affect multiple parts of the state

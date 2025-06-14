# effect()

Creates an effect that automatically runs when its dependencies change.

## Signature

```typescript
function effect(fn: () => void): () => void
```

## Parameters

- `fn: () => void` - A function that performs side effects. Any signals or computed values accessed within this function will be tracked as dependencies.

## Returns

Returns a cleanup function that, when called, will stop the effect from running.

## Example

```typescript
import { signal, effect } from 'linen';

const count = signal(0);
const name = signal('John');

// Create an effect that depends on both signals
const cleanup = effect(() => {
  console.log(`${name.value}'s count is ${count.value}`);
});
// Logs: "John's count is 0"

count.value = 1;
// Logs: "John's count is 1"

name.value = 'Jane';
// Logs: "Jane's count is 1"

// Stop the effect from running
cleanup();

// Updates no longer trigger the effect
count.value = 2; // No console output
```

## Usage Notes

- Effects run immediately when created, and then automatically whenever their dependencies change.
- Effects are used for performing side effects, such as updating the DOM, making API calls, or logging.
- Effects automatically track their dependencies - any signals or computed values accessed within the effect function.
- The cleanup function can be used to stop the effect from running, which is useful for preventing memory leaks.

## Implementation Details

Internally, effects maintain a list of their dependencies. When a signal's value is read inside an effect, the effect is registered as a dependency of that signal. When the signal's value changes, all dependent effects are re-run.
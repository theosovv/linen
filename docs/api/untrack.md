# untrack()

Prevents dependency tracking within a function.

## Signature

```typescript
function untrack<T>(fn: () => T): T
```

## Parameters

- `fn: () => T` - A function to execute without tracking dependencies.

## Returns

Returns the value returned by the function.

## Example

```typescript
import { signal, effect, untrack } from 'linen';

const count = signal(0);
const name = signal('John');

effect(() => {
  // This creates a dependency on count
  console.log(`Count: ${count.value}`);
  
  // This does NOT create a dependency on name
  const currentName = untrack(() => name.value);
  console.log(`Name: ${currentName}`);
});

// Initial effect run: "Count: 0", "Name: John"

count.value = 1;
// Effect runs again: "Count: 1", "Name: John"

name.value = 'Jane';
// Effect does NOT run again, because we used untrack for name
```

## Usage Notes

- Use `untrack()` when you want to read a signal or computed value without creating a dependency.
- This is useful for optimizing performance by preventing unnecessary recalculations.
- `untrack()` can be used with any function, not just simple signal reads.

## Implementation Details

Internally, `untrack()` temporarily sets the current effect to `null` before executing the provided function, then restores the previous effect afterward. This prevents any signals read within the function from registering the effect as a dependency.
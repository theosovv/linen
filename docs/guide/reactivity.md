# Reactivity

Linen's reactivity system is based on three core primitives: signals, computed values, and effects.

## Signals

Signals are the basic building blocks of Linen's reactivity system. They store values that can change over time and notify dependents when they change.

```typescript
import { signal } from 'linen';

const count = signal(0);
console.log(count.value); // 0

count.value = 1;
console.log(count.value); // 1
```

Signals provide a `.peek()` method to read the current value without creating a dependency:

```typescript
const count = signal(0);
const currentValue = count.peek(); // 0
```

## Computed Values

Computed values are derived from signals or other computed values. They automatically update when their dependencies change.

```typescript
import { signal, computed } from 'linen';

const count = signal(0);
const doubled = computed(() => count.value * 2);

console.log(doubled.value); // 0

count.value = 5;
console.log(doubled.value); // 10
```

Computed values are lazy by default - they only recalculate when their value is read and their dependencies have changed.

## Effects

Effects are functions that run when their dependencies change. They're useful for performing side effects like updating the DOM or making API calls.

```typescript
import { signal, effect } from 'linen';

const count = signal(0);

effect(() => {
  console.log(`Count is now: ${count.value}`);
});
// Logs: "Count is now: 0"

count.value = 1;
// Logs: "Count is now: 1"
```

Effects run immediately when created, and then automatically whenever their dependencies change.

## Untracking Dependencies

Sometimes you want to read a signal or computed value without creating a dependency. You can use the `untrack()` function for this:

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

## Batching Updates

When you need to update multiple signals at once, you can use the `batch()` function to prevent unnecessary recalculations:

```typescript
import { signal, computed, effect, batch } from 'linen';

const firstName = signal('John');
const lastName = signal('Doe');
const fullName = computed(() => `${firstName.value} ${lastName.value}`);

effect(() => {
  console.log(`Full name: ${fullName.value}`);
});
// Logs: "Full name: John Doe"

// Without batching, this would cause the effect to run twice
firstName.value = 'Jane'; // Effect runs: "Full name: Jane Doe"
lastName.value = 'Smith'; // Effect runs: "Full name: Jane Smith"

// With batching, the effect runs only once after all updates
batch(() => {
  firstName.value = 'Bob';
  lastName.value = 'Johnson';
});
// Effect runs once: "Full name: Bob Johnson"
```

Note: The `batch()` function is currently a placeholder in the implementation and will be fully implemented in the future.
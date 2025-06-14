# signal()

Creates a reactive signal that can be used to store and update state.

## Signature

```typescript
function signal<T>(initialValue: T): Signal<T>

interface Signal<T> {
  value: T;
  peek(): T;
}
```

## Parameters

- `initialValue: T` - The initial value of the signal

## Returns

Returns a signal object with the following properties:

- `value: T` - Getter/setter for the signal value. Reading this property will track dependencies, and setting it will trigger updates.
- `peek(): T` - Method to read the current value without tracking dependencies

## Example

```typescript
import { signal, effect } from 'linen';

// Create a signal with an initial value
const count = signal(0);

// Create an effect that depends on the signal
effect(() => {
  console.log(`Count is now: ${count.value}`);
});
// Logs: "Count is now: 0"

// Update the signal value
count.value = 1;
// Logs: "Count is now: 1"

// Read the value without tracking dependencies
const currentValue = count.peek();
// Returns 1, but doesn't create a dependency
```

## Usage Notes

- When you read a signal's `value` property inside an effect or computed value, a dependency is automatically tracked.
- When you update a signal's `value` property, all dependent effects and computed values will automatically update.
- Use `peek()` when you want to read a signal's value without creating a dependency.

## TypeScript Example

```typescript
import { signal } from 'linen';

// With type inference
const count = signal(0); // Signal<number>

// With explicit type
const name = signal<string | null>(null); // Signal<string | null>

// With complex types
interface User {
  id: number;
  name: string;
  email: string;
}

const user = signal<User>({
  id: 1,
  name: 'John Doe',
  email: 'john@example.com'
});
```

## Implementation Details

Internally, signals are implemented using JavaScript getters and setters. When a signal's value is read, it registers the current effect as a dependency. When a signal's value is updated, it notifies all dependent effects to re-run.
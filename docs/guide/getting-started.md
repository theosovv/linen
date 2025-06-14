# Getting Started

This guide will help you get started with Linen's reactivity system.

## Installation

You can install Linen using your preferred package manager:

```bash
# Using npm
npm install linen

# Using yarn
yarn add linen

# Using pnpm
pnpm add linen
```

## Basic Usage

Here's a simple example of using Linen's reactivity system:

```typescript
import { signal, computed, effect } from 'linen';

// Create a signal with an initial value
const count = signal(0);

// Create a computed value that depends on the signal
const doubled = computed(() => count.value * 2);

// Create an effect that runs when dependencies change
effect(() => {
  console.log(`Count: ${count.value}, Doubled: ${doubled.value}`);
});
// Logs: "Count: 0, Doubled: 0"

// Update the signal value
count.value = 5;
// Logs: "Count: 5, Doubled: 10"
```

## Core Concepts

### Signals

Signals are the basic building blocks of Linen's reactivity system. They store values that can change over time.

```typescript
import { signal } from 'linen';

const count = signal(0);
console.log(count.value); // 0

count.value = 1;
console.log(count.value); // 1
```

### Computed Values

Computed values are derived from signals or other computed values. They automatically update when their dependencies change.

```typescript
import { signal, computed } from 'linen';

const count = signal(0);
const doubled = computed(() => count.value * 2);

console.log(doubled.value); // 0

count.value = 5;
console.log(doubled.value); // 10
```

### Effects

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

## Next Steps

Now that you understand the basics of Linen's reactivity system, check out the [API Reference](/api/) for more details on each function.
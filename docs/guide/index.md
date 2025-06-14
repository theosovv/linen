# What is Linen?

Linen is a modern JavaScript framework designed for building reactive web applications. It provides a simple yet powerful reactivity system based on signals, computed values, and effects.

## Key Features

### Fine-grained Reactivity

Linen uses a signal-based reactivity system that allows for fine-grained updates. Unlike frameworks that update entire components, Linen tracks dependencies at a granular level, ensuring that only the parts of your application that need to update will update.

### Simple and Intuitive API

Linen provides a simple and intuitive API for creating reactive state:

- `signal()` - Creates a reactive value that can be read and written to
- `computed()` - Creates a derived value that automatically updates when dependencies change
- `effect()` - Runs side effects when dependencies change
- `untrack()` - Prevents dependency tracking within a function
- `events` - Event subscription and notification system
- `batch()` - Batches multiple updates together

### TypeScript-first

Linen is built with TypeScript from the ground up, providing excellent type safety and inference.

## Getting Started

Check out the [Getting Started](/guide/getting-started) guide to learn how to use Linen in your project.
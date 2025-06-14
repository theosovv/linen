# API Reference

This section provides detailed documentation for Linen's API.

## Core APIs

Linen's core API consists of several key functions that enable reactive programming:

- [signal()](/api/signal) - Creates a reactive signal
- [computed()](/api/computed) - Creates a computed value that depends on signals
- [effect()](/api/effect) - Creates an effect that runs when dependencies change
- [untrack()](/api/untrack) - Prevents dependency tracking
- [batch()](/api/batch) - Batches multiple updates together

## Importing

You can import all core APIs directly from the main package:

```typescript
import { signal, computed, effect, untrack, batch } from 'linen';
```

## TypeScript Support

Linen is built with TypeScript and provides full type definitions for all APIs.
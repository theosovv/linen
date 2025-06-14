# Resource Disposal and Cleanup

Resource disposal and cleanup are essential for preventing memory leaks and ensuring that resources are properly released when they are no longer needed. Linen provides several utilities to help manage resource lifecycles.

## Basic Cleanup with `onCleanup`

The `onCleanup` function registers a cleanup function that will be called when the current effect is re-executed or stopped:

```typescript
import { signal, effect, onCleanup } from 'linen';

const count = signal(0);

const stop = effect(() => {
  console.log(`Count: ${count.value}`);
  
  onCleanup(() => {
    console.log(`Cleaning up for count: ${count.value}`);
  });
});

// When count changes, the cleanup function will be called before the effect runs again
count.value = 1;

// When the effect is stopped, the cleanup function will be called
stop();
```

## Returning Cleanup Function from Effect

You can also return a cleanup function directly from an effect:

```typescript
import { signal, effect } from 'linen';

const message = signal('Hello');

const stop = effect(() => {
  console.log(`Message: ${message.value}`);
  
  // Return a cleanup function
  return () => {
    console.log(`Cleaning up message: ${message.value}`);
  };
});

message.value = 'World';
stop();
```

## Resource Utilities

Linen provides several utilities for common resource management patterns:

### `createResource`

Creates a resource with automatic cleanup:

```typescript
import { createResource } from 'linen';

const connection = createResource(() => {
  // Setup the resource
  const conn = createDatabaseConnection();
  
  // Return the resource and its cleanup function
  return [conn, () => conn.close()];
});

// Use the connection
connection.query('SELECT * FROM users');

// The connection will be automatically closed when the effect is cleaned up
```

### `createInterval`

Creates an interval that is automatically cleared:

```typescript
import { createInterval } from 'linen';

const clear = createInterval(() => {
  console.log('Interval tick');
}, 1000);

// Later, to manually clear the interval
clear();
```

### `createTimeout`

Creates a timeout that is automatically cleared:

```typescript
import { createTimeout } from 'linen';

const clear = createTimeout(() => {
  console.log('Timeout fired');
}, 2000);

// Optionally cancel the timeout before it fires
clear();
```

### `createEventListener`

Adds an event listener that is automatically removed:

```typescript
import { createEventListener } from 'linen';

const remove = createEventListener(document, 'click', (event) => {
  console.log('Document clicked', event);
});

// Later, to manually remove the event listener
remove();
```

### `createEffect`

Creates a reactive effect with automatic cleanup:

```typescript
import { signal, createEffect } from 'linen';

const count = signal(0);

const stop = createEffect(() => {
  console.log(`Count: ${count.value}`);
  
  // You can return a cleanup function
  return () => {
    console.log('Effect cleaned up');
  };
});

// Later, to stop the effect
stop();
```

## Disposable Scopes

For more complex scenarios, you can create disposable scopes that manage multiple resources:

### `createScope`

Creates a disposable scope that automatically cleans up all resources when disposed:

```typescript
import { createScope, onCleanup, createInterval } from 'linen';

const dispose = createScope(() => {
  // Set up resources
  const interval = createInterval(() => {
    console.log('Tick');
  }, 1000);
  
  // Register custom cleanup
  onCleanup(() => {
    console.log('Custom cleanup');
  });
  
  // All resources and cleanup functions will be handled automatically
});

// Later, to dispose all resources in the scope
dispose();
```

### Nested Scopes

Scopes can be nested, and each scope manages its own resources:

```typescript
import { createScope, onCleanup } from 'linen';

const outerDispose = createScope(() => {
  console.log('Setting up outer scope');
  
  onCleanup(() => {
    console.log('Cleaning up outer scope');
  });
  
  const innerDispose = createScope(() => {
    console.log('Setting up inner scope');
    
    onCleanup(() => {
      console.log('Cleaning up inner scope');
    });
  });
  
  // Dispose inner scope separately if needed
  // innerDispose();
});

// Disposing the outer scope will also dispose any nested scopes that haven't been disposed yet
outerDispose();
```

## Error Handling

Cleanup functions should be robust and handle errors gracefully. If a cleanup function throws an error, it will be caught and logged, but it won't prevent other cleanup functions from running:

```typescript
import { effect, onCleanup } from 'linen';

effect(() => {
  onCleanup(() => {
    // This error will be caught and logged
    throw new Error('Cleanup error');
  });
  
  onCleanup(() => {
    // This will still run even if the previous cleanup function throws
    console.log('Second cleanup');
  });
});
```

## Best Practices

1. **Always clean up resources**: Any resource that needs to be released (event listeners, timers, connections) should have a cleanup function.

2. **Keep cleanup functions simple**: Cleanup functions should focus on releasing resources and avoid complex logic.

3. **Use the appropriate utility**: Choose the right utility for your use case to minimize boilerplate code.

4. **Handle errors gracefully**: Make sure your cleanup functions handle errors properly to avoid affecting other cleanup operations.

5. **Dispose scopes when done**: Always dispose scopes when they're no longer needed to prevent memory leaks.
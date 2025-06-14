# Event Subscription and Notification System

Linen provides a powerful event subscription and notification system that integrates with the reactive signal system. This system allows components to subscribe to events and receive notifications when they occur.

## PubSub

The main class for working with events.

```typescript
import { createPubSub, globalPubSub } from 'linen';

// Using the global instance
globalPubSub.subscribe('event', (data) => {
  console.log(data);
});

// Creating a local instance
const localPubSub = createPubSub();
```

### PubSub Methods
#### subscribe

Subscribes to an event and returns a function to unsubscribe.

```typescript
const unsubscribe = pubSub.subscribe('eventName', (data) => {
  console.log(data);
});

// Later, when the subscription is no longer needed
unsubscribe();
```

#### publish

Publishes an event with data.

```typescript
pubSub.publish('eventName', { message: 'Hello, world!' });
```

#### hasSubscribers

Checks if there are subscribers to an event.

```typescript
if (pubSub.hasSubscribers('eventName')) {
  console.log('The event has subscribers');
}
```

#### subscriberCount

Returns the number of subscribers to an event.


```typescript
const count = pubSub.subscriberCount('eventName');
console.log(`The event has ${count} subscribers`);
```

#### clearEvent

Unsubscribes all subscribers from an event.

```typescript
pubSub.clearEvent('eventName');
```

#### clearAllEvents

Unsubscribes all subscribers from all events.

```typescript
pubSub.clearAllEvents();
```

### Integration with Signals

#### eventSignal

Creates a signal that updates when an event occurs.

```typescript
import { eventSignal } from 'linen';

// Create a signal linked to the 'userChanged' event
const currentUser = eventSignal('userChanged', null);

// Now when the 'userChanged' event is published, the signal will update
pubSub.publish('userChanged', { id: 1, name: 'John' });

// You can use the signal as usual
console.log(currentUser.value); // { id: 1, name: 'John' }
```

#### eventHistory

Creates a signal that accumulates event history.

```typescript
import { eventHistory } from 'linen';

// Create a signal with event history (maximum 5 events)
const notifications = eventHistory('notification', 5);

// When events are published, they are added to the history
pubSub.publish('notification', { id: 1, message: 'First notification' });
pubSub.publish('notification', { id: 2, message: 'Second notification' });

// You can get the entire history
console.log(notifications.value); // [{ id: 1, ... }, { id: 2, ... }]
```

#### eventEffect

Creates an effect that runs when an event occurs.

```typescript
import { eventEffect } from 'linen';

// Create an effect that runs on the 'serverStatus' event
const stopEffect = eventEffect('serverStatus', (status) => {
  if (status.online) {
    console.log(`Server online. Latency: ${status.latency}ms`);
  } else {
    console.log('Server offline');
  }
});

// Later, when the effect is no longer needed
stopEffect();
```

#### publishEffect

Creates an effect that publishes an event when dependencies change.

```typescript
import { signal, publishEffect } from 'linen';

// Create signals
const temperature = signal(20);
const humidity = signal(50);

// Create an effect that publishes an event when signals change
const stopPublishEffect = publishEffect('weatherChanged', () => {
  return {
    temperature: temperature.value,
    humidity: humidity.value,
    timestamp: new Date().toISOString()
  };
});

// Changing signals will trigger event publication
temperature.value = 22;

// Later, when the effect is no longer needed
stopPublishEffect();
```

### Typed Events

For improved type safety, you can use typed events.

```typescript
import { createPubSub } from 'linen';

interface User {
  name: string;
  id: number;
}

const pubSub = createPubSub<User>();

const unsubscribe = pubSub.subscribe('userLoggedIn', user => {
  console.log(`User is logged in: ${user.name}`);
});

pubSub.publish('userLoggedIn', { id: 1, name: 'John' });

unsubscribe();

// Compilation error with incorrect types
// typedPubSub.publish('user:login', { id: 1 }); // Error: missing 'name' property

```

### Usage Examples
#### Notifications

```typescript
import { eventHistory, eventEffect } from 'linen';

// Create notification history
const notifications = eventHistory('notification', 10);

// Subscribe to new notifications
eventEffect('notification', (notification) => {
  // Show notification to the user
  showToast(notification.message, notification.type);
});

// Publish a notification
globalPubSub.publish('notification', {
  id: Date.now(),
  message: 'Operation completed successfully',
  type: 'success'
});

```
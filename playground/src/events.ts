import {
  eventSignal,
  eventHistory,
  eventEffect,
  publishEffect,
  signal,
  computed,
  effect,
  createPubSub,
} from '../../src';

interface User {
  name: string;
  id: number;
}

interface Notification {
  message: string;
  id: number;
}

interface Status {
  online: boolean;
  latency: number;
}

interface Weather {
  temperature: number;
  humidity: number;
  timestamp: string;
}

export function eventsPlayground() {
  console.log('--- Example 1: Basic usage PubSub ---');

  const pubSub = createPubSub<User>();

  const unsubscribe = pubSub.subscribe('userLoggedIn', user => {
    console.log(`User is logged in: ${user.name}`);
  });

  pubSub.publish('userLoggedIn', { id: 1, name: 'John' });

  unsubscribe();

  pubSub.publish('userLoggedIn', { id: 2, name: 'Jane' });
  console.log('Second publish should not be logged because we unsubscribed');

  console.log('\n--- Example 2: Usage eventSignal ---');

  const userPubSub = createPubSub<User | null>();

  const currentUser = eventSignal<User | null>('user', null, userPubSub);

  const isLoggedIn = computed(() => currentUser.value !== null);

  effect(() => {
    if (isLoggedIn.value) {
      console.log(`User is logged in: ${currentUser.value?.name}`);
    } else {
      console.log('User is not logged in');
    }
  });

  console.log('Initial state:', currentUser.value);

  userPubSub.publish('user', { id: 1, name: 'John' });

  userPubSub.publish('user', null);

  console.log('\n--- Example 3: Usage eventHistory ---');

  const notificationPubSub = createPubSub<Notification>();

  const notificationHistory = eventHistory<Notification>(
    'notification',
    3,
    notificationPubSub
  );

  effect(() => {
    console.log('Current notifications:');
    notificationHistory.value.forEach((notification, index) => {
      console.log(`${index + 1}. ${notification.message}`);
    });
  });

  notificationPubSub.publish('notification', {
    id: 1,
    message: 'First notification',
  });

  notificationPubSub.publish('notification', {
    id: 2,
    message: 'Second notification',
  });

  notificationPubSub.publish('notification', {
    id: 3,
    message: 'Third notification',
  });

  notificationPubSub.publish('notification', {
    id: 4,
    message: 'Fourth notification',
  });

  console.log('\n--- Example 4: Usage eventEffect ---');

  const serverStatusPubSub = createPubSub<Status>();

  const stopEffect = eventEffect<Status>(
    'serverStatus',
    status => {
      if (status.online) {
        console.log(`Server is online. Latency: ${status.latency}ms`);
      } else {
        console.log('Server is offline');
      }
    },
    serverStatusPubSub
  );

  serverStatusPubSub.publish('serverStatus', {
    online: true,
    latency: 100,
  });
  serverStatusPubSub.publish('serverStatus', {
    online: false,
    latency: 0,
  });
  serverStatusPubSub.publish('serverStatus', {
    online: true,
    latency: 150,
  });

  stopEffect();

  serverStatusPubSub.publish('serverStatus', {
    online: true,
    latency: 100,
  });
  console.log(
    'Last server status update should not be logged because we stopped the effect'
  );

  console.log('\n--- Example 5: Usage publishEffect ---');

  const weatherPubSub = createPubSub<Weather>();

  const temperature = signal(20);
  const humidity = signal(50);

  weatherPubSub.subscribe('weatherChanged', data => {
    console.log(
      `Weather changed: ${data.temperature}°C, ${data.humidity}% humidity at ${data.timestamp}`
    );
  });

  const stopPublishEffect = publishEffect<Weather>(
    'weatherChanged',
    () => {
      return {
        temperature: temperature.value,
        humidity: humidity.value,
        timestamp: new Date().toISOString(),
      };
    },
    weatherPubSub
  );

  console.log('Changing temperature to 22°C');
  temperature.value = 22;

  console.log('Changing humidity to 55%');
  humidity.value = 55;

  stopPublishEffect();

  console.log(
    'Changing temperature to 25°C (should not trigger event because we stopped the effect)'
  );
  temperature.value = 25;

  console.log('\n--- End of examples ---');
}

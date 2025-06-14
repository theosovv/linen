import {
  createResource,
  createInterval,
  createTimeout,
  createScope,
  signal,
  effect,
  onCleanup,
} from '../../src';

export function disposalPlayground() {
  console.log('--- Example 1: Basic cleanup with onCleanup ---');

  const count = signal(0);

  const stop = effect(() => {
    console.log(`Count: ${count.value}`);

    onCleanup(() => {
      console.log(`Cleaning up for count: ${count.value}`);
    });
  });

  console.log('Updating count to 1');
  count.value = 1;

  console.log('Updating count to 2');
  count.value = 2;

  console.log('Stopping effect');
  stop();

  console.log('\n--- Example 2: Returning cleanup function from effect ---');

  const message = signal('Hello');

  const stopMessage = effect(() => {
    console.log(`Message: ${message.value}`);

    return () => {
      console.log(`Cleaning up message: ${message.value}`);
    };
  });

  console.log('Updating message');
  message.value = 'World';

  console.log('Stopping message effect');
  stopMessage();

  console.log('\n--- Example 3: Using createInterval ---');

  let counter = 0;
  const clearInterval = createInterval(() => {
    counter++;
    console.log(`Interval counter: ${counter}`);

    if (counter >= 3) {
      console.log('Clearing interval');
      clearInterval();
    }
  }, 1000);

  console.log('\n--- Example 4: Using createTimeout ---');

  const _ = createTimeout(() => {
    console.log('Timeout fired!');
  }, 2000);

  console.log('\n--- Example 5: Using createResource ---');

  const resource = createResource(() => {
    console.log('Setting up resource');

    const data = { id: 1, name: 'Resource' };

    return [
      data,
      () => {
        console.log('Cleaning up resource');
      },
    ];
  });

  console.log(`Resource: ${JSON.stringify(resource)}`);

  console.log('\n--- Example 6: Using createScope ---');

  const dispose = createScope(() => {
    console.log('Setting up scope');

    createInterval(() => {
      console.log('Scope interval tick');
    }, 1000);

    const _ = createTimeout(() => {
      console.log('Scope timeout fired');
    }, 1500);

    onCleanup(() => {
      console.log('Custom scope cleanup');
    });
  });

  console.log('Waiting for some time...');
  setTimeout(() => {
    console.log('Disposing scope');
    dispose();

    console.log('\n--- Example 7: Nested scopes ---');

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

      setTimeout(() => {
        console.log('Disposing inner scope');
        innerDispose();
      }, 1000);
    });

    setTimeout(() => {
      console.log('Disposing outer scope');
      outerDispose();

      console.log('\n--- End of examples ---');
    }, 2000);
  }, 3000);
}

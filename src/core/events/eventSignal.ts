import { onCleanup, signal } from '../signals';
import { globalPubSub } from './pubsub';
import { TypedPubSub } from './types';

/**
 * Creates a signal that is updated when an event occurs.
 * @param event Event name
 * @param initialValue Initial value
 * @param pubSub PubSub instance (global by default)
 * @returns Event signal
 */
export function eventSignal<T>(
  event: string,
  initialValue: T,
  pubSub: TypedPubSub<T> = globalPubSub as unknown as TypedPubSub<T>
) {
  const eventValue = signal<T>(initialValue);

  const unsubscribe = pubSub.subscribe(event, (data: T) => {
    eventValue.value = data as T;
  });

  onCleanup(unsubscribe);

  return eventValue;
}

/**
 * Creates a signal that is updated with the history of an event.
 * @param event Event name
 * @param maxHistory Maximum history length
 * @param pubSub PubSub instance (global by default)
 * @returns Event history signal
 */
export function eventHistory<T>(
  event: string,
  maxHistory: number = 0,
  pubSub: TypedPubSub<T> = globalPubSub as unknown as TypedPubSub<T>
) {
  const history = signal<T[]>([]);

  const unsubscribe = pubSub.subscribe(event, (data: T) => {
    const newHistory = [...history.value, data];

    if (maxHistory > 0 && newHistory.length > maxHistory) {
      history.value = newHistory.slice(newHistory.length - maxHistory);
    } else {
      history.value = newHistory;
    }
  });

  onCleanup(unsubscribe);

  return history;
}

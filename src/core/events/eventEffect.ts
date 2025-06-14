import { effect } from '../signals';
import { globalPubSub } from './pubsub';
import { TypedPubSub } from './types';

/**
 * Creates an effect that is executed when an event occurs.
 * @param event Event name
 * @param callback Callback function
 * @param pubSub PubSub instance (global by default)
 * @returns Cleanup function
 */
export function eventEffect<T>(
  event: string,
  callback: (data: T) => void,
  pubSub: TypedPubSub<T> = globalPubSub as unknown as TypedPubSub<T>
): () => void {
  return pubSub.subscribe(event, callback);
}

export function publishEffect<T>(
  event: string,
  producer: () => T,
  pubSub: TypedPubSub<T> = globalPubSub as unknown as TypedPubSub<T>
): () => void {
  return effect(() => {
    const data = producer();

    pubSub.publish(event, data);
  });
}

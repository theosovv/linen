import { TypedPubSub } from './types';

/**
 * PubSub - Publish-Subscribe pattern implementation.
 */
export class PubSub<T> implements TypedPubSub<T> {
  private subscribers = new Map<string, Set<(data: T) => void>>();

  /**
   * Subscribes to an event.
   * @param event Event name
   * @param callback Callback function
   * @returns Unsubscribe function
   */
  subscribe(event: string, callback: (data: T) => void): () => void {
    if (!this.subscribers.has(event)) {
      this.subscribers.set(event, new Set());
    }

    const callbacks = this.subscribers.get(event)!;

    callbacks.add(callback);

    return () => {
      callbacks.delete(callback);

      if (callbacks.size === 0) {
        this.subscribers.delete(event);
      }
    };
  }

  /**
   * Publishes an event.
   * @param event Event name
   * @param data Data to publish
   */
  publish(event: string, data: T): void {
    const callbacks = this.subscribers.get(event);

    if (!callbacks) return;

    callbacks.forEach(callback => {
      try {
        callback(data);
      } catch (error) {
        console.error(`Error in event "${String(event)}":`, error);
      }
    });
  }

  /**
   * Checks if there are subscribers for an event.
   * @param event Event name
   * @returns True if there are subscribers, false otherwise
   */
  hasSubscribers(event: string): boolean {
    return this.subscribers.has(event) && this.subscribers.get(event)!.size > 0;
  }

  /**
   * Gets the number of subscribers for an event.
   * @param event Event name
   * @returns Number of subscribers
   */
  subscriberCount(event: string): number {
    return this.subscribers.has(event) ? this.subscribers.get(event)!.size : 0;
  }

  /**
   * Clears all subscribers for an event.
   * @param event Event name
   */
  clearEvent(event: string): void {
    this.subscribers.delete(event);
  }

  /**
   * Clears all subscribers.
   */
  clearAll(): void {
    this.subscribers.clear();
  }
}

export const globalPubSub = new PubSub<TypedPubSub<unknown>>();

export function createPubSub<T>(): TypedPubSub<T> {
  return new PubSub<T>();
}

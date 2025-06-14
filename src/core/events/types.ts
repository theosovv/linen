
/**
 * Interface for typed pubsub
 */
export interface TypedPubSub<T> {
  subscribe(event: string, callback: (data: T) => void): () => void;
  publish(event: string, data: T): void;
  hasSubscribers(event: string): boolean;
  subscriberCount(event: string): number;
  clearEvent(event: string): void;
  clearAll(): void;
}

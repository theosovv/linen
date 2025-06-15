import { PubSub } from '../../src/core/events';
import { describe, it, expect, vi } from 'vitest';

describe('PubSub', () => {
  it('should allow subscribing to events', () => {
    const pubsub = new PubSub();
    const callback = vi.fn();

    pubsub.subscribe('test', callback);
    pubsub.publish('test', { test: 'hello' });

    expect(callback).toHaveBeenCalledWith({ test: 'hello' });
  });

  it('should allow unsubscribing from events', () => {
    const pubsub = new PubSub();
    const callback = vi.fn();

    const unsubscribe = pubsub.subscribe('test', callback);
    unsubscribe();

    pubsub.publish('test', { test: 'hello' });

    expect(callback).not.toHaveBeenCalled();
  });

  it('should handle multiple subscribers', () => {
    const pubsub = new PubSub();
    const callback1 = vi.fn();
    const callback2 = vi.fn();

    pubsub.subscribe('test', callback1);
    pubsub.subscribe('test', callback2);

    pubsub.publish('test', { test: 'hello' });

    expect(callback1).toHaveBeenCalledWith({ test: 'hello' });
    expect(callback2).toHaveBeenCalledWith({ test: 'hello' });
  });

  it('should correctly count subscribers', () => {
    const pubsub = new PubSub();

    expect(pubsub.subscriberCount('test')).toBe(0);

    const unsubscribe1 = pubsub.subscribe('test', () => {});
    expect(pubsub.subscriberCount('test')).toBe(1);

    const unsubscribe2 = pubsub.subscribe('test', () => {});
    expect(pubsub.subscriberCount('test')).toBe(2);

    unsubscribe1();
    expect(pubsub.subscriberCount('test')).toBe(1);

    unsubscribe2();
    expect(pubsub.subscriberCount('test')).toBe(0);
  });

  it('should clear events', () => {
    const pubsub = new PubSub();

    pubsub.subscribe('test1', () => {});
    pubsub.subscribe('test2', () => {});

    expect(pubsub.hasSubscribers('test1')).toBe(true);
    expect(pubsub.hasSubscribers('test2')).toBe(true);

    pubsub.clearEvent('test1');

    expect(pubsub.hasSubscribers('test1')).toBe(false);
    expect(pubsub.hasSubscribers('test2')).toBe(true);

    pubsub.clearAll();

    expect(pubsub.hasSubscribers('test2')).toBe(false);
  });

  it('should handle errors in callbacks', () => {
    const pubsub = new PubSub();
    const errorCallback = vi.fn().mockImplementation(() => {
      throw new Error('Test error');
    });
    const normalCallback = vi.fn();

    // Мокаем console.error, чтобы не засорять вывод теста
    const originalConsoleError = console.error;
    console.error = vi.fn();

    pubsub.subscribe('test', errorCallback);
    pubsub.subscribe('test', normalCallback);

    pubsub.publish('test', 'hello');

    expect(errorCallback).toHaveBeenCalledWith('hello');
    expect(normalCallback).toHaveBeenCalledWith('hello');
    expect(console.error).toHaveBeenCalled();

    // Восстанавливаем console.error
    console.error = originalConsoleError;
  });
});

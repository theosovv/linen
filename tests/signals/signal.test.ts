import { describe, test, expect, vi } from 'vitest';
import { signal } from '../../src';

describe('signal', () => {
  test('should create a signal with initial value', () => {
    const count = signal(0);
    expect(count.value).toBe(0);
  });

  test('should update signal value', () => {
    const count = signal(0);
    count.value = 5;
    expect(count.value).toBe(5);
  });

  test('should not trigger update if value is the same', () => {
    const count = signal(0);
    const mockFn = vi.fn();

    const effect = () => {
      mockFn(count.value);
    };

    effect();

    count.value = 0;

    expect(mockFn).toHaveBeenCalledTimes(1);
  });

  test('peek should return current value without tracking', () => {
    const count = signal(0);
    const value = count.peek();
    expect(value).toBe(0);

    count.value = 5;
    expect(count.peek()).toBe(5);
  });
});

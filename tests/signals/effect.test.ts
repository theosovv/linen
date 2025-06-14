import { describe, test, expect, vi } from 'vitest';
import { signal, effect } from '../../src';

describe('effect', () => {
  test('should run effect immediately', () => {
    const mockFn = vi.fn();
    effect(mockFn);

    expect(mockFn).toHaveBeenCalledTimes(1);
  });

  test('should run effect when dependencies change', () => {
    const count = signal(0);
    const mockFn = vi.fn();

    effect(() => {
      mockFn(count.value);
    });

    expect(mockFn).toHaveBeenCalledWith(0);
    expect(mockFn).toHaveBeenCalledTimes(1);

    count.value = 5;
    expect(mockFn).toHaveBeenCalledWith(5);
    expect(mockFn).toHaveBeenCalledTimes(2);
  });

  test('should handle multiple dependencies', () => {
    const a = signal(1);
    const b = signal(2);
    const mockFn = vi.fn();

    effect(() => {
      mockFn(a.value, b.value);
    });

    expect(mockFn).toHaveBeenCalledWith(1, 2);
    expect(mockFn).toHaveBeenCalledTimes(1);

    a.value = 3;
    expect(mockFn).toHaveBeenCalledWith(3, 2);
    expect(mockFn).toHaveBeenCalledTimes(2);

    b.value = 4;
    expect(mockFn).toHaveBeenCalledWith(3, 4);
    expect(mockFn).toHaveBeenCalledTimes(3);
  });

  test('should cleanup previous dependencies', () => {
    const condition = signal(true);
    const a = signal(1);
    const b = signal(2);
    const mockFn = vi.fn();

    effect(() => {
      mockFn(condition.value ? a.value : b.value);
    });

    expect(mockFn).toHaveBeenCalledWith(1);
    expect(mockFn).toHaveBeenCalledTimes(1);

    a.value = 3;
    expect(mockFn).toHaveBeenCalledWith(3);
    expect(mockFn).toHaveBeenCalledTimes(2);

    b.value = 4;
    expect(mockFn).toHaveBeenCalledTimes(2);

    condition.value = false;
    expect(mockFn).toHaveBeenCalledWith(4);
    expect(mockFn).toHaveBeenCalledTimes(3);

    b.value = 5;
    expect(mockFn).toHaveBeenCalledWith(5);
    expect(mockFn).toHaveBeenCalledTimes(4);

    a.value = 6;
    expect(mockFn).toHaveBeenCalledTimes(4);
  });

  test('should stop tracking when returned function is called', () => {
    const count = signal(0);
    const mockFn = vi.fn();

    const stop = effect(() => {
      mockFn(count.value);
    });

    expect(mockFn).toHaveBeenCalledTimes(1);

    count.value = 5;
    expect(mockFn).toHaveBeenCalledTimes(2);

    stop();

    count.value = 10;
    expect(mockFn).toHaveBeenCalledTimes(2);
  });
});

import { describe, test, expect, vi } from 'vitest';
import { signal, effect, untrack } from '../../src';

describe('untrack', () => {
  test('should not track dependencies inside untrack', () => {
    const count = signal(0);
    const mockFn = vi.fn();

    effect(() => {
      mockFn(untrack(() => count.value));
    });

    expect(mockFn).toHaveBeenCalledWith(0);
    expect(mockFn).toHaveBeenCalledTimes(1);

    count.value = 5;
    expect(mockFn).toHaveBeenCalledTimes(1);
  });
});

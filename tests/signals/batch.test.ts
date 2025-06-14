import { describe, it, expect, vi } from 'vitest';
import { signal, effect, batch, transaction } from '../../src';

describe('batch', () => {
  it('should batch multiple signal updates', () => {
    const count = signal(0);
    const double = signal(0);

    const effectFn = vi.fn(() => {
      double.value = count.value * 2;
    });

    effect(effectFn);

    effectFn.mockClear();

    count.value = 1;
    count.value = 2;
    count.value = 3;

    expect(effectFn).toHaveBeenCalledTimes(3);
    expect(double.value).toBe(6);

    effectFn.mockClear();

    batch(() => {
      count.value = 4;
      count.value = 5;
      count.value = 6;
    });

    expect(effectFn).toHaveBeenCalledTimes(1);
    expect(double.value).toBe(12);
  });

  it('should support nested batches', () => {
    const count = signal(0);
    const effectFn = vi.fn();

    effect(() => {
      effectFn(count.value);
    });

    effectFn.mockClear();

    batch(() => {
      count.value = 1;

      batch(() => {
        count.value = 2;
        count.value = 3;
      });

      count.value = 4;
    });

    expect(effectFn).toHaveBeenCalledTimes(1);
    expect(effectFn).toHaveBeenCalledWith(4);
  });

  it('should handle errors in batched functions', () => {
    const count = signal(0);
    const effectFn = vi.fn();

    effect(() => {
      effectFn(count.value);
    });

    effectFn.mockClear();

    expect(() => {
      batch(() => {
        count.value = 1;
        throw new Error('Test error');
      });
    }).toThrow('Test error');

    expect(effectFn).toHaveBeenCalledTimes(1);
    expect(effectFn).toHaveBeenCalledWith(1);
  });

  it('should work with transaction alias', () => {
    const count = signal(0);
    const effectFn = vi.fn();

    effect(() => {
      effectFn(count.value);
    });

    effectFn.mockClear();

    transaction(() => {
      count.value = 1;
      count.value = 2;
      count.value = 3;
    });

    expect(effectFn).toHaveBeenCalledTimes(1);
    expect(effectFn).toHaveBeenCalledWith(3);
  });

  it('should return the result of the batched function', () => {
    const count = signal(0);

    const result = batch(() => {
      count.value = 1;
      return 'result';
    });

    expect(result).toBe('result');
  });
});

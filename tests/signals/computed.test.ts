import { describe, test, expect, vi } from 'vitest';
import { signal, computed } from '../../src';

describe('computed', () => {
  test('should compute value based on signals', () => {
    const count = signal(0);
    const doubled = computed(() => count.value * 2);

    expect(doubled.value).toBe(0);

    count.value = 5;
    expect(doubled.value).toBe(10);
  });

  test('should recompute only when dependencies change', () => {
    const count = signal(0);
    const multiplier = signal(2);

    const computeFn = vi.fn(() => count.value * multiplier.value);

    const result = computed(computeFn);

    expect(computeFn).toHaveBeenCalledTimes(1);

    expect(result.value).toBe(0);

    computeFn.mockClear();
    expect(result.value).toBe(0);
    expect(computeFn).toHaveBeenCalledTimes(0);

    computeFn.mockClear();
    count.value = 5;
    expect(computeFn).toHaveBeenCalledTimes(1);
    expect(result.value).toBe(10);

    computeFn.mockClear();
    multiplier.value = 3;
    expect(computeFn).toHaveBeenCalledTimes(1);
    expect(result.value).toBe(15);
  });

  test('computed should handle nested dependencies', () => {
    const a = signal(1);
    const b = signal(2);
    const c = computed(() => a.value + b.value);
    const d = computed(() => c.value * 2);

    expect(d.value).toBe(6); // (1 + 2) * 2 = 6

    a.value = 3;
    expect(d.value).toBe(10); // (3 + 2) * 2 = 10

    b.value = 4;
    expect(d.value).toBe(14); // (3 + 4) * 2 = 14
  });

  test('peek should return current value without tracking', () => {
    const count = signal(0);
    const doubled = computed(() => count.value * 2);

    expect(doubled.peek()).toBe(0);

    count.value = 5;
    expect(doubled.peek()).toBe(10);
  });
});

import { describe, test, expect, vi } from 'vitest';
import { signal, computed, effect } from '../../src';

describe('integration', () => {
  test('should handle complex reactive scenarios', () => {
    const firstName = signal('John');
    const lastName = signal('Doe');
    const showFullName = signal(true);

    const fullName = computed(() => {
      return showFullName.value
        ? `${firstName.value} ${lastName.value}`
        : firstName.value;
    });

    const greeting = computed(() => `Hello, ${fullName.value}!`);

    const mockFn = vi.fn();

    effect(() => {
      mockFn(greeting.value);
    });

    expect(mockFn).toHaveBeenCalledWith('Hello, John Doe!');

    firstName.value = 'Jane';
    expect(mockFn).toHaveBeenCalledWith('Hello, Jane Doe!');

    lastName.value = 'Smith';
    expect(mockFn).toHaveBeenCalledWith('Hello, Jane Smith!');

    showFullName.value = false;
    expect(mockFn).toHaveBeenCalledWith('Hello, Jane!');

    lastName.value = 'Johnson';
    expect(mockFn).toHaveBeenCalledTimes(4);

    showFullName.value = true;
    expect(mockFn).toHaveBeenCalledWith('Hello, Jane Johnson!');
  });

  test('should handle circular dependencies gracefully', () => {
    const a = signal(1);
    const b = computed(() => a.value + 1);

    effect(() => {
      if (b.value > 5) {
        a.value = 1;
      }
    });

    expect(a.value).toBe(1);
    expect(b.value).toBe(2);

    a.value = 2;
    expect(a.value).toBe(2);
    expect(b.value).toBe(3);

    a.value = 3;
    expect(a.value).toBe(3);
    expect(b.value).toBe(4);

    a.value = 4;
    expect(a.value).toBe(4);
    expect(b.value).toBe(5);

    a.value = 5;

    expect(a.value).toBe(1);
    expect(b.value).toBe(2);
  });
});

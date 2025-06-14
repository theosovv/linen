import { effect, signal } from './signal';

type ComputedFn<T> = () => T;

/**
 * Creates a computed value that automatically updates when dependencies change
 * @param fn Function to calculate value
 * @returns Read-only Signal
 */
export function computed<T>(fn: ComputedFn<T>) {
  const result = signal<T>(undefined as T);
  let initialized = false;
  let value: T;

  effect(() => {
    const newValue = fn();

    if (!initialized || !Object.is(newValue, value)) {
      value = newValue;
      result.value = newValue;
      initialized = true;
    }
  });

  return {
    get value() {
      return result.value;
    },
    peek(): T {
      return result.peek();
    },
  };
}

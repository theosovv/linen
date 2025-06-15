import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { signal, effect } from '../../src/';
import { createScope, onCleanup } from '../../src/core/signals/signal';
import {
  createInterval,
  createTimeout,
  createEventListener,
  createEffect,
  createResource,
} from '../../src/core/signals/resource';

describe('Disposal and Cleanup', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('should run cleanup function when effect is stopped', () => {
    const count = signal(0);
    const cleanup = vi.fn();

    const stop = effect(() => {
      count.value;
      onCleanup(cleanup);
    });

    expect(cleanup).not.toHaveBeenCalled();

    stop();

    expect(cleanup).toHaveBeenCalledTimes(1);
  });

  it('should run cleanup function when dependencies change', () => {
    const count = signal(0);
    const cleanup1 = vi.fn();
    const cleanup2 = vi.fn();

    effect(() => {
      if (count.value === 0) {
        onCleanup(cleanup1);
      } else {
        onCleanup(cleanup2);
      }
    });

    expect(cleanup1).not.toHaveBeenCalled();
    expect(cleanup2).not.toHaveBeenCalled();

    count.value = 1;

    expect(cleanup1).toHaveBeenCalledTimes(1);
    expect(cleanup2).not.toHaveBeenCalled();

    count.value = 2;

    expect(cleanup1).toHaveBeenCalledTimes(1);
    expect(cleanup2).toHaveBeenCalledTimes(1);
  });

  it('should support returning cleanup function from effect', () => {
    const count = signal(0);
    const cleanup = vi.fn();

    const stop = effect(() => {
      count.value;
      return cleanup;
    });

    expect(cleanup).not.toHaveBeenCalled();

    count.value = 1;

    expect(cleanup).toHaveBeenCalledTimes(1);

    stop();

    expect(cleanup).toHaveBeenCalledTimes(2);
  });

  it('should create and clean up interval', () => {
    const callback = vi.fn();
    const clear = createInterval(callback, 1000);

    expect(callback).not.toHaveBeenCalled();

    vi.advanceTimersByTime(1000);

    expect(callback).toHaveBeenCalledTimes(1);

    vi.advanceTimersByTime(1000);

    expect(callback).toHaveBeenCalledTimes(2);

    clear();

    vi.advanceTimersByTime(1000);

    expect(callback).toHaveBeenCalledTimes(2);
  });

  it('should create and clean up timeout', () => {
    const callback = vi.fn();
    const clear = createTimeout(callback, 1000);

    expect(callback).not.toHaveBeenCalled();

    vi.advanceTimersByTime(500);

    expect(callback).not.toHaveBeenCalled();

    vi.advanceTimersByTime(500);

    expect(callback).toHaveBeenCalledTimes(1);

    // Timeout should only run once
    vi.advanceTimersByTime(1000);

    expect(callback).toHaveBeenCalledTimes(1);

    // Clear should have no effect after timeout has fired
    clear();
  });

  it('should create and clean up event listener', () => {
    // Mock DOM event
    const target = {
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    } as unknown as HTMLElement;

    const listener = vi.fn();
    const remove = createEventListener(target, 'click' as any, listener as any);

    expect(target.addEventListener).toHaveBeenCalledWith(
      'click',
      listener,
      undefined
    );
    expect(target.removeEventListener).not.toHaveBeenCalled();

    remove();

    expect(target.removeEventListener).toHaveBeenCalledWith(
      'click',
      listener,
      undefined
    );
  });

  it('should create and clean up resource', () => {
    const setup = vi.fn();
    const cleanup = vi.fn();

    // Создаем эффект, который будет использовать ресурс
    const stop = effect(() => {
      const resource = createResource(() => {
        setup();
        return ['resource', cleanup];
      });

      expect(resource).toBe('resource');
    });

    expect(setup).toHaveBeenCalledTimes(1);
    expect(cleanup).not.toHaveBeenCalled();

    // Останавливаем эффект, что должно вызвать очистку ресурса
    stop();

    expect(cleanup).toHaveBeenCalledTimes(1);
  });

  it('should create and clean up effect', () => {
    const count = signal(0);
    const run = vi.fn();
    const cleanup = vi.fn();

    const stop = createEffect(() => {
      run(count.value);
      return cleanup;
    });

    expect(run).toHaveBeenCalledWith(0);
    expect(cleanup).not.toHaveBeenCalled();

    count.value = 1;

    expect(run).toHaveBeenCalledWith(1);
    expect(cleanup).toHaveBeenCalledTimes(1);

    stop();

    expect(cleanup).toHaveBeenCalledTimes(2);

    count.value = 2;

    // Effect should not run after being stopped
    expect(run).not.toHaveBeenCalledWith(2);
  });

  it('should create and clean up disposable scope', () => {
    const setup = vi.fn();
    const cleanup1 = vi.fn();
    const cleanup2 = vi.fn();

    // Создаем скоуп с функциями очистки
    const dispose = createScope(() => {
      setup();
      onCleanup(cleanup1);
      onCleanup(cleanup2);
    });

    expect(setup).toHaveBeenCalledTimes(1);
    expect(cleanup1).not.toHaveBeenCalled();
    expect(cleanup2).not.toHaveBeenCalled();

    // Вызываем функцию dispose, что должно вызвать функции очистки
    dispose();

    expect(cleanup1).toHaveBeenCalledTimes(1);
    expect(cleanup2).toHaveBeenCalledTimes(1);
  });

  it('should support nested disposable scopes', () => {
    const cleanup1 = vi.fn();
    const cleanup2 = vi.fn();
    const cleanup3 = vi.fn();

    // Создаем вложенные скоупы
    const dispose = createScope(() => {
      onCleanup(cleanup1);

      // Создаем вложенный скоуп
      const _ = createScope(() => {
        onCleanup(cleanup2);
      });

      onCleanup(cleanup3);
    });

    expect(cleanup1).not.toHaveBeenCalled();
    expect(cleanup2).not.toHaveBeenCalled();
    expect(cleanup3).not.toHaveBeenCalled();

    // Вызываем функцию dispose, что должно вызвать все функции очистки
    dispose();

    expect(cleanup1).toHaveBeenCalledTimes(1);
    expect(cleanup2).toHaveBeenCalledTimes(1);
    expect(cleanup3).toHaveBeenCalledTimes(1);
  });

  it('should handle errors in cleanup functions', () => {
    const consoleErrorSpy = vi
      .spyOn(console, 'error')
      .mockImplementation(() => {});

    const error = new Error('Cleanup error');
    const cleanup = vi.fn(() => {
      throw error;
    });

    const stop = effect(() => {
      onCleanup(cleanup);
    });

    stop();

    expect(cleanup).toHaveBeenCalledTimes(1);
    expect(consoleErrorSpy).toHaveBeenCalledWith(
      'Error in cleanup function:',
      error
    );
  });
});

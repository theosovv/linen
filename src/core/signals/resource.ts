import { effect, onCleanup } from './signal';

/**
 * Creates a resource that will be automatically cleaned up.
 * @param setup Setup function that returns a cleanup function
 * @returns The resource
 */
export function createResource<T>(setup: () => [T, () => void]): T {
  const [resource, cleanup] = setup();

  onCleanup(cleanup);

  return resource;
}

/**
 * Creates an interval that will be automatically cleared.
 * @param callback Callback function
 * @param delay Delay in milliseconds
 * @returns Clear function
 */
export function createInterval(
  callback: () => void,
  delay: number
): () => void {
  const id = setInterval(callback, delay);

  const clear = () => clearInterval(id);

  onCleanup(clear);

  return clear;
}

/**
 * Creates a timeout that will be automatically cleared.
 * @param callback Callback function
 * @param delay Delay in milliseconds
 * @returns Clear function
 */
export function createTimeout(callback: () => void, delay: number): () => void {
  const id = setTimeout(callback, delay);

  const clear = () => clearTimeout(id);
  onCleanup(clear);

  return clear;
}

/**
 * Adds an event listener that will be automatically removed.
 * @param target Target element
 * @param type Event type
 * @param listener Event listener
 * @param options Event listener options
 * @returns Remove function
 */
export function createEventListener<K extends keyof HTMLElementEventMap>(
  target: HTMLElement | Window | Document,
  type: K,
  listener: (this: HTMLElement, ev: HTMLElementEventMap[K]) => unknown,
  options?: boolean | AddEventListenerOptions
): () => void {
  target.addEventListener(type, listener as EventListener, options);

  const remove = () =>
    target.removeEventListener(type, listener as EventListener, options);
  onCleanup(remove);

  return remove;
}

/**
 * Creates a reactive effect that will be automatically cleaned up.
 * @param fn Effect function
 * @returns Stop function
 */
export function createEffect(fn: () => void | (() => void)): () => void {
  return effect(fn);
}

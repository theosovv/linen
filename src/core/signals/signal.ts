type Fn = () => void;

export interface SignalObject<T> {
  value: T;
  _version?: number;
}

let currentEffect: Fn | null = null;
const effectsStack: Fn[] = [];
const subscribers = new WeakMap<object, Map<PropertyKey, Set<Fn>>>();
const effectDependencies = new Map<Fn, Set<[object, PropertyKey]>>();
let batchDepth = 0;
const pendingEffects = new Set<Fn>();

/**
 * Creates a new signal with the given initial value.
 * @param initialValue The initial value of the signal.
 * @returns A signal object that can be used to get and set the value of the signal.
 */
export function signal<T>(initialValue: T) {
  const signalObject: SignalObject<T> = {
    value: initialValue,
    _version: 0,
  };

  if (!subscribers.has(signalObject)) {
    subscribers.set(signalObject, new Map());
  }

  const signalSubscribers = subscribers.get(signalObject)!;

  if (!signalSubscribers.has('value')) {
    signalSubscribers.set('value', new Set<Fn>());
  }

  return {
    get value() {
      track(signalObject, 'value');

      return signalObject.value;
    },

    set value(newValue: T) {
      if (Object.is(signalObject.value, newValue)) {
        return;
      }

      signalObject.value = newValue;
      signalObject._version = (signalObject._version || 0) + 1;

      trigger(signalObject, 'value');
    },
    peek(): T {
      return signalObject.value;
    },
  };
}

/**
 * Tracks dependency
 * @param target The target object to track.
 * @param key The property key to track.
 */
function track(target: object, key: PropertyKey) {
  if (!currentEffect) return;

  let depsMap = subscribers.get(target);

  if (!depsMap) {
    depsMap = new Map();

    subscribers.set(target, depsMap);
  }

  let dep = depsMap.get(key);

  if (!dep) {
    dep = new Set();

    depsMap.set(key, dep);
  }

  dep.add(currentEffect);

  let effectsDeps = effectDependencies.get(currentEffect);

  if (!effectsDeps) {
    effectsDeps = new Set();

    effectDependencies.set(currentEffect, effectsDeps);
  }

  effectsDeps.add([target, key]);
}

/**
 * Runs the function without tracking dependencies.
 * @param fn Function to run
 * @returns Result of the function
 */
export function untrack<T>(fn: () => T): T {
  const prevEffect = currentEffect;

  currentEffect = null;

  try {
    return fn();
  } finally {
    currentEffect = prevEffect;
  }
}

/**
 * Triggers a signal update.
 * @param target The target object to trigger.
 * @param key The property key to trigger.
 */
function trigger(target: object, key: PropertyKey) {
  const depsMap = subscribers.get(target);

  if (!depsMap) return;

  const dep = depsMap.get(key);

  if (!dep) return;

  const effects = [...dep];

  for (const effect of effects) {
    if (batchDepth > 0) {
      pendingEffects.add(effect);
    } else {
      effect();
    }
  }
}

/**
 * Creates an effect that is automatically triggered when dependencies change.
 * @param fn Effect function
 * @returns Function to stop the effect
 */
export function effect(fn: Fn) {
  const execute = () => {
    cleanup(execute);

    effectsStack.push(execute);

    currentEffect = execute;

    try {
      fn();
    } finally {
      effectsStack.pop();

      currentEffect = effectsStack[effectsStack.length - 1] ?? null;
    }
  };

  execute();

  return () => {
    cleanup(execute);
  };
}

/**
 * Cleans up effect dependencies.
 * @param effect Effect function
 */
function cleanup(effect: Fn) {
  const deps = effectDependencies.get(effect);

  if (!deps) return;

  for (const [target, key] of deps) {
    const depsMap = subscribers.get(target);

    if (!depsMap) continue;

    const targetDeps = depsMap.get(key);

    if (!targetDeps) continue;

    targetDeps.delete(effect);
  }

  deps.clear();
}

/**
 * Runs the function in a batch.
 * @param fn Function to run
 */
export function batch<T>(fn: () => T): T {
  batchDepth++;

  try {
    return fn();
  } finally {
    batchDepth--;

    if (batchDepth === 0 && pendingEffects.size > 0) {
      const effects = [...pendingEffects];

      pendingEffects.clear();

      for (const effect of effects) {
        effect();
      }
    }
  }
}

/**
 * Creates a transaction that automatically batches all signal updates.
 * @param fn Function to run in transaction
 * @returns Result of the function
 */
export function transaction<T>(fn: () => T): T {
  return batch(fn);
}

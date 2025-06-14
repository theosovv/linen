import { signal, computed, effect } from '../../src';

export function signalPlayground() {
  const count = signal(0);
  const multiplier = signal(2);

  const doubled = computed(() => count.value * multiplier.value);

  effect(() => {
    console.log(
      `count: ${count.value}, multiplier: ${multiplier.value}, doubled: ${doubled.value}`
    );
  });

  count.value = 5;
  multiplier.value = 3;
}

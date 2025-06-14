import { batch, computed, effect, signal, transaction } from '../../src';

export function batchPlayground() {
  console.log('--- Example 1: Basic batch usage ---');

  const count = signal(0);
  const double = computed(() => count.value * 2);

  effect(() => {
    console.log(`count: ${count.value}, double: ${double.value}`);
  });

  console.log('Without batching:');

  count.value = 1;
  count.value = 2;
  count.value = 3;

  console.log('\nWith batching:');

  batch(() => {
    count.value = 4;
    count.value = 5;
    count.value = 6;
  });

  console.log('\n--- Example 2: Nested batches ---');

  const x = signal(0);
  const y = signal(0);
  const sum = computed(() => x.value + y.value);

  effect(() => {
    console.log(`x: ${x.value}, y: ${y.value}, sum: ${sum.value}`);
  });

  batch(() => {
    x.value = 10;

    batch(() => {
      y.value = 20;
      x.value = 15;
    });

    y.value = 25;
  });

  console.log('\n--- Example 3: Transaction (alias for batch) ---');

  const counter = signal(0);
  const isEven = computed(() => counter.value % 2 === 0);

  effect(() => {
    console.log(`Counter: ${counter.value}, Is even: ${isEven.value}`);
  });

  transaction(() => {
    counter.value = 1;
    counter.value = 2;
    counter.value = 3;
    counter.value = 4;
  });
}

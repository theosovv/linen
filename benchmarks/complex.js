import { Bench } from 'tinybench';
import * as fs from 'fs';
import * as path from 'path';
import { signal, computed, effect } from '../dist/index.esm.js';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

async function runComplexBenchmarks() {
  console.log('Running complex benchmarks...');

  const bench = new Bench({ time: 1000 });

  bench.add('deep dependency chain', () => {
    const a = signal(1);
    const b = computed(() => a.value + 1);
    const c = computed(() => b.value * 2);
    const d = computed(() => c.value + 3);
    const e = computed(() => d.value * 4);
    const f = computed(() => e.value + 5);
    const g = computed(() => f.value * 6);
    const h = computed(() => g.value + 7);
    const i = computed(() => h.value * 8);
    const j = computed(() => i.value + 9);

    a.value = 2;
    console.log(j.value);
  });

  bench.add('many effects', () => {
    const count = signal(0);

    for (let i = 0; i < 100; i++) {
      effect(() => {
        console.log(count.value);
      });
    }

    count.value = 1;
  });

  bench.add('many computed values', () => {
    const count = signal(0);
    const computedValues = [];

    for (let i = 0; i < 100; i++) {
      computedValues.push(computed(() => count.value * i));
    }

    count.value = 1;

    for (const c of computedValues) {
      console.log(c.value);
    }
  });

  bench.add('conditional dependencies', () => {
    const condition = signal(true);
    const a = signal(1);
    const b = signal(2);

    const result = computed(() => {
      return condition.value ? a.value : b.value;
    });

    condition.value = false;
    console.log(result.value);

    condition.value = true;
    console.log(result.value);
  });

  await bench.run();

  console.table(bench.table());

  const resultsDir = path.resolve(__dirname, '../benchmark-results');
  if (!fs.existsSync(resultsDir)) {
    fs.mkdirSync(resultsDir, { recursive: true });
  }

  fs.writeFileSync(
    path.join(resultsDir, 'complex.json'),
    JSON.stringify(bench.table(), null, 2)
  );

  console.log('Complex benchmark results saved to benchmark-results/complex.json');
}

runComplexBenchmarks().catch(console.error);

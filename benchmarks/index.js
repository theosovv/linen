import { Bench } from 'tinybench';
import { signal, computed, effect } from '../dist/index.esm.js';
import * as fs from 'fs';
import * as path from 'path';

import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

async function runBenchmarks() {
  const bench = new Bench({ time: 1000 });

  bench
    .add('signal creation', () => {
      console.log(signal(0));
    })
    .add('signal read', () => {
      const count = signal(0);
      console.log(count.value);
    })
    .add('signal write', () => {
      const count = signal(0);
      count.value = 1;
    })
    .add('computed creation', () => {
      const count = signal(0);
      console.log(computed(() => count.value * 2).value);
    })
    .add('computed read', () => {
      const count = signal(0);
      const doubled = computed(() => count.value * 2);
      console.log(doubled.value);
    })
    .add('effect creation', () => {
      const count = signal(0);
      effect(() => {
        console.log(count.value);
      });
    })
    .add('signal update with effect', () => {
      const count = signal(0);
      effect(() => {
        console.log(count.value);
      });
      count.value = 1;
    });

  await bench.run();

  console.table(bench.table());

  const resultsDir = path.resolve(__dirname, '../benchmark-results');
  if (!fs.existsSync(resultsDir)) {
    fs.mkdirSync(resultsDir, { recursive: true });
  }

  fs.writeFileSync(
    path.join(resultsDir, 'basic.json'),
    JSON.stringify(bench.table(), null, 2)
  );

  console.log('Basic benchmark results saved to benchmark-results/basic.json');
}

runBenchmarks().catch(console.error);

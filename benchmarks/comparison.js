import { Bench } from 'tinybench';
import { signal as linenSignal, computed as linenComputed } from '../dist/index.esm.js';
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

async function runComparisonBenchmarks() {
  console.log('Running comparison benchmarks...');

  const bench = new Bench({ time: 1000 });

  bench.add('Linen - create 1000 signals', () => {
    const signals = [];
    for (let i = 0; i < 1000; i++) {
      signals.push(linenSignal(i));
    }
  });

  bench.add('Linen - read 1000 signals', () => {
    const signals = [];
    for (let i = 0; i < 1000; i++) {
      signals.push(linenSignal(i));
    }

    for (const s of signals) {
      console.log(s.value);
    }
  });
  bench.add('Linen - update 1000 signals', () => {
    const signals = [];
    for (let i = 0; i < 1000; i++) {
      signals.push(linenSignal(i));
    }

    for (let i = 0; i < 1000; i++) {
      signals[i].value = i + 1;
    }
  });

  bench.add('Linen - computed with 10 dependencies', () => {
    const signals = [];
    for (let i = 0; i < 10; i++) {
      signals.push(linenSignal(i));
    }

    const computed = linenComputed(() => {
      let sum = 0;
      for (const s of signals) {
        sum += s.value;
      }
      return sum;
    });

    console.log(computed.value);
  });

  await bench.run();

  console.table(bench.table());

  const resultsDir = path.resolve(__dirname, '../benchmark-results');
  if (!fs.existsSync(resultsDir)) {
    fs.mkdirSync(resultsDir, { recursive: true });
  }

  fs.writeFileSync(
    path.join(resultsDir, 'comparison.json'),
    JSON.stringify(bench.table(), null, 2)
  );

  console.log(
    'Comparison benchmark results saved to benchmark-results/comparison.json'
  );
}

runComparisonBenchmarks().catch(console.error);

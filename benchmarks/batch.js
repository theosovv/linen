import { Bench } from 'tinybench';
import fs from 'fs';
import * as path from 'path';
import { signal, effect, batch } from '../dist/index.esm.js';

async function runBatchBenchmarks() {
  console.log('=== Batch Update Benchmarks ===');
  
  const bench = new Bench({ time: 1000 });

  bench.add('single update without batch', () => {
    const count = signal(0);
    let updated = false;
    
    effect(() => {
      updated = true;
      count.value;
    });
    
    updated = false;
    count.value = 1;
  });
  
  bench.add('single update with batch', () => {
    const count = signal(0);
    let updated = false;
    
    effect(() => {
      updated = true;
      count.value;
    });
    
    updated = false;
    batch(() => {
      count.value = 1;
    });
  });
  
  bench.add('multiple updates without batch', () => {
    const count = signal(0);
    let updateCount = 0;
    
    effect(() => {
      updateCount++;
      count.value;
    });
    
    updateCount = 0;
    count.value = 1;
    count.value = 2;
    count.value = 3;
    count.value = 4;
    count.value = 5;
  });
  
  bench.add('multiple updates with batch', () => {
    const count = signal(0);
    let updateCount = 0;
    
    effect(() => {
      updateCount++;
      count.value;
    });
    
    updateCount = 0;
    batch(() => {
      count.value = 1;
      count.value = 2;
      count.value = 3;
      count.value = 4;
      count.value = 5;
    });
  });
  
  bench.add('nested batch operations', () => {
    const x = signal(0);
    const y = signal(0);
    let updateCount = 0;
    
    effect(() => {
      updateCount++;
      x.value;
      y.value;
    });
    
    updateCount = 0;
    batch(() => {
      x.value = 1;
      
      batch(() => {
        y.value = 1;
        x.value = 2;
      });
      
      y.value = 2;
    });
  });
  
  bench.add('many signals in batch (10)', () => {
    const signals = Array.from({ length: 10 }, () => signal(0));
    let updateCount = 0;
    
    effect(() => {
      updateCount++;
      signals.forEach(s => s.value);
    });
    
    updateCount = 0;
    batch(() => {
      signals.forEach((s, i) => {
        s.value = i;
      });
    });
  });
  
  bench.add('many signals in batch (100)', () => {
    const signals = Array.from({ length: 100 }, () => signal(0));
    let updateCount = 0;
    
    effect(() => {
      updateCount++;
      signals.forEach(s => s.value);
    });
    
    updateCount = 0;
    batch(() => {
      signals.forEach((s, i) => {
        s.value = i;
      });
    });
  });
  
  bench.add('computed values in batch', () => {
    const count = signal(0);
    const double = signal(0);
    const triple = signal(0);
    let updateCount = 0;
    
    effect(() => {
      double.value = count.value * 2;
    });
    
    effect(() => {
      triple.value = count.value * 3;
    });
    
    effect(() => {
      updateCount++;
      count.value;
      double.value;
      triple.value;
    });
    
    updateCount = 0;
    batch(() => {
      count.value = 1;
      count.value = 2;
      count.value = 3;
      count.value = 4;
      count.value = 5;
    });
  });
  
  bench.add('error handling in batch', () => {
    const count = signal(0);
    let updateCount = 0;
    
    effect(() => {
      updateCount++;
      count.value;
    });
    
    updateCount = 0;
    try {
      batch(() => {
        count.value = 1;
        count.value = 2;
        throw new Error('Test error');
      });
    } catch (e) {
      // Ignore error for benchmark
    }
  });

  // Run all benchmarks
  await bench.run();

  // Print results in a table
  console.table(bench.table());
  // Print comparison between batched and non-batched operations
  console.log('\n=== Performance Comparison ===');
  
  const singleWithout = bench.tasks.find(t => t.name === 'single update without batch').result;
  const singleWith = bench.tasks.find(t => t.name === 'single update with batch').result;
  
  const multipleWithout = bench.tasks.find(t => t.name === 'multiple updates without batch').result;
  const multipleWith = bench.tasks.find(t => t.name === 'multiple updates with batch').result;
  
  console.log(`Single update: batch is ${(singleWithout.mean / singleWith.mean).toFixed(2)}x faster`);
  console.log(`Multiple updates: batch is ${(multipleWithout.mean / multipleWith.mean).toFixed(2)}x faster`);  
  
  const csvContent = [
    ['Task name', 'Latency avg (ns)', 'Latency med (ns)', 'Throughput avg (ops/s)', 'Samples'].join(','),
    ...bench.tasks.map(({ name, result }) => [
      name,
      `${result?.mean.toFixed(2) * 1000000} ± ${(result?.sd / result?.mean * 100).toFixed(2)}%`,
      `${result?.median.toFixed(2) * 1000000} ± ${result?.mad.toFixed(2) * 1000000}`,
      `${(1 / result?.mean).toFixed(0)}`,
      result?.samples.length
    ].join(','))
  ].join('\n');
  
  const csvDir = path.join(__dirname, '..', 'benchmark-csv');
  if (!fs.existsSync(csvDir)) {
    fs.mkdirSync(csvDir, { recursive: true });
  }
  
  fs.writeFileSync(path.join(csvDir, 'batch.csv'), csvContent);
  console.log(`\nResults saved to ${path.join(csvDir, 'batch.csv')}`);
}

runBatchBenchmarks().catch(console.error);

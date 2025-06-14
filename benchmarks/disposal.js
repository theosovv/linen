import { Bench } from 'tinybench';
import fs from 'fs';
import path from 'path';
import { 
  signal, 
  effect, 
  onCleanup, 
  createResource,
  createInterval,
  createTimeout,
  createEffect,
  createScope
} from '../dist/index.esm.js';

async function runDisposalBenchmarks() {
  console.log('=== Disposal and Cleanup Benchmarks ===');
  
  const bench = new Bench({ time: 1000 });

  bench.add('effect with onCleanup', () => {
    const count = signal(0);
    const cleanup = () => {};
    
    const stop = effect(() => {
      count.value;
      onCleanup(cleanup);
    });
    
    count.value = 1;
    stop();
  });
  
  bench.add('effect returning cleanup', () => {
    const count = signal(0);
    const cleanup = () => {};
    
    const stop = effect(() => {
      count.value;
      return cleanup;
    });
    
    count.value = 1;
    stop();
  });
  
  bench.add('createResource', () => {
    const resource = createResource(() => {
      const data = {};
      return [data, () => {}];
    });
  });
  
  bench.add('createInterval and clear', () => {
    const clear = createInterval(() => {}, 1000);
    clear();
  });
  
  bench.add('createTimeout and clear', () => {
    const clear = createTimeout(() => {}, 1000);
    clear();
  });
  
  bench.add('createEffect and stop', () => {
    const count = signal(0);
    const stop = createEffect(() => {
      count.value;
      return () => {};
    });
    
    count.value = 1;
    stop();
  });
  
  bench.add('createScope with single cleanup', () => {
    const dispose = createScope(() => {
      onCleanup(() => {});
    });
    
    dispose();
  });
  
  bench.add('createScope with multiple cleanups', () => {
    const dispose = createScope(() => {
      onCleanup(() => {});
      onCleanup(() => {});
      onCleanup(() => {});
      onCleanup(() => {});
      onCleanup(() => {});
    });
    
    dispose();
  });
  
  bench.add('nested createScope', () => {
    const dispose = createScope(() => {
      onCleanup(() => {});
      
      createScope(() => {
        onCleanup(() => {});
      });
      
      onCleanup(() => {});
    });
    
    dispose();
  });
  
  bench.add('error handling in cleanup', () => {
    const originalConsoleError = console.error;
    console.error = () => {};
    
    const dispose = createScope(() => {
      onCleanup(() => {
        throw new Error('Test error');
      });
      
      onCleanup(() => {});
    });
    
    dispose();
    
    console.error = originalConsoleError;
  });

  // Run all benchmarks
  await bench.run();

  // Print results in a table
  console.table(bench.tasks);  
  
  const csvDir = path.join(__dirname, '..', 'benchmark-csv');
  if (!fs.existsSync(csvDir)) {
    fs.mkdirSync(csvDir, { recursive: true });
  }
  
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
  
  fs.writeFileSync(path.join(csvDir, 'disposal.csv'), csvContent);
  console.log(`\nResults saved to ${path.join(csvDir, 'disposal.csv')}`);
  
  // Compare different cleanup approaches
  console.log('\n=== Cleanup Approaches Comparison ===');
  
  const onCleanupResult = bench.tasks.find(t => t.name === 'effect with onCleanup').result;
  const returnCleanupResult = bench.tasks.find(t => t.name === 'effect returning cleanup').result;
  
  console.log(`onCleanup vs returning cleanup: ${(returnCleanupResult.mean / onCleanupResult.mean).toFixed(2)}x ratio`);
  
  const singleCleanupResult = bench.tasks.find(t => t.name === 'createScope with single cleanup').result;
  const multipleCleanupResult = bench.tasks.find(t => t.name === 'createScope with multiple cleanups').result;
  
  console.log(`Multiple cleanups overhead: ${(multipleCleanupResult.mean / singleCleanupResult.mean).toFixed(2)}x slower than single cleanup`);
  
  const nestedScopeResult = bench.tasks.find(t => t.name === 'nested createScope').result;
  
  console.log(`Nested scope overhead: ${(nestedScopeResult.mean / singleCleanupResult.mean).toFixed(2)}x slower than single scope`);
  
  const errorHandlingResult = bench.tasks.find(t => t.name === 'error handling in cleanup').result;
  
  console.log(`Error handling overhead: ${(errorHandlingResult.mean / singleCleanupResult.mean).toFixed(2)}x slower than normal cleanup`);
}

runDisposalBenchmarks().catch(console.error);

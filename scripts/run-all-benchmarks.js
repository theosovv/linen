import { execSync } from 'child_process';
import * as path from 'path';
import * as fs from 'fs';

import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const benchmarksDir = path.resolve(__dirname, '../benchmarks');
const resultsDir = path.resolve(__dirname, '../benchmark-results');

if (!fs.existsSync(resultsDir)) {
  fs.mkdirSync(resultsDir, { recursive: true });
}

function runBenchmark(name, file) {
  console.log(`\n=== ${name} ===`);
  try {
    execSync(`npx ts-node ${file}`, { stdio: 'inherit' });
    console.log(`${name} completed successfully.`);
  } catch (error) {
    console.error(`Error running ${name}:`, error.message);
  }
}

console.log('Running all benchmarks...');

runBenchmark('Basic Benchmarks', path.join(benchmarksDir, 'index.js'));
runBenchmark('Comparison Benchmarks', path.join(benchmarksDir, 'comparison.js'));
runBenchmark('Complex Benchmarks', path.join(benchmarksDir, 'complex.js'));
runBenchmark('Real-World Benchmarks', path.join(benchmarksDir, 'real-world.js'));
runBenchmark('Events Benchmarks', path.join(benchmarksDir, 'events.js'));

console.log('\nAll benchmarks completed!');
console.log('Generating CSV reports...');

try {
  execSync(`node ${path.join(__dirname, 'generate-csv.js')}`, { stdio: 'inherit' });
  console.log('CSV reports generated successfully!');
} catch (error) {
  console.error('Error generating CSV reports:', error.message);
}

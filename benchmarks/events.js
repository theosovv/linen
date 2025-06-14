import { Bench } from 'tinybench';
import { createPubSub, eventSignal, eventEffect, publishEffect, signal } from '../dist/index.esm.js';
import fs from 'fs';
import path from 'path';

const bench = new Bench({ time: 1000 });

bench.add('pubsub creation', () => {
  createPubSub();
});

bench.add('event subscription', () => {
  const pubsub = createPubSub();
  const unsubscribe = pubsub.subscribe('test', () => {});
  unsubscribe();
});

bench.add('event publishing', () => {
  const pubsub = createPubSub();
  pubsub.subscribe('test', () => {});
  pubsub.publish('test', { data: 'test' });
});

bench.add('eventSignal creation', () => {
  const pubsub = createPubSub();
  const sig = eventSignal('test', null, pubsub);
  return sig;
});

bench.add('eventSignal update', () => {
  const pubsub = createPubSub();
  const sig = eventSignal('test', null, pubsub);
  pubsub.publish('test', { data: 'test' });
  return sig.value;
});

bench.add('eventEffect creation', () => {
  const pubsub = createPubSub();
  const stop = eventEffect('test', () => {}, pubsub);
  stop();
});

bench.add('eventEffect execution', () => {
  const pubsub = createPubSub();
  let counter = 0;
  const stop = eventEffect('test', () => { counter++ }, pubsub);
  pubsub.publish('test', { data: 'test' });
  stop();
  return counter;
});

bench.add('publishEffect creation', () => {
  const pubsub = createPubSub();
  const count = signal(0);
  const stop = publishEffect('test', () => count.value, pubsub);
  stop();
});

bench.add('publishEffect execution', () => {
  const pubsub = createPubSub();
  const count = signal(0);
  let received = null;
  
  pubsub.subscribe('test', (data) => {
    received = data;
  });
  
  const stop = publishEffect('test', () => count.value, pubsub);
  count.value = 42;
  stop();
  
  return received;
});

bench.add('many subscribers (100)', () => {
  const pubsub = createPubSub();
  const unsubscribes = [];
  
  for (let i = 0; i < 100; i++) {
    unsubscribes.push(pubsub.subscribe('test', () => {}));
  }
  
  pubsub.publish('test', { data: 'test' });
  
  unsubscribes.forEach(unsub => unsub());
});

await bench.run();

console.table(bench.table());

const results = bench.table();
const csvData = [
  'Task name,Latency avg (ns),Latency med (ns),Throughput avg (ops/s),Throughput med (ops/s),Samples',
  ...results.map(result => [
    result.name,
    `${result['Latency avg (ns)']} ± ${result['stddev (%)'] * 100}%`,
    `${result['Latency med (ns)']} ± ${result['median-stddev (ns)']}`,
    `${result['Throughput avg (ops/s)']} ± ${result['stddev (%)'] * 100}%`,
    `${result['Throughput med (ops/s)']} ± ${result['median-stddev']}`,
    result.samples
  ].join(','))
].join('\n');

const benchmarkDir = path.join(process.cwd(), 'benchmark-csv');
if (!fs.existsSync(benchmarkDir)) {
  fs.mkdirSync(benchmarkDir, { recursive: true });
}

fs.writeFileSync(path.join(benchmarkDir, 'events.csv'), csvData);
console.log('Benchmark results saved to benchmark-csv/events.csv');
import { Bench } from 'tinybench';
import * as fs from 'fs';
import * as path from 'path';
import { signal, computed } from '../dist/index.esm.js';

import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

async function runRealWorldBenchmarks() {
  console.log('Running real-world benchmarks...');

  const bench = new Bench({ time: 1000 });

  bench.add('form validation', () => {
    const username = signal('');
    const email = signal('');
    const password = signal('');

    const isUsernameValid = computed(() => username.value.length >= 3);
    const isEmailValid = computed(() =>
      /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email.value)
    );
    const isPasswordValid = computed(() => password.value.length >= 8);

    const isFormValid = computed(
      () => isUsernameValid.value && isEmailValid.value && isPasswordValid.value
    );

    username.value = 'john';
    email.value = 'john@example.com';
    password.value = 'password123';

    console.log(isFormValid.value);
  });

  bench.add('list filtering', () => {
    const items = signal(
      Array.from({ length: 1000 }, (_, i) => ({ id: i, value: `Item ${i}` }))
    );
    const filter = signal('');

    const filteredItems = computed(() => {
      const f = filter.value.toLowerCase();
      if (!f) return items.value;
      return items.value.filter(item => item.value.toLowerCase().includes(f));
    });

    filter.value = 'item 5';
    console.log(filteredItems.value);
  });

  bench.add('shopping cart', () => {
    const products = signal([
      { id: 1, name: 'Product 1', price: 10 },
      { id: 2, name: 'Product 2', price: 20 },
      { id: 3, name: 'Product 3', price: 30 },
    ]);

    const cart = signal([
      { productId: 1, quantity: 2 },
      { productId: 3, quantity: 1 },
    ]);

    const cartItems = computed(() => {
      return cart.value.map(item => {
        const product = products.value.find(p => p.id === item.productId);
        return {
          ...item,
          product,
          total: product ? product.price * item.quantity : 0,
        };
      });
    });

    const cartTotal = computed(() => {
      return cartItems.value.reduce((sum, item) => sum + item.total, 0);
    });

    cart.value = [...cart.value, { productId: 2, quantity: 3 }];

    cart.value = cart.value.map(item =>
      item.productId === 1 ? { ...item, quantity: 4 } : item
    );

    console.log(cartTotal.value);
  });

  await bench.run();

  console.table(bench.table());

  const resultsDir = path.resolve(__dirname, '../benchmark-results');
  if (!fs.existsSync(resultsDir)) {
    fs.mkdirSync(resultsDir, { recursive: true });
  }

  fs.writeFileSync(
    path.join(resultsDir, 'real-world.json'),
    JSON.stringify(bench.table(), null, 2)
  );

  console.log('Real-world benchmark results saved to benchmark-results/real-world.json');
}

runRealWorldBenchmarks().catch(console.error);

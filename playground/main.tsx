/** @jsx Linen */
// eslint-disable-next-line @typescript-eslint/no-unused-vars, unused-imports/no-unused-imports
import Linen, { signal, render } from '../src';

function Counter() {
  const count = signal(0);

  return (
    <div>
      <button onClick={() => count.value++}>+</button>
      <span>{count}</span>
      <button onClick={() => count.value--}>-</button>
    </div>
  );
}

render((<Counter />) as Node, document.getElementById('app')!);

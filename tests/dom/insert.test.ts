import { describe, it, expect } from 'vitest';
import { insert } from '../../src/core/dom/insert';
import { signal } from '../../src';

describe('insert', () => {
  it('inserts text', () => {
    const div = document.createElement('div');
    insert(div, 'hello');
    expect(div.textContent).toBe('hello');
  });

  it('inserts DOM node', () => {
    const div = document.createElement('div');
    const span = document.createElement('span');
    insert(div, span);
    expect(div.firstChild).toBe(span);
  });

  it('inserts array', () => {
    const div = document.createElement('div');
    insert(div, ['a', 'b']);
    expect(div.textContent).toBe('ab');
  });

  it('reactively updates text on signal change', async () => {
    const div = document.createElement('div');
    const s = signal('x');
    insert(div, s);
    expect(div.textContent).toBe('x');
    s.value = 'y';

    await Promise.resolve();
    expect(div.textContent).toBe('y');
  });
});

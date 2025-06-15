import { describe, it, expect } from 'vitest';
import { render } from '../../src';

describe('render', () => {
  it('clears container and inserts new node', () => {
    const container = document.createElement('div');
    container.innerHTML = '<span>old</span>';
    const node = document.createElement('b');
    render(node, container);
    expect(container.childNodes.length).toBe(1);
    expect(container.firstChild).toBe(node);
  });
});

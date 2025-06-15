import { describe, it, expect } from 'vitest';
import { Fragment } from '../../src';

describe('Fragment', () => {
  it('creates an empty DocumentFragment if children are not passed', () => {
    const frag = Fragment({ children: null });
    expect(frag).toBeInstanceOf(DocumentFragment);
    expect(frag.childNodes.length).toBe(0);
  });

  it('adds one child', () => {
    const span = document.createElement('span');
    const frag = Fragment({ children: span });
    expect(frag.childNodes.length).toBe(1);
    expect(frag.firstChild).toBe(span);
  });

  it('adds an array of children', () => {
    const a = document.createElement('a');
    const b = document.createElement('b');
    const frag = Fragment({ children: [a, b] });
    expect(frag.childNodes.length).toBe(2);
    expect(frag.childNodes[0]).toBe(a);
    expect(frag.childNodes[1]).toBe(b);
  });

  it('ignores null/undefined children', () => {
    const a = document.createElement('a');
    const frag = Fragment({ children: [null, a, undefined] });
    expect(frag.childNodes.length).toBe(1);
    expect(frag.firstChild).toBe(a);
  });
});

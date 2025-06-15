import { describe, it, expect, vi } from 'vitest';
import { Linen as h } from '../../src/core/dom';

describe('h', () => {
  it('creates an element by string', () => {
    const el = h('div');
    expect(el).toBeInstanceOf(HTMLDivElement);
  });

  it('sets attributes', () => {
    const el = h('div', { id: 'foo', title: 'bar' }) as HTMLElement;
    expect(el.getAttribute('id')).toBe('foo');
    expect(el.getAttribute('title')).toBe('bar');
  });
  it('adds children', () => {
    const el = h('div', {}, 'text', h('span'));
    expect(el.childNodes.length).toBe(2);
    expect(el.childNodes[0].textContent).toBe('text');
    expect(el.childNodes[1]).toBeInstanceOf(HTMLSpanElement);
  });

  it('handles events', () => {
    const onClick = vi.fn();
    const btn = h('button', { onClick });
    btn.dispatchEvent(new MouseEvent('click'));
    expect(onClick).toHaveBeenCalled();
  });

  it('calls a component function', () => {
    const Comp = vi.fn(() => h('div', {}, 'comp'));
    const el = h(Comp, { foo: 1 }, 'bar');
    expect(Comp).toHaveBeenCalledWith({ foo: 1, children: ['bar'] });
    expect(el).toBeInstanceOf(HTMLDivElement);
    expect(el.textContent).toBe('comp');
  });
});

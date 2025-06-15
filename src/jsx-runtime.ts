import { Fragment } from './core/dom';
import Linen, { Props } from './core/dom/h';

export { Fragment };

export function jsx(type: unknown, props: Props, _key?: unknown) {
  return Linen(type, props);
}

export function jsxs(type: unknown, props: Props, _key?: unknown) {
  return Linen(type, props);
}

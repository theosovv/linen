import { effect } from '../signals';
import { insert } from './insert';

export type Child =
  | Node
  | string
  | number
  | boolean
  | null
  | undefined
  | Child[];
export type Props = Record<string, unknown> & { children?: Child };

/**
 * Creates a new DOM element with the given type, props, and children.
 * @param type - The type of the element to create.
 * @param props - The props of the element to create.
 * @param children - The children of the element to create.
 * @returns The created element.
 */
function Linen(type: unknown, props: Props = {}, ...children: Child[]): Node {
  if (typeof type === 'function') {
    return type({ ...props, children });
  }

  const element = document.createElement(type as string);

  for (const [key, value] of Object.entries(props || {})) {
    if (key === 'children' || value == null) continue;
    if (key.startsWith('on') && typeof value === 'function') {
      element.addEventListener(
        key.slice(2).toLowerCase(),
        value as EventListener
      );
    } else if (typeof value === 'object' && value && 'value' in value) {
      // сигнал
      effect(() => {
        element.setAttribute(key, value.value as string);
      });
    } else {
      element.setAttribute(key, value as string);
    }
  }

  children.forEach(child => insert(element, child));

  return element;
}

export default Linen;

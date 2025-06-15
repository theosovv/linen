import { effect } from '../signals';

/**
 * Inserts a value into the DOM.
 * @param parent - The parent node to insert the value into.
 * @param value - The value to insert.
 * @returns The inserted node, or null if the value is not supported.
 */
export function insert(parent: Node, value: unknown) {
  if (Array.isArray(value)) {
    value.forEach(child => insert(parent, child));
    return;
  }
  if (value instanceof Node) {
    parent.appendChild(value);
    return;
  }
  if (value && typeof value === 'object' && 'value' in value) {
    // Это сигнал
    const marker = document.createTextNode(String(value.value));
    effect(() => {
      marker.textContent = String(value.value);
    });
    parent.appendChild(marker);
    return;
  }
  if (
    value !== null &&
    value !== undefined &&
    value !== false &&
    value !== true &&
    typeof value !== 'function'
  ) {
    parent.appendChild(document.createTextNode(String(value)));
  }
}

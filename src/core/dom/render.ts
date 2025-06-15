/**
 * Render the node to the container.
 * @param node - The node to render.
 * @param container - The container to render the node to.
 */
export function render(node: Node, container: Element): void {
  container.innerHTML = '';
  container.appendChild(node);
}

export function Fragment(props: { children: unknown }): DocumentFragment {
  const fragment = document.createDocumentFragment();

  if (props.children) {
    if (Array.isArray(props.children)) {
      props.children.forEach(child => {
        if (child) {
          fragment.appendChild(child);
        }
      });
    } else {
      fragment.appendChild(props.children as Node);
    }
  }

  return fragment;
}
